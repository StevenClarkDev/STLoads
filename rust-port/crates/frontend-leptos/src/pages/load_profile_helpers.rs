use crate::api;
use leptos::{
    prelude::*,
    tachys::view::any_view::{AnyView, IntoAny},
    task::spawn_local,
};
use leptos_router::components::A;
use shared::{
    AdminReviewLoadRequest, EscrowReleaseRequest, LoadDocumentRow, LoadHandoffSummary,
    LoadHistoryRow, LoadProfileLegRow,
};
pub(super) fn tone_style(tone: &str) -> &'static str {
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

pub(super) fn render_handoff(handoff: Option<LoadHandoffSummary>, admin_mode: bool) -> AnyView {
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

pub(super) fn render_legs(
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
                                                                ().into_any()
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

pub(super) fn admin_profile_actions(
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
        return ().into_any();
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
                    idempotency_key: None,
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

pub(super) fn render_admin_profile_summary(
    _load_id: u64,
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
    let _first_finance_label = legs
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
    let _blocker_items = {
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
                "{} document row(s) can still be verified with a content hash.",
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
            "Docs awaiting hash",
            missing_blockchain_count.to_string(),
            if missing_blockchain_count > 0 {
                "warning"
            } else {
                "success"
            },
            "Profile documents still eligible for SHA-256 hash verification.",
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
                {cards.into_iter().map(|(label, value, tone, _note)| view! {
                    <div style="padding:0.9rem 1rem;border:1px solid #e5e7eb;border-radius:0.95rem;background:#ffffff;display:grid;gap:0.3rem;">
                        <div style="display:flex;justify-content:space-between;gap:0.6rem;align-items:center;flex-wrap:wrap;">
                            <strong>{label}</strong>
                            <span style=tone_style(tone)>{tone.replace('_', " ")}</span>
                        </div>
                        <div style="font-size:1.3rem;font-weight:700;color:#111827;">{value}</div>
                    </div>
                }).collect_view()}
            </section>
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
    }
}

pub(super) fn render_admin_attention_lanes(
    _load_id: u64,
    _legs: Vec<LoadProfileLegRow>,
    _documents: Vec<LoadDocumentRow>,
    _handoff: Option<LoadHandoffSummary>,
) -> impl IntoView {
    view! { <></> }
}

pub(super) fn render_history(history: Vec<LoadHistoryRow>) -> AnyView {
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
