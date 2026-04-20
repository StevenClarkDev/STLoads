use leptos::{prelude::*, tachys::view::any_view::IntoAny, task::spawn_local};
use leptos_router::{components::A, hooks::use_params_map};

use crate::{
    api, document_upload,
    session::{self, use_auth},
};
use shared::{
    AdminReviewLoadRequest, EscrowFundRequest, EscrowHoldRequest, EscrowReleaseRequest,
    LoadDocumentRow, LoadHandoffSummary, LoadHistoryRow, LoadProfileLegRow, LoadProfileScreen,
    UpsertLoadDocumentRequest, VerifyLoadDocumentRequest,
};

fn tone_style(tone: &str) -> &'static str {
    match tone {
        "success" => {
            "background:#e8fff3;padding:0.25rem 0.55rem;border-radius:999px;color:#0f766e;"
        }
        "warning" => {
            "background:#fff7dd;padding:0.25rem 0.55rem;border-radius:999px;color:#b45309;"
        }
        "danger" => "background:#ffe4e6;padding:0.25rem 0.55rem;border-radius:999px;color:#be123c;",
        "info" => "background:#e0f2fe;padding:0.25rem 0.55rem;border-radius:999px;color:#0369a1;",
        "primary" => {
            "background:#ede9fe;padding:0.25rem 0.55rem;border-radius:999px;color:#6d28d9;"
        }
        "dark" => "background:#e5e7eb;padding:0.25rem 0.55rem;border-radius:999px;color:#111827;",
        _ => "background:#f1f5f9;padding:0.25rem 0.55rem;border-radius:999px;color:#475569;",
    }
}

fn human_file_size(bytes: Option<u64>) -> String {
    match bytes {
        Some(value) if value >= 1024 * 1024 => format!("{:.1} MB", value as f64 / 1024.0 / 1024.0),
        Some(value) if value >= 1024 => format!("{:.1} KB", value as f64 / 1024.0),
        Some(value) => format!("{} B", value),
        None => "Size not recorded".into(),
    }
}

fn render_handoff(handoff: Option<LoadHandoffSummary>, admin_mode: bool) -> AnyView {
    match handoff {
        Some(handoff) => {
            let reconciliation_href = match handoff.status_tone.as_str() {
                "danger" => Some("/admin/stloads/reconciliation?action=mismatch_detected".to_string()),
                "warning" => Some("/admin/stloads/reconciliation?action=auto_archive".to_string()),
                _ => None,
            };
            view! {
            <section style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#ffffff;display:grid;gap:0.85rem;">
                <div style="display:flex;justify-content:space-between;gap:1rem;align-items:flex-start;flex-wrap:wrap;">
                    <div>
                        <strong>"STLOADS board status"</strong>
                        <div><small>{format!("TMS Load {}", handoff.tms_load_id)}</small></div>
                    </div>
                    <div style="display:grid;gap:0.45rem;justify-items:end;">
                        <span style=tone_style(&handoff.status_tone)>{handoff.status_label}</span>
                        {admin_mode.then(|| view! {
                            <div style="display:flex;gap:0.45rem;flex-wrap:wrap;justify-content:flex-end;">
                                <A href="/admin/stloads/operations" attr:style="padding:0.45rem 0.7rem;border-radius:0.75rem;background:#eef2ff;color:#312e81;text-decoration:none;">
                                    "STLOADS Ops"
                                </A>
                                {reconciliation_href.clone().map(|href| view! {
                                    <A href=href attr:style="padding:0.45rem 0.7rem;border-radius:0.75rem;background:#fff7dd;color:#92400e;text-decoration:none;">
                                        "Reconciliation"
                                    </A>
                                })}
                            </div>
                        })}
                    </div>
                </div>
                <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:1rem;">
                    <div><small style="color:#64748b;">"Board rate"</small><div>{handoff.board_rate_label}</div></div>
                    <div><small style="color:#64748b;">"TMS status"</small><div>{handoff.tms_status_label.unwrap_or_else(|| "No upstream status yet".into())}</div></div>
                    <div><small style="color:#64748b;">"TMS status updated"</small><div>{handoff.tms_status_at_label.unwrap_or_else(|| "No upstream timestamp yet".into())}</div></div>
                    <div><small style="color:#64748b;">"Published"</small><div>{handoff.published_at_label.unwrap_or_else(|| "Not published yet".into())}</div></div>
                    <div><small style="color:#64748b;">"Pushed by"</small><div>{handoff.pushed_by_label.unwrap_or_else(|| "System".into())}</div></div>
                </div>
            </section>
        }.into_any()
        }
        None => view! {
            <section style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#fcfcfb;">
                "This load has not been pushed to STLOADS yet."
            </section>
        }.into_any(),
    }
}

