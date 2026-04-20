use leptos::{prelude::*, tachys::view::any_view::IntoAny, task::spawn_local};
use leptos_router::components::A;

use crate::{
    api,
    session::{self, use_auth},
};
use shared::{
    AdminLoadListScreen, AdminLoadRow, AdminReviewLoadRequest, EscrowFundRequest,
    EscrowHoldRequest, EscrowReleaseRequest,
};

use super::admin_guard_view;

fn tone_style(tone: &str) -> &'static str {
    match tone {
        "success" => "background:#e8fff3;padding:0.25rem 0.6rem;border-radius:999px;color:#0f766e;",
        "warning" => "background:#fff7dd;padding:0.25rem 0.6rem;border-radius:999px;color:#b45309;",
        "danger" => "background:#ffe4e6;padding:0.25rem 0.6rem;border-radius:999px;color:#be123c;",
        "info" => "background:#e0f2fe;padding:0.25rem 0.6rem;border-radius:999px;color:#0369a1;",
        "primary" => "background:#ede9fe;padding:0.25rem 0.6rem;border-radius:999px;color:#6d28d9;",
        "dark" => "background:#e5e7eb;padding:0.25rem 0.6rem;border-radius:999px;color:#111827;",
        _ => "background:#f1f5f9;padding:0.25rem 0.6rem;border-radius:999px;color:#475569;",
    }
}