fn render_legs(
    legs: Vec<LoadProfileLegRow>,
    admin_mode: bool,
    can_manage_payments: bool,
    finance_loading_leg_id: RwSignal<Option<u64>>,
    run_finance_action: impl Fn(LoadProfileLegRow) + Copy + 'static,
) -> AnyView {
    if legs.is_empty() {
        view! { <p style="margin:0;">"No legs are attached to this load yet."</p> }.into_any()
    } else {
        view! {
            <table style="width:100%;border-collapse:collapse;min-width:760px;">
                <thead style="background:#f8fafc;">
                    <tr>
                        <th style="text-align:left;padding:0.75rem;">"Leg"</th>
                        <th style="text-align:left;padding:0.75rem;">"Route"</th>
                        <th style="text-align:left;padding:0.75rem;">"Dates"</th>
                        <th style="text-align:left;padding:0.75rem;">"Status"</th>
                        <th style="text-align:left;padding:0.75rem;">"Rate"</th>
                        <th style="text-align:left;padding:0.75rem;">"Action"</th>
                    </tr>
                </thead>
                <tbody>
                    {legs.into_iter().map(|leg| {
                        let can_track = !leg.status_label.trim().is_empty();
                        let finance_leg = leg.clone();
                        let leg_id = leg.leg_id;
                        let confirm_finance_action = RwSignal::new(None::<String>);
                        view! {
                            <tr style="border-top:1px solid #f1f5f9;vertical-align:top;">
                                <td style="padding:0.75rem;">
                                    <strong>{leg.leg_code.clone()}</strong>
                                    {leg.carrier_label.clone().map(|carrier| view! { <div><small>{carrier}</small></div> })}
                                </td>
                                <td style="padding:0.75rem;">{leg.route_label.clone()}</td>
                                <td style="padding:0.75rem;">{format!("{} -> {}", leg.pickup_date_label, leg.delivery_date_label)}</td>
                                <td style="padding:0.75rem;display:grid;gap:0.35rem;">
                                    <span style=tone_style(&leg.status_tone)>{leg.status_label.clone()}</span>
                                    <small>{leg.bid_status_label.clone()}</small>
                                    {leg.payment_label.clone().map(|label| view! {
                                        <small style="color:#7c3aed;">{format!("Payment: {}", label)}</small>
                                    })}
                                </td>
                                <td style="padding:0.75rem;">{leg.amount_label.clone()}</td>
                                <td style="padding:0.75rem;display:grid;gap:0.4rem;min-width:180px;">
                                    {can_track.then(|| view! {
                                        <A href=format!("/execution/legs/{}", leg.leg_id) attr:style="color:#1d4ed8;text-decoration:none;">"Track leg"</A>
                                    })}
                                    {admin_mode.then(|| {
                                        leg.payments_href.clone().map(|href| view! {
                                            <A href=href attr:style="display:inline-block;padding:0.45rem 0.7rem;border-radius:0.7rem;background:#fff7dd;color:#92400e;text-decoration:none;">
                                                "Payments"
                                            </A>
                                        })
                                    })}
                                    {admin_mode.then(|| {
                                        leg.finance_action_label.clone().map(|label| {
                                            if leg.finance_action_enabled && can_manage_payments {
                                                view! {
                                                    <div style="display:grid;gap:0.35rem;">
                                                        <button
                                                            type="button"
                                                            style="padding:0.45rem 0.7rem;border:none;border-radius:0.7rem;background:#166534;color:white;cursor:pointer;"
                                                            disabled=move || finance_loading_leg_id.get() == Some(leg_id)
                                                            on:click=move |_| {
                                                                let Some(action_key) = finance_leg.finance_action_key.clone() else {
                                                                    return;
                                                                };
                                                                if confirm_finance_action.get() != Some(action_key.clone()) {
                                                                    confirm_finance_action.set(Some(action_key));
                                                                } else {
                                                                    run_finance_action(finance_leg.clone());
                                                                    confirm_finance_action.set(None);
                                                                }
                                                            }
                                                        >
                                                            {move || {
                                                                if finance_loading_leg_id.get() == Some(leg_id) {
                                                                    "Updating finance...".to_string()
                                                                } else if confirm_finance_action.get() == leg.finance_action_key {
                                                                    format!("Confirm {}", label.clone())
                                                                } else {
                                                                    label.clone()
                                                                }
                                                            }}
                                                        </button>
                                                        {move || {
                                                            if confirm_finance_action.get().is_some()
                                                                && finance_loading_leg_id.get() != Some(leg_id)
                                                            {
                                                                view! {
                                                                    <button
                                                                        type="button"
                                                                        style="padding:0.45rem 0.7rem;border:1px solid #d6d3d1;border-radius:0.7rem;background:#fafaf9;color:#111827;cursor:pointer;"
                                                                        on:click=move |_| confirm_finance_action.set(None)
                                                                    >
                                                                        "Cancel finance action"
                                                                    </button>
                                                                }.into_any()
                                                            } else {
                                                                view! { <></> }.into_any()
                                                            }
                                                        }}
                                                    </div>
                                                }.into_any()
                                            } else if leg.finance_action_enabled {
                                                view! {
                                                    <small style="color:#92400e;">"Payments permission required for direct finance actions."</small>
                                                }.into_any()
                                            } else {
                                                view! {
                                                    <small style="color:#64748b;">{label}</small>
                                                }.into_any()
                                            }
                                        })
                                    })}
                                    {(admin_mode && leg.payments_href.is_none() && leg.finance_action_label.is_none()).then(|| view! {
                                        <small style="color:#64748b;">"Finance controls will appear here when this leg reaches a payment milestone."</small>
                                    })}
                                </td>
                            </tr>
                        }
                    }).collect_view()}
                </tbody>
            </table>
        }.into_any()
    }
}

fn admin_profile_actions(
    load_id: u64,
    has_pending: bool,
    has_release_ready: bool,
    release_ready_leg_id: Option<u64>,
    review_note: RwSignal<String>,
    review_loading: RwSignal<bool>,
    finance_loading: RwSignal<bool>,
    action_message: RwSignal<Option<String>>,
    refresh_nonce: RwSignal<u64>,
) -> AnyView {
    if !has_pending && !has_release_ready {
        return view! { <></> }.into_any();
    }

    let run_review = move |decision: &'static str| {
        review_loading.set(true);
        let note = review_note.get();
        let remarks = if note.trim().is_empty() {
            None
        } else {
            Some(note.trim().to_string())
        };
        spawn_local(async move {
            match api::review_admin_load(
                load_id,
                &AdminReviewLoadRequest {
                    decision: decision.into(),
                    remarks,
                },
            )
            .await
            {
                Ok(response) => {
                    action_message.set(Some(response.message));
                    if response.success {
                        review_note.set(String::new());
                        refresh_nonce.update(|value| *value += 1);
                    }
                }
                Err(error) => action_message.set(Some(error)),
            }
            review_loading.set(false);
        });
    };
    let release_ready_leg_id_for_action = release_ready_leg_id;
    let run_release = move || {
        let Some(leg_id) = release_ready_leg_id_for_action else {
            action_message.set(Some(
                "No release-ready leg is available on this profile yet.".into(),
            ));
            return;
        };
        finance_loading.set(true);
        spawn_local(async move {
            match api::release_escrow(
                leg_id,
                &EscrowReleaseRequest {
                    transfer_id: None,
                    note: Some(format!(
                        "Triggered from the Rust admin load profile for load #{}.",
                        load_id
                    )),
                },
            )
            .await
            {
                Ok(response) => {
                    action_message.set(Some(response.message));
                    if response.success {
                        refresh_nonce.update(|value| *value += 1);
                    }
                }
                Err(error) => action_message.set(Some(error)),
            }
            finance_loading.set(false);
        });
    };

    view! {
        <section style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#fffbeb;display:grid;gap:0.85rem;">
            <div style="display:flex;justify-content:space-between;gap:1rem;align-items:flex-start;flex-wrap:wrap;">
                <div style="display:grid;gap:0.25rem;">
                    <strong>"Admin workflow controls"</strong>
                    <small style="color:#78716c;">
                        "This profile is open inside the admin shell, so review and finance shortcuts stay here instead of forcing a Blade fallback."
                    </small>
                </div>
                {has_release_ready.then(|| {
                    let href = release_ready_leg_id
                        .map(|leg_id| {
                            format!(
                                "/admin/payments?leg_id={}&action=release&source=admin-load-profile&load_id={}",
                                leg_id, load_id
                            )
                        })
                        .unwrap_or_else(|| "/admin/payments".into());
                    view! {
                        <div style="display:flex;gap:0.5rem;flex-wrap:wrap;">
                            <button
                                type="button"
                                on:click=move |_| run_release()
                                disabled=move || finance_loading.get()
                                style="padding:0.55rem 0.85rem;border:none;border-radius:0.75rem;background:#166534;color:white;cursor:pointer;"
                            >
                                {move || if finance_loading.get() { "Releasing..." } else { "Release Funds" }}
                            </button>
                            <A href=href attr:style="padding:0.55rem 0.85rem;border-radius:0.75rem;background:#e0f2fe;color:#0f172a;text-decoration:none;">
                                "Open Payments Console"
                            </A>
                        </div>
                    }
                })}
            </div>

            {has_pending.then(|| view! {
                <div style="display:grid;gap:0.6rem;padding-top:0.35rem;border-top:1px solid #e7e5e4;">
                    <small style="color:#57534e;">
                        "At least one leg is still pending approval. Approve, reject, or send back the load from this profile."
                    </small>
                    <textarea
                        rows="3"
                        prop:value=move || review_note.get()
                        on:input=move |ev| review_note.set(event_target_value(&ev))
                        placeholder="Remarks are required for reject or revision"
                        style="padding:0.7rem 0.8rem;border:1px solid #d6d3d1;border-radius:0.85rem;resize:vertical;"
                    />
                    <div style="display:flex;gap:0.5rem;flex-wrap:wrap;">
                        <button
                            type="button"
                            disabled=move || review_loading.get()
                            on:click=move |_| run_review("approve")
                            style="padding:0.55rem 0.8rem;border:none;border-radius:0.8rem;background:#166534;color:white;cursor:pointer;"
                        >
                            {move || if review_loading.get() { "Working..." } else { "Approve load" }}
                        </button>
                        <button
                            type="button"
                            disabled=move || review_loading.get()
                            on:click=move |_| run_review("revision")
                            style="padding:0.55rem 0.8rem;border:none;border-radius:0.8rem;background:#b45309;color:white;cursor:pointer;"
                        >
                            "Send back for revision"
                        </button>
                        <button
                            type="button"
                            disabled=move || review_loading.get()
                            on:click=move |_| run_review("reject")
                            style="padding:0.55rem 0.8rem;border:none;border-radius:0.8rem;background:#be123c;color:white;cursor:pointer;"
                        >
                            "Reject load"
                        </button>
                    </div>
                </div>
            })}

            {has_release_ready.then(|| view! {
                <div style="display:grid;gap:0.35rem;padding-top:0.35rem;border-top:1px solid #e7e5e4;">
                    <small style="color:#57534e;">
                        "This load has a release-ready leg, so admin can release funds here or open the payments console for exception handling."
                    </small>
                </div>
            })}
        </section>
    }
    .into_any()
}

fn render_admin_profile_summary(
    load_id: u64,
    legs: Vec<LoadProfileLegRow>,
    documents: Vec<LoadDocumentRow>,
    history: Vec<LoadHistoryRow>,
    handoff: Option<LoadHandoffSummary>,
) -> impl IntoView {
    let pending_count = legs.iter().filter(|leg| leg.status_code == 1).count();
    let release_ready_count = legs
        .iter()
        .filter(|leg| {
            leg.finance_action_key.as_deref() == Some("release") && leg.finance_action_enabled
        })
        .count();
    let execution_active_count = legs
        .iter()
        .filter(|leg| matches!(leg.status_code, 5 | 6 | 8 | 9))
        .count();
    let missing_blockchain_count = documents
        .iter()
        .filter(|document| document.can_verify_blockchain && document.blockchain_label.is_none())
        .count();
    let recent_history_count = history.len().min(5);
    let first_active_leg_id = legs
        .iter()
        .find(|leg| matches!(leg.status_code, 5 | 6 | 8 | 9))
        .map(|leg| leg.leg_id);
    let first_payments_href = legs.iter().find_map(|leg| leg.payments_href.clone());
    let first_finance_label = legs
        .iter()
        .find(|leg| leg.finance_action_enabled)
        .and_then(|leg| leg.finance_action_label.clone());
    let unassigned_leg_count = legs
        .iter()
        .filter(|leg| {
            leg.carrier_label
                .as_ref()
                .is_none_or(|label| label.trim().is_empty())
        })
        .count();
    let stloads_attention = handoff
        .as_ref()
        .filter(|handoff| matches!(handoff.status_tone.as_str(), "warning" | "danger"));
    let reconciliation_href = stloads_attention.map(|handoff| {
        if handoff.status_tone == "danger" {
            "/admin/stloads/reconciliation?action=mismatch_detected".to_string()
        } else {
            "/admin/stloads/reconciliation?action=auto_archive".to_string()
        }
    });
    let blocker_items = {
        let mut items = Vec::new();

        if pending_count > 0 {
            items.push(format!(
                "{} leg(s) still need review before this load is truly clear.",
                pending_count
            ));
        }

        if release_ready_count > 0 {
            items.push(format!(
                "{} leg(s) are release-ready and should move through finance next.",
                release_ready_count
            ));
        }

        if unassigned_leg_count > 0 {
            items.push(format!(
                "{} leg(s) still show no carrier assignment on the profile.",
                unassigned_leg_count
            ));
        }

        if missing_blockchain_count > 0 {
            items.push(format!(
                "{} document row(s) can still be anchored for blockchain follow-up.",
                missing_blockchain_count
            ));
        }

        if let Some(handoff) = stloads_attention {
            items.push(format!(
                "STLOADS handoff is signaling {} and should be checked in ops or reconciliation.",
                handoff.status_label
            ));
        }

        if items.is_empty() {
            items.push(
                "No major admin blockers stand out here; the profile reads clean from the Rust side."
                    .to_string(),
            );
        }

        items
    };

    let cards = vec![
        (
            "Pending legs",
            pending_count.to_string(),
            if pending_count > 0 {
                "warning"
            } else {
                "success"
            },
            "Legs that still need an admin review decision.",
        ),
        (
            "Release-ready legs",
            release_ready_count.to_string(),
            if release_ready_count > 0 {
                "success"
            } else {
                "info"
            },
            "Legs that can complete direct release in the Rust profile.",
        ),
        (
            "Execution-active",
            execution_active_count.to_string(),
            if execution_active_count > 0 {
                "primary"
            } else {
                "info"
            },
            "Legs still moving through pickup, transit, or delivery.",
        ),
        (
            "Docs awaiting anchor",
            missing_blockchain_count.to_string(),
            if missing_blockchain_count > 0 {
                "warning"
            } else {
                "success"
            },
            "Profile documents still eligible for blockchain follow-up.",
        ),
        (
            "Carrier gaps",
            unassigned_leg_count.to_string(),
            if unassigned_leg_count > 0 {
                "danger"
            } else {
                "success"
            },
            "Legs still missing a clear carrier assignment on this profile.",
        ),
        (
            "Recent history rows",
            recent_history_count.to_string(),
            "dark",
            "Latest visible status changes and operator context on this load.",
        ),
    ];

    view! {
        <section style="display:grid;gap:0.85rem;">
            <section style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:0.75rem;">
                {cards.into_iter().map(|(label, value, tone, note)| view! {
                    <div style="padding:0.9rem 1rem;border:1px solid #e5e7eb;border-radius:0.95rem;background:#ffffff;display:grid;gap:0.3rem;">
                        <div style="display:flex;justify-content:space-between;gap:0.6rem;align-items:center;flex-wrap:wrap;">
                            <strong>{label}</strong>
                            <span style=tone_style(tone)>{tone.replace('_', " ")}</span>
                        </div>
                        <div style="font-size:1.3rem;font-weight:700;color:#111827;">{value}</div>
                        <small style="color:#64748b;">{note}</small>
                    </div>
                }).collect_view()}
            </section>
            <section style="padding:0.85rem 1rem;border:1px solid #e5e7eb;border-radius:0.95rem;background:#fcfcfb;display:flex;justify-content:space-between;gap:0.75rem;align-items:center;flex-wrap:wrap;">
                <div style="display:grid;gap:0.2rem;">
                    <strong>"Admin next step"</strong>
                    <small style="color:#64748b;">
                        {if pending_count > 0 {
                            "Pending review still comes first on this load."
                        } else if release_ready_count > 0 {
                            "Finance release is the next likely admin action on this load."
                        } else if execution_active_count > 0 {
                            "Execution is still active, so tracking and delivery follow-up should stay in focus."
                        } else if missing_blockchain_count > 0 {
                            "Document anchor cleanup is the main remaining admin follow-up here."
                        } else {
                            "This load is relatively clean from an admin-overview perspective."
                        }}
                    </small>
                </div>
                <div style="display:flex;gap:0.5rem;flex-wrap:wrap;">
                    {first_active_leg_id.map(|leg_id| view! {
                        <A href=format!("/execution/legs/{}", leg_id) attr:style="padding:0.55rem 0.85rem;border-radius:0.75rem;background:#e8fff3;color:#166534;text-decoration:none;">
                            "Track active leg"
                        </A>
                    })}
                    {first_payments_href.clone().map(|href| view! {
                        <A href=href attr:style="padding:0.55rem 0.85rem;border-radius:0.75rem;background:#fff7dd;color:#92400e;text-decoration:none;">
                            "Open Payments"
                        </A>
                    })}
                    <A href="/admin/stloads/operations" attr:style="padding:0.55rem 0.85rem;border-radius:0.75rem;background:#eef2ff;color:#312e81;text-decoration:none;">
                        "STLOADS Ops"
                    </A>
                    {reconciliation_href.clone().map(|href| view! {
                        <A href=href attr:style="padding:0.55rem 0.85rem;border-radius:0.75rem;background:#ffe4e6;color:#be123c;text-decoration:none;">
                            "Reconciliation"
                        </A>
                    })}
                </div>
            </section>
            <section style="display:grid;grid-template-columns:repeat(auto-fit,minmax(280px,1fr));gap:0.85rem;">
                <section style="padding:0.85rem 1rem;border:1px solid #e5e7eb;border-radius:0.95rem;background:#ffffff;display:grid;gap:0.45rem;">
                    <strong>"Admin blocker checklist"</strong>
                    <ul style="margin:0;padding-left:1.1rem;display:grid;gap:0.3rem;color:#475569;">
                        {blocker_items.into_iter().map(|item| view! { <li>{item}</li> }).collect_view()}
                    </ul>
                </section>
                <section style="padding:0.85rem 1rem;border:1px solid #e5e7eb;border-radius:0.95rem;background:#ffffff;display:grid;gap:0.35rem;">
                    <strong>"Finance handoff"</strong>
                    <small style="color:#64748b;">
                        {first_finance_label
                            .map(|label| format!("The first direct finance cue on this profile is '{}'.", label))
                            .unwrap_or_else(|| "No direct finance shortcut is armed yet, so use the leg rows or the payments console when finance opens.".to_string())}
                    </small>
                    <small style="color:#64748b;">{format!("Admin load id: #{}", load_id)}</small>
                </section>
            </section>
        </section>
    }
}