#[component]
pub fn AdminLoadsPage() -> impl IntoView {
    let auth = use_auth();
    let screen = RwSignal::new(None::<AdminLoadListScreen>);
    let loading = RwSignal::new(false);
    let feedback = RwSignal::new(None::<String>);
    let active_tab = RwSignal::new(String::from("all"));
    let search_query = RwSignal::new(String::new());
    let refresh_nonce = RwSignal::new(0_u64);
    let action_loading_load_id = RwSignal::new(None::<u64>);
    let finance_loading_leg_id = RwSignal::new(None::<u64>);

    let can_view = Signal::derive(move || {
        session::has_permission(&auth, "access_admin_portal")
            || session::has_permission(&auth, "manage_loads")
    });
    let can_manage_payments = Signal::derive(move || {
        session::has_permission(&auth, "access_admin_portal")
            || session::has_permission(&auth, "manage_payments")
    });

    Effect::new(move |_| {
        let current_tab = active_tab.get();
        let _refresh = refresh_nonce.get();

        if !auth.session_ready.get() || !auth.session.get().authenticated || !can_view.get() {
            return;
        }

        loading.set(true);
        let auth = auth.clone();
        spawn_local(async move {
            match api::fetch_admin_load_list_screen(&current_tab).await {
                Ok(next) => {
                    screen.set(Some(next));
                    feedback.set(None);
                }
                Err(error) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(
                            &auth,
                            "Your Rust session expired; sign in again.",
                        );
                    }
                    feedback.set(Some(error));
                }
            }
            loading.set(false);
        });
    });

    view! {
        {move || {
            if let Some(guard) = admin_guard_view(&auth, "Admin Loads", &["access_admin_portal", "manage_loads"]) {
                guard
            } else {
                view! {
                    <article style="display:grid;gap:1rem;">
                        <section style="display:flex;justify-content:space-between;gap:1rem;align-items:flex-start;flex-wrap:wrap;">
                            <div>
                                <h2>"Admin Loads"</h2>
                                <p>"Rust mirror of the Laravel admin load list, with approval-stage, active, completion, and fund-release tabs."</p>
                            </div>
                            <div style="min-width:280px;">
                                <input
                                    type="text"
                                    placeholder="Search by load, owner, carrier, route, or status"
                                    prop:value=move || search_query.get()
                                    on:input=move |ev| search_query.set(event_target_value(&ev))
                                    style="width:100%;padding:0.75rem 0.85rem;border:1px solid #d6d3d1;border-radius:0.9rem;"
                                />
                            </div>
                        </section>

                        {move || feedback.get().map(|message| view! {
                            <section style="padding:0.85rem 1rem;border:1px solid #dbeafe;border-radius:0.9rem;background:#eff6ff;color:#1d4ed8;">
                                {message}
                            </section>
                        })}

                        <section style="display:flex;gap:0.6rem;flex-wrap:wrap;">
                            {move || screen.get().map(|screen_data| {
                                screen_data.tabs.into_iter().map(|tab| {
                                    let key = tab.key.clone();
                                    let active = tab.is_active;
                                    let style = if active {
                                        "padding:0.55rem 0.85rem;border-radius:999px;border:none;background:#111827;color:white;cursor:pointer;"
                                    } else {
                                        "padding:0.55rem 0.85rem;border-radius:999px;border:1px solid #d6d3d1;background:#fafaf9;color:#111827;cursor:pointer;"
                                    };
                                    view! {
                                        <button
                                            type="button"
                                            style=style
                                            on:click=move |_| active_tab.set(key.clone())
                                        >
                                            {format!("{} ({})", tab.label, tab.count)}
                                        </button>
                                    }
                                }).collect_view()
                            })}
                        </section>

                        {move || {
                            screen.get().map(|screen_data| {
                                let query = search_query.get().to_ascii_lowercase();
                                let rows = screen_data
                                    .rows
                                    .iter()
                                    .filter(|row| row_matches_query(row, &query))
                                    .cloned()
                                    .collect::<Vec<_>>();
                                let active_tab = screen_data.active_tab.clone();

                                view! {
                                    <section style="display:grid;gap:0.75rem;">
                                        {render_admin_load_attention_summary(active_tab.clone(), rows.clone())}
                                        <section style="padding:0.85rem 1rem;border:1px solid #e5e7eb;border-radius:0.95rem;background:#fcfcfb;display:grid;gap:0.25rem;">
                                            <strong>"Queue guidance"</strong>
                                            <small style="color:#64748b;">{admin_tab_guidance(&active_tab)}</small>
                                        </section>
                                    </section>
                                }
                            })
                        }}

                        <section style="overflow:auto;border:1px solid #e5e7eb;border-radius:1rem;background:#fff;">
                            <table style="width:100%;border-collapse:collapse;min-width:1120px;">
                                <thead style="background:#f8fafc;">
                                    <tr>
                                        <th style="text-align:left;padding:0.85rem;">"Load"</th>
                                        <th style="text-align:left;padding:0.85rem;">"Owner"</th>
                                        <th style="text-align:left;padding:0.85rem;">"Route"</th>
                                        <th style="text-align:left;padding:0.85rem;">"Dates"</th>
                                        <th style="text-align:left;padding:0.85rem;">"Status"</th>
                                        <th style="text-align:left;padding:0.85rem;">"Carrier"</th>
                                        <th style="text-align:left;padding:0.85rem;">"Amount"</th>
                                        <th style="text-align:left;padding:0.85rem;">"Actions"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {move || {
                                        if loading.get() && screen.get().is_none() {
                                            view! {
                                                <tr>
                                                    <td colspan="8" style="padding:1rem;">"Loading admin loads from the Rust backend..."</td>
                                                </tr>
                                            }.into_any()
                                        } else if let Some(screen_data) = screen.get() {
                                            let query = search_query.get().to_ascii_lowercase();
                                            let rows = screen_data.rows.into_iter().filter(|row| row_matches_query(row, &query)).collect::<Vec<_>>();
                                            if rows.is_empty() {
                                                view! {
                                                    <tr>
                                                        <td colspan="8" style="padding:1rem;">"No admin loads match the current tab or search query."</td>
                                                    </tr>
                                                }.into_any()
                                            } else {
                                                rows.into_iter().map(|row| {
                                                    render_admin_load_row(
                                                        row,
                                                        can_manage_payments.get(),
                                                        feedback,
                                                        refresh_nonce,
                                                        action_loading_load_id,
                                                        finance_loading_leg_id,
                                                    )
                                                }).collect_view().into_any()
                                            }
                                        } else {
                                            view! {
                                                <tr>
                                                    <td colspan="8" style="padding:1rem;">"No Rust admin load data is available yet."</td>
                                                </tr>
                                            }.into_any()
                                        }
                                    }}
                                </tbody>
                            </table>
                        </section>

                        <section style="display:grid;gap:0.35rem;">
                            {move || screen.get().map(|value| {
                                value.notes.into_iter().map(|note| view! {
                                    <p style="margin:0;">{note}</p>
                                }).collect_view()
                            })}
                        </section>
                    </article>
                }.into_any()
            }
        }}
    }
}