fn render_admin_attention_lanes(
    load_id: u64,
    legs: Vec<LoadProfileLegRow>,
    documents: Vec<LoadDocumentRow>,
    handoff: Option<LoadHandoffSummary>,
) -> impl IntoView {
    let pending_review_count = legs.iter().filter(|leg| leg.status_code == 1).count();
    let execution_active_count = legs
        .iter()
        .filter(|leg| matches!(leg.status_code, 5 | 6 | 8 | 9))
        .count();
    let release_ready_count = legs
        .iter()
        .filter(|leg| {
            leg.finance_action_key.as_deref() == Some("release") && leg.finance_action_enabled
        })
        .count();
    let completed_count = legs.iter().filter(|leg| leg.status_code == 11).count();
    let visible_document_count = documents
        .iter()
        .filter(|document| document.can_view_file)
        .count();
    let editable_document_count = documents
        .iter()
        .filter(|document| document.can_edit)
        .count();
    let anchor_follow_up_count = documents
        .iter()
        .filter(|document| document.can_verify_blockchain && document.blockchain_label.is_none())
        .count();
    let first_execution_leg_id = legs
        .iter()
        .find(|leg| matches!(leg.status_code, 5 | 6 | 8 | 9))
        .map(|leg| leg.leg_id);
    let first_payment_href = legs.iter().find_map(|leg| leg.payments_href.clone());
    let handoff_attention_label = handoff
        .as_ref()
        .filter(|item| matches!(item.status_tone.as_str(), "warning" | "danger"))
        .map(|item| item.status_label.clone());
    let handoff_attention_href = handoff_attention_label.as_ref().map(|_| {
        if handoff
            .as_ref()
            .is_some_and(|item| item.status_tone == "danger")
        {
            "/admin/stloads/reconciliation?action=mismatch_detected".to_string()
        } else {
            "/admin/stloads/reconciliation?action=auto_archive".to_string()
        }
    });

    let lane_cards = vec![
        (
            "Review lane",
            pending_review_count.to_string(),
            if pending_review_count > 0 {
                "warning"
            } else {
                "success"
            },
            "Pending review legs should be cleared before execution and finance feel trustworthy.",
            Some("/admin/loads?tab=pending".to_string()),
            Some("Open pending loads".to_string()),
        ),
        (
            "Execution lane",
            execution_active_count.to_string(),
            if execution_active_count > 0 {
                "primary"
            } else {
                "info"
            },
            "Active legs belong in execution follow-up until pickup, transit, and delivery stabilize.",
            first_execution_leg_id.map(|leg_id| format!("/execution/legs/{}", leg_id)),
            first_execution_leg_id.map(|_| "Track active leg".to_string()),
        ),
        (
            "Finance lane",
            release_ready_count.to_string(),
            if release_ready_count > 0 {
                "success"
            } else {
                "info"
            },
            "Release-ready work should move into payments or closeout instead of staying buried in the profile.",
            first_payment_href.clone(),
            Some(if first_payment_href.is_some() {
                "Open payments".to_string()
            } else {
                "Payments opens here when finance activates".to_string()
            }),
        ),
        (
            "Document lane",
            visible_document_count.to_string(),
            if anchor_follow_up_count > 0 {
                "warning"
            } else {
                "success"
            },
            "Protected docs stay visible here for admin review, edits, and blockchain follow-up.",
            Some(format!("/admin/loads/{}", load_id)),
            Some("Stay on admin profile".to_string()),
        ),
    ];

    view! {
        <section style="display:grid;gap:0.85rem;">
            <section style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:0.75rem;">
                {lane_cards.into_iter().map(|(label, value, tone, note, href, action_label)| view! {
                    <div style="padding:0.95rem 1rem;border:1px solid #e5e7eb;border-radius:0.95rem;background:#ffffff;display:grid;gap:0.35rem;">
                        <div style="display:flex;justify-content:space-between;gap:0.6rem;align-items:center;flex-wrap:wrap;">
                            <strong>{label}</strong>
                            <span style=tone_style(tone)>{tone.replace('_', " ")}</span>
                        </div>
                        <div style="font-size:1.35rem;font-weight:700;color:#111827;">{value}</div>
                        <small style="color:#64748b;">{note}</small>
                        {href.zip(action_label).map(|(href, action_label)| view! {
                            <A href=href attr:style="justify-self:start;padding:0.45rem 0.75rem;border-radius:0.75rem;background:#f8fafc;color:#0f172a;text-decoration:none;">
                                {action_label}
                            </A>
                        })}
                    </div>
                }).collect_view()}
            </section>

            <section style="display:grid;grid-template-columns:repeat(auto-fit,minmax(280px,1fr));gap:0.85rem;">
                <section style="padding:0.9rem 1rem;border:1px solid #e5e7eb;border-radius:0.95rem;background:#fcfcfb;display:grid;gap:0.35rem;">
                    <strong>"Admin handoff guidance"</strong>
                    <small style="color:#64748b;">
                        {if pending_review_count > 0 {
                            "Review is still the lead workflow here, so clear those legs before finance and closeout start driving decisions."
                        } else if release_ready_count > 0 {
                            "This profile is now finance-first. Use payments or closeout while the release-ready lane is hot."
                        } else if execution_active_count > 0 {
                            "Execution is still the strongest operational lane, so stay close to tracking before you return here."
                        } else if completed_count > 0 {
                            "Completed legs are the likely handoff into closeout or collections now."
                        } else {
                            "No single lane is dominating this load right now; the Rust profile is mostly in clean-up mode."
                        }}
                    </small>
                    <div style="display:flex;gap:0.5rem;flex-wrap:wrap;">
                        <A href="/desk/facility" attr:style="padding:0.45rem 0.75rem;border-radius:0.75rem;background:#eef2ff;color:#312e81;text-decoration:none;">
                            "Facility desk"
                        </A>
                        <A href="/desk/closeout" attr:style="padding:0.45rem 0.75rem;border-radius:0.75rem;background:#fff7dd;color:#92400e;text-decoration:none;">
                            "Closeout desk"
                        </A>
                        <A href="/desk/collections" attr:style="padding:0.45rem 0.75rem;border-radius:0.75rem;background:#ffe4e6;color:#be123c;text-decoration:none;">
                            "Collections desk"
                        </A>
                    </div>
                </section>

                <section style="padding:0.9rem 1rem;border:1px solid #e5e7eb;border-radius:0.95rem;background:#ffffff;display:grid;gap:0.35rem;">
                    <strong>"Document oversight"</strong>
                    <small style="color:#64748b;">{format!(
                        "{} protected document(s) are visible here, {} remain editable, and {} still have blockchain follow-up open.",
                        visible_document_count,
                        editable_document_count,
                        anchor_follow_up_count
                    )}</small>
                    {handoff_attention_label.as_ref().map(|status_label| view! {
                        <small style="color:#92400e;">{format!(
                            "STLOADS is also flagging '{}' on this load, so document and ops review may need to happen together.",
                            status_label
                        )}</small>
                    })}
                    <div style="display:flex;gap:0.5rem;flex-wrap:wrap;">
                        <A href="/admin/stloads/operations" attr:style="padding:0.45rem 0.75rem;border-radius:0.75rem;background:#ecfeff;color:#155e75;text-decoration:none;">
                            "STLOADS ops"
                        </A>
                        {handoff_attention_href.map(|href| view! {
                            <A href=href attr:style="padding:0.45rem 0.75rem;border-radius:0.75rem;background:#fff7dd;color:#92400e;text-decoration:none;">
                                "Reconciliation"
                            </A>
                        })}
                    </div>
                </section>
            </section>
        </section>
    }
}

fn render_history(history: Vec<LoadHistoryRow>) -> AnyView {
    if history.is_empty() {
        view! { <p style="margin:0;">"No history entries are recorded for this load yet."</p> }
            .into_any()
    } else {
        view! {
            <table style="width:100%;border-collapse:collapse;min-width:720px;">
                <thead style="background:#f8fafc;">
                    <tr>
                        <th style="text-align:left;padding:0.75rem;">"When"</th>
                        <th style="text-align:left;padding:0.75rem;">"Status"</th>
                        <th style="text-align:left;padding:0.75rem;">"Actor"</th>
                        <th style="text-align:left;padding:0.75rem;">"Remarks"</th>
                    </tr>
                </thead>
                <tbody>
                    {history.into_iter().map(|entry| view! {
                        <tr style="border-top:1px solid #f1f5f9;vertical-align:top;">
                            <td style="padding:0.75rem;">{entry.created_at_label}</td>
                            <td style="padding:0.75rem;"><span style=tone_style(&entry.status_tone)>{entry.status_label}</span></td>
                            <td style="padding:0.75rem;">{entry.actor_label}</td>
                            <td style="padding:0.75rem;">{entry.remarks_label}</td>
                        </tr>
                    }).collect_view()}
                </tbody>
            </table>
        }.into_any()
    }
}