fn render_admin_load_row(
    row: AdminLoadRow,
    can_manage_payments: bool,
    feedback: RwSignal<Option<String>>,
    refresh_nonce: RwSignal<u64>,
    action_loading_load_id: RwSignal<Option<u64>>,
    finance_loading_leg_id: RwSignal<Option<u64>>,
) -> impl IntoView {
    let load_id = row.load_id;
    let leg_id = row.leg_id;
    let route_label = format!("{} -> {}", row.origin_label, row.destination_label);
    let date_label = format!("{} -> {}", row.pickup_date_label, row.delivery_date_label);
    let review_note = RwSignal::new(String::new());
    let confirm_finance_action = RwSignal::new(None::<String>);
    let finance_action_key = row.finance_action_key.clone();
    let finance_action_label = row.finance_action_label.clone();
    let finance_action_key_for_run = finance_action_key.clone();
    let finance_action_label_for_run = finance_action_label.clone();

    let run_review = move |decision: &'static str| {
        action_loading_load_id.set(Some(load_id));
        let remarks = {
            let value = review_note.get();
            let trimmed = value.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
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
                    feedback.set(Some(response.message));
                    if response.success {
                        review_note.set(String::new());
                        refresh_nonce.update(|value| *value += 1);
                    }
                }
                Err(error) => feedback.set(Some(error)),
            }
            action_loading_load_id.set(None);
        });
    };

    let run_finance_action = move || {
        let Some(action_key) = finance_action_key_for_run.clone() else {
            feedback.set(Some(
                "No direct finance action is available for this admin load row.".into(),
            ));
            return;
        };

        if confirm_finance_action.get() != Some(action_key.clone()) {
            confirm_finance_action.set(Some(action_key.clone()));
            feedback.set(Some(format!(
                "Confirm {} for load #{} to continue the Rust finance action.",
                finance_action_label_for_run
                    .clone()
                    .unwrap_or_else(|| action_key.replace('_', " ")),
                load_id
            )));
            return;
        }

        finance_loading_leg_id.set(Some(leg_id));
        spawn_local(async move {
            let note = Some(format!(
                "Triggered from the Rust admin loads page for load #{}.",
                load_id
            ));
            let result = match action_key.as_str() {
                "fund" => {
                    api::fund_escrow(
                        leg_id,
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
                "hold" => api::hold_escrow(leg_id, &EscrowHoldRequest { note }).await,
                "release" => {
                    api::release_escrow(
                        leg_id,
                        &EscrowReleaseRequest {
                            transfer_id: None,
                            note,
                        },
                    )
                    .await
                }
                other => Err(format!("Unsupported admin finance action '{}'.", other)),
            };

            match result {
                Ok(response) => {
                    feedback.set(Some(response.message));
                    if response.success {
                        confirm_finance_action.set(None);
                        refresh_nonce.update(|value| *value += 1);
                    }
                }
                Err(error) => feedback.set(Some(error)),
            }
            finance_loading_leg_id.set(None);
        });
    };

    view! {
        <tr style="border-top:1px solid #e5e7eb;vertical-align:top;">
            <td style="padding:0.85rem;display:grid;gap:0.25rem;">
                <A href=row.load_href.clone() attr:style="color:#1d4ed8;text-decoration:none;font-weight:600;">{row.leg_code}</A>
                <small>{format!("Load #{}", load_id)}</small>
                <small>{row.bid_status_label}</small>
            </td>
            <td style="padding:0.85rem;">{row.owner_label}</td>
            <td style="padding:0.85rem;">{route_label}</td>
            <td style="padding:0.85rem;">{date_label}</td>
            <td style="padding:0.85rem;display:grid;gap:0.35rem;">
                <span style=tone_style(&row.status_tone)>{row.status_label}</span>
                {row.payment_label.map(|payment| view! {
                    <small style="color:#475569;">{format!("Payment: {}", payment)}</small>
                })}
                {row.finance_note.map(|note| view! {
                    <small style="color:#7c3aed;">{note}</small>
                })}
            </td>
            <td style="padding:0.85rem;">{row.carrier_label.unwrap_or_else(|| "Unassigned".into())}</td>
            <td style="padding:0.85rem;">{row.amount_label}</td>
            <td style="padding:0.85rem;display:grid;gap:0.45rem;min-width:190px;">
                <A href=row.load_href attr:style="display:inline-block;padding:0.5rem 0.75rem;border-radius:0.75rem;background:#e0f2fe;color:#0f172a;text-decoration:none;">
                    "Profile"
                </A>
                {row.track_href.map(|href| view! {
                    <A href=href attr:style="display:inline-block;padding:0.5rem 0.75rem;border-radius:0.75rem;background:#e8fff3;color:#0f172a;text-decoration:none;">
                        "Track"
                    </A>
                })}
                {row.payments_href.map(|href| view! {
                    <A href=href attr:style="display:inline-block;padding:0.5rem 0.75rem;border-radius:0.75rem;background:#fff7dd;color:#0f172a;text-decoration:none;">
                        "Payments"
                    </A>
                })}
                {finance_action_label.as_ref().map(|label| {
                    let label = label.clone();
                    let finance_action_key = finance_action_key.clone();
                    if row.finance_action_enabled && can_manage_payments {
                        view! {
                            <div style="display:grid;gap:0.35rem;">
                                <button
                                    type="button"
                                    on:click=move |_| run_finance_action()
                                    disabled=move || finance_loading_leg_id.get() == Some(leg_id)
                                    style="padding:0.5rem 0.75rem;border:none;border-radius:0.75rem;background:#166534;color:white;cursor:pointer;"
                                >
                                    {move || {
                                        if finance_loading_leg_id.get() == Some(leg_id) {
                                            "Updating finance...".to_string()
                                        } else if confirm_finance_action.get() == finance_action_key {
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
                                                on:click=move |_| confirm_finance_action.set(None)
                                                style="padding:0.45rem 0.7rem;border:1px solid #d6d3d1;border-radius:0.75rem;background:#fafaf9;color:#111827;cursor:pointer;"
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
                    } else if row.finance_action_enabled {
                        view! {
                            <small style="color:#92400e;">"Payments permission required for direct payout actions."</small>
                        }.into_any()
                    } else {
                        view! {
                            <small style="color:#64748b;">{label}</small>
                        }.into_any()
                    }
                })}
                {row.primary_action_label.map(|label| view! {
                    <small style="color:#64748b;">{format!("Primary: {}", label)}</small>
                })}
                {row.can_review.then(|| view! {
                    <div style="display:grid;gap:0.45rem;padding-top:0.35rem;border-top:1px solid #e5e7eb;">
                        <textarea
                            rows="2"
                            prop:value=move || review_note.get()
                            on:input=move |ev| review_note.set(event_target_value(&ev))
                            placeholder="Remarks for reject or revision"
                            style="padding:0.55rem 0.65rem;border:1px solid #cbd5e1;border-radius:0.75rem;"
                        />
                        <div style="display:flex;gap:0.35rem;flex-wrap:wrap;">
                            <button
                                type="button"
                                disabled=move || action_loading_load_id.get() == Some(load_id)
                                on:click=move |_| run_review("approve")
                                style="padding:0.45rem 0.7rem;border:none;border-radius:0.75rem;background:#166534;color:white;cursor:pointer;"
                            >
                                {move || if action_loading_load_id.get() == Some(load_id) { "Working..." } else { "Approve" }}
                            </button>
                            <button
                                type="button"
                                disabled=move || action_loading_load_id.get() == Some(load_id)
                                on:click=move |_| run_review("revision")
                                style="padding:0.45rem 0.7rem;border:none;border-radius:0.75rem;background:#b45309;color:white;cursor:pointer;"
                            >
                                "Send Back"
                            </button>
                            <button
                                type="button"
                                disabled=move || action_loading_load_id.get() == Some(load_id)
                                on:click=move |_| run_review("reject")
                                style="padding:0.45rem 0.7rem;border:none;border-radius:0.75rem;background:#be123c;color:white;cursor:pointer;"
                            >
                                "Reject"
                            </button>
                        </div>
                    </div>
                })}
            </td>
        </tr>
    }
}

fn row_matches_query(row: &AdminLoadRow, query: &str) -> bool {
    if query.trim().is_empty() {
        return true;
    }

    [
        row.leg_code.as_str(),
        row.owner_label.as_str(),
        row.origin_label.as_str(),
        row.destination_label.as_str(),
        row.status_label.as_str(),
        row.bid_status_label.as_str(),
        row.carrier_label.as_deref().unwrap_or_default(),
        row.payment_label.as_deref().unwrap_or_default(),
    ]
    .into_iter()
    .any(|value| value.to_ascii_lowercase().contains(query))
}

fn render_admin_load_attention_summary(
    active_tab: String,
    rows: Vec<AdminLoadRow>,
) -> impl IntoView {
    let review_count = rows.iter().filter(|row| row.can_review).count();
    let release_count = rows
        .iter()
        .filter(|row| {
            row.finance_action_key.as_deref() == Some("release") && row.finance_action_enabled
        })
        .count();
    let execution_count = rows
        .iter()
        .filter(|row| matches!(row.status_code, 5 | 6 | 8 | 9))
        .count();
    let attention_count = rows
        .iter()
        .filter(|row| {
            row.can_review
                || row.finance_action_enabled
                || row
                    .finance_note
                    .as_ref()
                    .is_some_and(|note| !note.trim().is_empty())
        })
        .count();

    let summary_cards = vec![
        (
            "Visible rows",
            rows.len().to_string(),
            "dark",
            "Rows matching the current tab and search query.",
        ),
        (
            "Needs review",
            review_count.to_string(),
            if review_count > 0 {
                "warning"
            } else {
                "success"
            },
            "Pending approval, reject, or revision decisions in the current queue.",
        ),
        (
            "Release-ready",
            release_count.to_string(),
            if release_count > 0 { "success" } else { "info" },
            "Rows that can complete the direct Rust release flow right now.",
        ),
        (
            "Execution-active",
            execution_count.to_string(),
            if execution_count > 0 {
                "primary"
            } else {
                "info"
            },
            "Loads still moving through pickup, transit, or delivery execution stages.",
        ),
        (
            "Attention queue",
            attention_count.to_string(),
            if attention_count > 0 {
                "danger"
            } else {
                "success"
            },
            "Rows that still need finance, review, or operator follow-up.",
        ),
    ];

    view! {
        <section style="display:grid;grid-template-columns:repeat(auto-fit,minmax(190px,1fr));gap:0.75rem;">
            {summary_cards.into_iter().map(|(label, value, tone, note)| {
                let pill_label = if active_tab == "release-funds" && label == "Release-ready" {
                    "focus".to_string()
                } else {
                    tone.replace('_', " ")
                };
                view! {
                <div style="padding:0.9rem 1rem;border:1px solid #e5e7eb;border-radius:0.95rem;background:#ffffff;display:grid;gap:0.3rem;">
                    <div style="display:flex;justify-content:space-between;gap:0.6rem;align-items:center;flex-wrap:wrap;">
                        <strong>{label}</strong>
                        <span style=tone_style(tone)>{pill_label}</span>
                    </div>
                    <div style="font-size:1.3rem;font-weight:700;color:#111827;">{value}</div>
                    <small style="color:#64748b;">{note}</small>
                </div>
            }}).collect_view()}
        </section>
    }
}

fn admin_tab_guidance(active_tab: &str) -> &'static str {
    match active_tab {
        "pending" => {
            "Pending approval mirrors the old Blade action modal flow: review the profile, leave remarks when needed, then approve, reject, or send back for revision."
        }
        "approved" => {
            "Approved and active loads are the fastest route into profile review and live tracking. Use Track for moving freight and Profile for document or finance follow-up."
        }
        "completed" => {
            "Completed loads should now be checked for payout readiness, closeout cleanup, and any document gaps before they fully leave the operator queue."
        }
        "release-funds" => {
            "Fund Release is the closest Rust mirror of the old Blade release queue. Use the confirm-first release action for clean cases and fall back to Payments when the row notes call out an exception."
        }
        _ => {
            "All Loads is the admin oversight view. Start with the attention cards above, then drill into rows that still need review, finance, or tracking follow-up."
        }
    }
}