#[component]
pub fn LoadProfilePage(#[prop(optional)] admin_mode: bool) -> impl IntoView {
    let auth = use_auth();
    let params = use_params_map();
    let load_id = Memo::new(move |_| {
        params.with(|map| {
            map.get("load_id")
                .and_then(|value| value.parse::<u64>().ok())
        })
    });

    let screen = RwSignal::new(None::<LoadProfileScreen>);
    let is_loading = RwSignal::new(false);
    let error_message = RwSignal::new(None::<String>);
    let action_message = RwSignal::new(None::<String>);
    let refresh_nonce = RwSignal::new(0_u64);

    let upload_document_name = RwSignal::new(String::new());
    let upload_document_type = RwSignal::new("rate_confirmation".to_string());
    let is_uploading_document = RwSignal::new(false);

    let editing_document_id = RwSignal::new(None::<u64>);
    let document_name = RwSignal::new(String::new());
    let document_type = RwSignal::new("rate_confirmation".to_string());
    let file_path = RwSignal::new(String::new());
    let original_name = RwSignal::new(String::new());
    let mime_type = RwSignal::new(String::new());
    let file_size_input = RwSignal::new(String::new());
    let is_saving_document = RwSignal::new(false);
    let verifying_document_id = RwSignal::new(None::<u64>);
    let opening_document_id = RwSignal::new(None::<u64>);
    let admin_review_note = RwSignal::new(String::new());
    let admin_review_loading = RwSignal::new(false);
    let admin_finance_loading = RwSignal::new(false);
    let leg_finance_loading_id = RwSignal::new(None::<u64>);
    let can_manage_payments = Signal::derive(move || {
        session::has_permission(&auth, "access_admin_portal")
            || session::has_permission(&auth, "manage_payments")
    });

    Effect::new(move |_| {
        let ready = auth.session_ready.get();
        let current_session = auth.session.get();
        let load_id = load_id.get();
        let _refresh = refresh_nonce.get();

        if !ready {
            return;
        }

        let Some(load_id) = load_id else {
            screen.set(None);
            is_loading.set(false);
            error_message.set(Some(
                "The requested Rust load profile URL is missing a valid load id.".into(),
            ));
            return;
        };

        if !current_session.authenticated {
            screen.set(None);
            is_loading.set(false);
            error_message.set(Some("Sign in before opening a Rust load profile.".into()));
            return;
        }

        is_loading.set(true);
        let auth = auth.clone();

        spawn_local(async move {
            match api::fetch_load_profile_screen(load_id).await {
                Ok(next_screen) => {
                    error_message.set(None);
                    screen.set(Some(next_screen));
                }
                Err(error) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(
                            &auth,
                            "Your Rust session expired; sign in again.",
                        );
                    }
                    error_message.set(Some(error));
                }
            }

            is_loading.set(false);
        });
    });

    let clear_upload_form = move || {
        upload_document_name.set(String::new());
        upload_document_type.set("rate_confirmation".to_string());
    };

    let clear_document_form = move || {
        editing_document_id.set(None);
        document_name.set(String::new());
        document_type.set("rate_confirmation".to_string());
        file_path.set(String::new());
        original_name.set(String::new());
        mime_type.set(String::new());
        file_size_input.set(String::new());
    };

    let start_edit_document = move |document: LoadDocumentRow| {
        editing_document_id.set(Some(document.id));
        document_name.set(document.document_name);
        document_type.set(document.document_type_key);
        file_path.set(document.source_path);
        original_name.set(document.original_name.unwrap_or_default());
        mime_type.set(document.mime_type.unwrap_or_default());
        file_size_input.set(
            document
                .file_size_bytes
                .map(|value| value.to_string())
                .unwrap_or_default(),
        );
        action_message.set(Some(
            "Document row loaded into the Rust profile editor.".into(),
        ));
    };

    let upload_document = move || {
        let Some(current_screen) = screen.get() else {
            action_message.set(Some(
                "Load profile data is not ready yet, so the document upload could not start."
                    .into(),
            ));
            return;
        };

        let document_name_value = upload_document_name.get();
        let document_type_value = upload_document_type.get();
        if document_name_value.trim().is_empty() {
            action_message.set(Some(
                "Enter a document name before uploading a file.".into(),
            ));
            return;
        }
        if document_type_value.trim().is_empty() {
            action_message.set(Some(
                "Enter a document type before uploading a file.".into(),
            ));
            return;
        }

        let input_id = document_upload::upload_input_id(current_screen.load_id);
        is_uploading_document.set(true);
        action_message.set(None);
        let auth = auth.clone();

        spawn_local(async move {
            match document_upload::upload_load_document(
                current_screen.load_id,
                &document_name_value,
                &document_type_value,
                &input_id,
            )
            .await
            {
                Ok(response) => {
                    action_message.set(Some(response.message));
                    if response.success {
                        clear_upload_form();
                        clear_document_form();
                        refresh_nonce.update(|value| *value += 1);
                    }
                }
                Err(error) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(
                            &auth,
                            "Your Rust session expired; sign in again.",
                        );
                    }
                    action_message.set(Some(error));
                }
            }

            is_uploading_document.set(false);
        });
    };

    let save_document = move || {
        let Some(document_id) = editing_document_id.get() else {
            action_message.set(Some(
                "Choose an existing document row before saving metadata updates.".into(),
            ));
            return;
        };

        let payload = UpsertLoadDocumentRequest {
            document_name: document_name.get(),
            document_type: document_type.get(),
            file_path: file_path.get(),
            original_name: {
                let value = original_name.get();
                (!value.trim().is_empty()).then_some(value)
            },
            mime_type: {
                let value = mime_type.get();
                (!value.trim().is_empty()).then_some(value)
            },
            file_size: {
                let raw = file_size_input.get();
                if raw.trim().is_empty() {
                    None
                } else {
                    match raw.trim().parse::<i64>() {
                        Ok(value) => Some(value),
                        Err(_) => {
                            action_message
                                .set(Some("File size must be a whole number of bytes.".into()));
                            return;
                        }
                    }
                }
            },
        };

        is_saving_document.set(true);
        action_message.set(None);
        let auth = auth.clone();

        spawn_local(async move {
            match api::update_load_document(document_id, &payload).await {
                Ok(response) => {
                    action_message.set(Some(response.message));
                    if response.success {
                        clear_document_form();
                        refresh_nonce.update(|value| *value += 1);
                    }
                }
                Err(error) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(
                            &auth,
                            "Your Rust session expired; sign in again.",
                        );
                    }
                    action_message.set(Some(error));
                }
            }

            is_saving_document.set(false);
        });
    };

    let verify_document = move |document_id: u64| {
        verifying_document_id.set(Some(document_id));
        action_message.set(None);
        let auth = auth.clone();

        spawn_local(async move {
            match api::verify_load_document(
                document_id,
                &VerifyLoadDocumentRequest {
                    note: Some("Triggered from the Rust load profile.".into()),
                },
            )
            .await
            {
                Ok(response) => {
                    action_message.set(Some(response.message));
                    if response.success {
                        refresh_nonce.update(|value| *value += 1);
                    }
                }
                Err(error) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(
                            &auth,
                            "Your Rust session expired; sign in again.",
                        );
                    }
                    action_message.set(Some(error));
                }
            }

            verifying_document_id.set(None);
        });
    };

    let open_document = move |document_id: u64, download_path: String| {
        opening_document_id.set(Some(document_id));
        action_message.set(None);

        spawn_local(async move {
            match document_upload::open_protected_document(&download_path).await {
                Ok(()) => {
                    action_message.set(Some(
                        "Opening the protected document in a new browser tab.".into(),
                    ));
                }
                Err(error) => {
                    action_message.set(Some(error));
                }
            }

            opening_document_id.set(None);
        });
    };

    let download_document = move |document_id: u64, download_path: String, file_name: String| {
        opening_document_id.set(Some(document_id));
        action_message.set(None);

        spawn_local(async move {
            match document_upload::download_protected_document(&download_path, &file_name).await {
                Ok(()) => {
                    action_message.set(Some(
                        "Downloading the protected document from the Rust profile.".into(),
                    ));
                }
                Err(error) => {
                    action_message.set(Some(error));
                }
            }

            opening_document_id.set(None);
        });
    };

    let run_leg_finance_action = move |leg: LoadProfileLegRow| {
        let Some(action_key) = leg.finance_action_key.clone() else {
            action_message.set(Some(
                "No direct finance action is available for this leg yet.".into(),
            ));
            return;
        };

        leg_finance_loading_id.set(Some(leg.leg_id));
        action_message.set(None);

        spawn_local(async move {
            let note = Some(format!(
                "Triggered from the Rust admin load profile for leg {}.",
                leg.leg_code
            ));
            let result = match action_key.as_str() {
                "fund" => {
                    api::fund_escrow(
                        leg.leg_id,
                        &EscrowFundRequest {
                            amount_cents: None,
                            currency: Some("USD".into()),
                            platform_fee_cents: None,
                            payment_intent_id: None,
                            charge_id: None,
                            transfer_group: None,
                            note,
                        },
                    )
                    .await
                }
                "hold" => api::hold_escrow(leg.leg_id, &EscrowHoldRequest { note }).await,
                "release" => {
                    api::release_escrow(
                        leg.leg_id,
                        &EscrowReleaseRequest {
                            transfer_id: None,
                            note,
                        },
                    )
                    .await
                }
                other => Err(format!("Unsupported finance action '{}'.", other)),
            };

            match result {
                Ok(response) => {
                    action_message.set(Some(response.message));
                    if response.success {
                        refresh_nonce.update(|value| *value += 1);
                    }
                }
                Err(error) => action_message.set(Some(error)),
            }

            leg_finance_loading_id.set(None);
        });
    };

    let back_href = if admin_mode { "/admin/loads" } else { "/loads" };
    let profile_title = if admin_mode {
        "Admin Load Profile"
    } else {
        "Load Profile"
    };

    view! {
        <article style="display:grid;gap:1.25rem;">
            <section style="display:flex;justify-content:space-between;gap:1rem;align-items:flex-start;flex-wrap:wrap;">
                <div style="display:grid;gap:0.35rem;">
                    <h2>{move || screen.get().and_then(|value| value.load_number).unwrap_or_else(|| profile_title.into())}</h2>
                    <p>{move || screen.get().map(|value| value.subtitle).unwrap_or_else(|| "Rust load detail view".into())}</p>
                </div>
                <div style="display:flex;gap:0.75rem;flex-wrap:wrap;align-items:center;">
                    <A href=back_href attr:style="padding:0.7rem 1rem;border-radius:0.9rem;background:#f4f4f5;color:#111827;text-decoration:none;">{if admin_mode { "Back to admin loads" } else { "Back to loads" }}</A>
                    <A href=move || screen.get().map(|value| format!("/loads/{}/edit", value.load_id)).unwrap_or_else(|| "/loads/new".into()) attr:style="padding:0.7rem 1rem;border-radius:0.9rem;background:#fff7ed;color:#9a3412;text-decoration:none;">"Edit load"</A>
                    <A href="/loads/new" attr:style="padding:0.7rem 1rem;border-radius:0.9rem;background:#111827;color:white;text-decoration:none;">"Create another load"</A>
                </div>
            </section>

            {move || auth.session.get().user.map(|user| view! {
                <section style="padding:0.85rem 1rem;border:1px solid #dcfce7;border-radius:0.9rem;background:#f0fdf4;color:#166534;">
                    {format!("Authenticated as {} ({})", user.name, user.role_label)}
                </section>
            })}

            {move || action_message.get().map(|message| view! {
                <section style="padding:0.85rem 1rem;border:1px solid #dbeafe;border-radius:0.9rem;background:#eff6ff;color:#1d4ed8;">{message}</section>
            })}

            {move || error_message.get().map(|message| view! {
                <section style="padding:0.85rem 1rem;border:1px solid #fecaca;border-radius:0.9rem;background:#fff1f2;color:#be123c;">{message}</section>
            })}

            {move || {
                if is_loading.get() && screen.get().is_none() {
                    view! {
                        <section style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#fafaf9;">
                            "Loading Rust load profile data..."
                        </section>
                    }.into_any()
                } else if let Some(screen_value) = screen.get() {
                    let documents = screen_value.documents.clone();
                    let can_manage_documents = screen_value.can_manage_documents;
                    let upload_input_id = document_upload::upload_input_id(screen_value.load_id);
                    let has_pending_leg = screen_value.legs.iter().any(|leg| leg.status_code == 1);
                    let release_ready_leg_id = screen_value
                        .legs
                        .iter()
                        .find(|leg| leg.status_code == 10)
                        .map(|leg| leg.leg_id);
                    let has_release_ready_leg = release_ready_leg_id.is_some();
                    view! {
                        <>
                            <section style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:1rem;align-items:start;">
                                {screen_value.info_fields.into_iter().map(|field| view! {
                                    <div style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#fcfcfb;display:grid;gap:0.35rem;">
                                        <small style="color:#64748b;">{field.label}</small>
                                        <strong>{field.value}</strong>
                                    </div>
                                }).collect_view()}
                            </section>

                            <section style="display:grid;gap:1rem;grid-template-columns:minmax(0,1fr);">
                                {admin_mode.then(|| admin_profile_actions(
                                    screen_value.load_id,
                                    has_pending_leg,
                                    has_release_ready_leg,
                                    release_ready_leg_id,
                                    admin_review_note,
                                    admin_review_loading,
                                    admin_finance_loading,
                                    action_message,
                                    refresh_nonce,
                                ))}
                                {admin_mode.then(|| render_admin_profile_summary(
                                    screen_value.load_id,
                                    screen_value.legs.clone(),
                                    screen_value.documents.clone(),
                                    screen_value.history.clone(),
                                    screen_value.stloads_handoff.clone(),
                                ))}
                                {admin_mode.then(|| render_admin_attention_lanes(
                                    screen_value.load_id,
                                    screen_value.legs.clone(),
                                    screen_value.documents.clone(),
                                    screen_value.stloads_handoff.clone(),
                                ))}
                                {render_handoff(screen_value.stloads_handoff.clone(), admin_mode)}
                            </section>

                            <section style="display:grid;gap:1rem;grid-template-columns:repeat(auto-fit,minmax(320px,1fr));align-items:start;">
                                <section style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#ffffff;display:grid;gap:0.85rem;overflow:auto;">
                                    <strong>"Load legs"</strong>
                                    {render_legs(
                                        screen_value.legs.clone(),
                                        admin_mode,
                                        can_manage_payments.get(),
                                        leg_finance_loading_id,
                                        run_leg_finance_action,
                                    )}
                                </section>

                                <section style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#ffffff;display:grid;gap:1rem;overflow:auto;">
                                    <div style="display:flex;justify-content:space-between;gap:1rem;align-items:flex-start;flex-wrap:wrap;">
                                        <div style="display:grid;gap:0.35rem;">
                                            <strong>"Documents"</strong>
                                            <small style="color:#64748b;">"Uploads now store the binary through the Rust backend. Admins and the uploader profile can open the file, while metadata edits and blockchain follow-up stay on this screen."</small>
                                        </div>
                                        {can_manage_documents.then(|| view! {
                                            <button
                                                type="button"
                                                style="padding:0.55rem 0.85rem;border-radius:0.8rem;border:1px solid #d1d5db;background:#f8fafc;color:#111827;cursor:pointer;"
                                                on:click=move |_| { clear_upload_form(); clear_document_form(); }
                                            >
                                                "Reset forms"
                                            </button>
                                        })}
                                    </div>

                                    {can_manage_documents.then(|| view! {
                                        <form
                                            style="display:grid;gap:0.85rem;padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#fcfcfb;"
                                            on:submit=move |ev| {
                                                ev.prevent_default();
                                                upload_document();
                                            }
                                        >
                                            <div style="display:grid;gap:0.35rem;">
                                                <strong>"Upload a document"</strong>
                                                <small style="color:#64748b;">"The uploaded binary will only open for admin users and the profile that uploaded it."</small>
                                            </div>
                                            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:0.85rem;">
                                                <label style="display:grid;gap:0.35rem;">
                                                    <span>"Document name"</span>
                                                    <input type="text" prop:value=move || upload_document_name.get() on:input=move |ev| upload_document_name.set(event_target_value(&ev)) placeholder="Rate confirmation" />
                                                </label>
                                                <label style="display:grid;gap:0.35rem;">
                                                    <span>"Document type"</span>
                                                    <input type="text" prop:value=move || upload_document_type.get() on:input=move |ev| upload_document_type.set(event_target_value(&ev)) placeholder="rate_confirmation" />
                                                </label>
                                            </div>
                                            <label style="display:grid;gap:0.35rem;">
                                                <span>"Choose file"</span>
                                                <input id=upload_input_id.clone() type="file" />
                                            </label>
                                            <div style="display:flex;gap:0.75rem;flex-wrap:wrap;align-items:center;">
                                                <button type="submit" style="padding:0.65rem 0.95rem;border-radius:0.85rem;border:none;background:#111827;color:white;cursor:pointer;" disabled=move || is_uploading_document.get()>
                                                    {move || if is_uploading_document.get() { "Uploading..." } else { "Upload document" }}
                                                </button>
                                                <small style="color:#64748b;">"25 MB limit in the current Rust slice."</small>
                                            </div>
                                        </form>
                                    })}

                                    {can_manage_documents.then(|| view! {
                                        <form
                                            style="display:grid;gap:0.85rem;padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#f8fafc;"
                                            on:submit=move |ev| {
                                                ev.prevent_default();
                                                save_document();
                                            }
                                        >
                                            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:0.85rem;">
                                                <label style="display:grid;gap:0.35rem;">
                                                    <span>"Document name"</span>
                                                    <input type="text" prop:value=move || document_name.get() on:input=move |ev| document_name.set(event_target_value(&ev)) placeholder="Rate confirmation" />
                                                </label>
                                                <label style="display:grid;gap:0.35rem;">
                                                    <span>"Document type"</span>
                                                    <input type="text" prop:value=move || document_type.get() on:input=move |ev| document_type.set(event_target_value(&ev)) placeholder="rate_confirmation" />
                                                </label>
                                            </div>
                                            <label style="display:grid;gap:0.35rem;">
                                                <span>"Storage path or URL"</span>
                                                <input type="text" prop:value=move || file_path.get() on:input=move |ev| file_path.set(event_target_value(&ev)) placeholder="ibm-cos://bucket/load-docs/rate-confirmation.pdf" />
                                            </label>
                                            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:0.85rem;">
                                                <label style="display:grid;gap:0.35rem;">
                                                    <span>"Original file name"</span>
                                                    <input type="text" prop:value=move || original_name.get() on:input=move |ev| original_name.set(event_target_value(&ev)) placeholder="rate-confirmation.pdf" />
                                                </label>
                                                <label style="display:grid;gap:0.35rem;">
                                                    <span>"MIME type"</span>
                                                    <input type="text" prop:value=move || mime_type.get() on:input=move |ev| mime_type.set(event_target_value(&ev)) placeholder="application/pdf" />
                                                </label>
                                                <label style="display:grid;gap:0.35rem;">
                                                    <span>"File size (bytes)"</span>
                                                    <input type="number" min="0" step="1" prop:value=move || file_size_input.get() on:input=move |ev| file_size_input.set(event_target_value(&ev)) placeholder="1048576" />
                                                </label>
                                            </div>
                                            <div style="display:flex;gap:0.75rem;flex-wrap:wrap;align-items:center;">
                                                <button type="submit" style="padding:0.65rem 0.95rem;border-radius:0.85rem;border:none;background:#111827;color:white;cursor:pointer;" disabled=move || is_saving_document.get()>
                                                    {move || if is_saving_document.get() {
                                                        "Saving..."
                                                    } else {
                                                        "Save metadata"
                                                    }}
                                                </button>
                                                <small style="color:#64748b;">{move || editing_document_id.get().map(|id| format!("Editing document #{}", id)).unwrap_or_else(|| "Select a document row to edit its metadata".into())}</small>
                                            </div>
                                        </form>
                                    })}

                                    {documents.is_empty().then(|| view! {
                                        <p style="margin:0;">"No documents are attached yet. Upload the first one here to start the Rust-side document workflow."</p>
                                    })}

                                    {(!documents.is_empty()).then(|| view! {
                                        <table style="width:100%;border-collapse:collapse;min-width:720px;">
                                            <thead style="background:#f8fafc;">
                                                <tr>
                                                    <th style="text-align:left;padding:0.75rem;">"Name"</th>
                                                    <th style="text-align:left;padding:0.75rem;">"File"</th>
                                                    <th style="text-align:left;padding:0.75rem;">"Blockchain"</th>
                                                    <th style="text-align:left;padding:0.75rem;">"Uploaded"</th>
                                                    <th style="text-align:left;padding:0.75rem;">"Actions"</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                {documents.into_iter().map(|document| {
                                                    let file_meta = format!("{} | {}", document.mime_type.clone().unwrap_or_else(|| "unknown mime".into()), human_file_size(document.file_size_bytes));
                                                    let blockchain_badge = document.blockchain_label.clone().map(|label| {
                                                        let tone = document.blockchain_tone.clone().unwrap_or_else(|| "secondary".into());
                                                        view! { <span style=tone_style(&tone)>{label}</span> }.into_any()
                                                    });
                                                    let can_edit_row = document.can_edit;
                                                    let can_verify_row = document.can_verify_blockchain;
                                                    let can_view_row = document.can_view_file && document.download_path.is_some();
                                                    let edit_row = document.clone();
                                                    let document_id = document.id;
                                                    let download_path = document.download_path.clone();
                                                    let file_name = document
                                                        .original_name
                                                        .clone()
                                                        .unwrap_or_else(|| document.file_label.clone());
                                                    let can_preview_row = document.mime_type.clone().map(|mime| {
                                                        mime.starts_with("image/") || mime.eq_ignore_ascii_case("application/pdf")
                                                    }).unwrap_or(false);
                                                    view! {
                                                        <tr style="border-top:1px solid #f1f5f9;vertical-align:top;">
                                                            <td style="padding:0.75rem;display:grid;gap:0.3rem;">
                                                                <strong>{document.document_name}</strong>
                                                                <small>{document.document_type_label}</small>
                                                            </td>
                                                            <td style="padding:0.75rem;display:grid;gap:0.35rem;">
                                                                <strong>{document.file_label}</strong>
                                                                <small>{file_meta}</small>
                                                                <small style="color:#64748b;word-break:break-all;">{document.source_path}</small>
                                                                {document.uploaded_by_label.clone().map(|label| view! { <small style="color:#64748b;">{label}</small> })}
                                                            </td>
                                                            <td style="padding:0.75rem;display:grid;gap:0.35rem;">
                                                                {blockchain_badge.unwrap_or_else(|| view! { <span>"Not anchored yet"</span> }.into_any())}
                                                                {document.blockchain_hash_preview.clone().map(|preview| view! {
                                                                    <small style="color:#64748b;">{format!("Hash: {}", preview)}</small>
                                                                })}
                                                            </td>
                                                            <td style="padding:0.75rem;">{document.uploaded_at_label}</td>
                                                            <td style="padding:0.75rem;display:grid;gap:0.5rem;min-width:190px;">
                                                                {can_view_row.then(|| {
                                                                    let preview_path = download_path.clone().unwrap_or_default();
                                                                    let download_path = download_path.clone().unwrap_or_default();
                                                                    let file_name = file_name.clone();
                                                                    view! {
                                                                        <div style="display:grid;gap:0.35rem;">
                                                                            <button
                                                                                type="button"
                                                                                style="padding:0.55rem 0.8rem;border-radius:0.75rem;border:none;background:#1d4ed8;color:white;cursor:pointer;"
                                                                                disabled=move || opening_document_id.get() == Some(document_id)
                                                                                on:click=move |_| open_document(document_id, preview_path.clone())
                                                                            >
                                                                                {move || if opening_document_id.get() == Some(document_id) {
                                                                                    "Opening...".to_string()
                                                                                } else if can_preview_row {
                                                                                    "Preview".to_string()
                                                                                } else {
                                                                                    "View file".to_string()
                                                                                }}
                                                                            </button>
                                                                            <button
                                                                                type="button"
                                                                                style="padding:0.55rem 0.8rem;border-radius:0.75rem;border:1px solid #d1d5db;background:#f8fafc;color:#111827;cursor:pointer;"
                                                                                disabled=move || opening_document_id.get() == Some(document_id)
                                                                                on:click=move |_| download_document(document_id, download_path.clone(), file_name.clone())
                                                                            >
                                                                                {move || if opening_document_id.get() == Some(document_id) { "Preparing..." } else { "Download" }}
                                                                            </button>
                                                                        </div>
                                                                    }
                                                                })}
                                                                {can_edit_row.then(|| view! {
                                                                    <button
                                                                        type="button"
                                                                        style="padding:0.55rem 0.8rem;border-radius:0.75rem;border:1px solid #d1d5db;background:#f8fafc;color:#111827;cursor:pointer;"
                                                                        on:click=move |_| start_edit_document(edit_row.clone())
                                                                    >
                                                                        "Edit row"
                                                                    </button>
                                                                })}
                                                                {can_verify_row.then(|| view! {
                                                                    <button
                                                                        type="button"
                                                                        style="padding:0.55rem 0.8rem;border-radius:0.75rem;border:none;background:#0f766e;color:white;cursor:pointer;"
                                                                        disabled=move || verifying_document_id.get() == Some(document_id)
                                                                        on:click=move |_| verify_document(document_id)
                                                                    >
                                                                        {move || if verifying_document_id.get() == Some(document_id) { "Anchoring..." } else { "Anchor to blockchain" }}
                                                                    </button>
                                                                })}
                                                            </td>
                                                        </tr>
                                                    }
                                                }).collect_view()}
                                            </tbody>
                                        </table>
                                    })}
                                </section>
                            </section>

                            <section style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#ffffff;display:grid;gap:0.85rem;overflow:auto;">
                                <strong>"History"</strong>
                                {render_history(screen_value.history.clone())}
                            </section>

                            <section style="display:grid;gap:0.35rem;">
                                {screen_value.notes.into_iter().map(|note| view! { <p style="margin:0;">{note}</p> }).collect_view()}
                            </section>
                        </>
                    }.into_any()
                } else {
                    view! {
                        <section style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#fafaf9;">
                            "No Rust load profile data is available yet for this route."
                        </section>
                    }.into_any()
                }
            }}
        </article>
    }
}
