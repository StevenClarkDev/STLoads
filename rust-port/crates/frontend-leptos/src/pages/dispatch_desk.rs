use futures_util::future::AbortHandle;
use leptos::{prelude::*, tachys::view::any_view::IntoAny, task::spawn_local};
use leptos_router::{components::A, hooks::use_params_map};

use crate::{
    api, realtime,
    session::{self, use_auth},
};
use shared::{
    DispatchDeskFollowUpRequest, DispatchDeskRow, DispatchDeskScreen, EscrowFundRequest,
    EscrowHoldRequest, EscrowReleaseRequest, RealtimeEventKind, RealtimeTopic,
};

fn tone_style(tone: &str) -> &'static str {
    match tone {
        "success" => {
            "background:#e8fff3;padding:0.22rem 0.55rem;border-radius:999px;color:#0f766e;"
        }
        "warning" => {
            "background:#fff7dd;padding:0.22rem 0.55rem;border-radius:999px;color:#b45309;"
        }
        "danger" => "background:#ffe4e6;padding:0.22rem 0.55rem;border-radius:999px;color:#be123c;",
        "info" => "background:#e0f2fe;padding:0.22rem 0.55rem;border-radius:999px;color:#0369a1;",
        "primary" => {
            "background:#ede9fe;padding:0.22rem 0.55rem;border-radius:999px;color:#6d28d9;"
        }
        "dark" => "background:#e5e7eb;padding:0.22rem 0.55rem;border-radius:999px;color:#111827;",
        _ => "background:#f1f5f9;padding:0.22rem 0.55rem;border-radius:999px;color:#475569;",
    }
}

#[component]
pub fn DispatchDeskPage() -> impl IntoView {
    let auth = use_auth();
    let params = use_params_map();
    let screen = RwSignal::new(None::<DispatchDeskScreen>);
    let loading = RwSignal::new(false);
    let feedback = RwSignal::new(None::<String>);
    let refresh_nonce = RwSignal::new(0_u64);
    let ws_connected = RwSignal::new(false);
    let ws_handle = RwSignal::new(None::<AbortHandle>);
    let pending_handoff_id = RwSignal::new(None::<u64>);
    let pending_finance_leg_id = RwSignal::new(None::<u64>);
    let pending_note_leg_id = RwSignal::new(None::<u64>);

    let active_desk = Signal::derive(move || {
        params
            .with(|params| params.get("desk_key"))
            .unwrap_or_else(|| "quote".into())
    });

    let can_view = Signal::derive(move || {
        session::has_permission(&auth, "manage_dispatch_desk")
            || session::has_permission(&auth, "manage_loads")
            || session::has_permission(&auth, "access_admin_portal")
    });

    let can_manage_finance = Signal::derive(move || {
        session::has_permission(&auth, "manage_payments")
            || session::has_permission(&auth, "access_admin_portal")
    });

    Effect::new(move |_| {
        let desk_key = active_desk.get();
        let _refresh = refresh_nonce.get();

        if !auth.session_ready.get() {
            return;
        }

        if !auth.session.get().authenticated {
            screen.set(None);
            loading.set(false);
            feedback.set(Some("Sign in to view the Rust dispatch desk.".into()));
            return;
        }

        if !can_view.get() {
            screen.set(None);
            loading.set(false);
            feedback.set(Some(
                "The authenticated session does not have dispatch desk access in the Rust port."
                    .into(),
            ));
            return;
        }

        loading.set(true);
        let auth = auth.clone();
        spawn_local(async move {
            match api::fetch_dispatch_desk_screen(&desk_key).await {
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

    let run_handoff_action = move |handoff_id: u64, action_key: String| {
        pending_handoff_id.set(Some(handoff_id));
        let auth = auth.clone();
        spawn_local(async move {
            match api::run_dispatch_desk_handoff_action(handoff_id, &action_key).await {
                Ok(response) => {
                    feedback.set(Some(response.message));
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
                    feedback.set(Some(error));
                }
            }
            pending_handoff_id.set(None);
        });
    };

    let submit_follow_up = move |leg_id: u64, desk_key: String, note: String| {
        pending_note_leg_id.set(Some(leg_id));
        let auth = auth.clone();
        spawn_local(async move {
            match api::add_dispatch_desk_follow_up(
                leg_id,
                &DispatchDeskFollowUpRequest { desk_key, note },
            )
            .await
            {
                Ok(response) => {
                    feedback.set(Some(response.message));
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
                    feedback.set(Some(error));
                }
            }
            pending_note_leg_id.set(None);
        });
    };

    let run_finance_action = move |leg_id: u64, action_key: String, desk_key: String| {
        pending_finance_leg_id.set(Some(leg_id));
        let auth = auth.clone();
        spawn_local(async move {
            let note = format!("Triggered from the Rust {} desk.", desk_key);
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
                            note: Some(note),
                        },
                    )
                    .await
                }
                "hold" => api::hold_escrow(leg_id, &EscrowHoldRequest { note: Some(note) }).await,
                "release" => {
                    api::release_escrow(
                        leg_id,
                        &EscrowReleaseRequest {
                            transfer_id: None,
                            note: Some(note),
                        },
                    )
                    .await
                }
                other => Err(format!("Unsupported desk finance action '{}'.", other)),
            };

            match result {
                Ok(response) => {
                    feedback.set(Some(response.message));
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
                    feedback.set(Some(error));
                }
            }
            pending_finance_leg_id.set(None);
        });
    };

    Effect::new(move |_| {
        let current_session = auth.session.get();
        if !auth.session_ready.get() || !current_session.authenticated || !can_view.get() {
            if let Some(existing_handle) = ws_handle.get_untracked() {
                existing_handle.abort();
                ws_handle.set(None);
            }
            ws_connected.set(false);
            return;
        }

        let current_user_id = current_session.user.as_ref().map(|user| user.id);
        let auth = auth.clone();

        if let Some(existing_handle) = ws_handle.get_untracked() {
            existing_handle.abort();
        }

        let handle = realtime::connect_realtime_listener(
            None,
            vec![
                RealtimeTopic::LoadBoard,
                RealtimeTopic::AdminTmsOperations,
                RealtimeTopic::AdminTmsReconciliation,
                RealtimeTopic::AdminPayments,
            ],
            move |event| match event.kind {
                RealtimeEventKind::LoadLegBooked
                | RealtimeEventKind::LegExecutionUpdated
                | RealtimeEventKind::PaymentsOperationsUpdated
                | RealtimeEventKind::TmsOperationsUpdated
                | RealtimeEventKind::TmsReconciliationUpdated => {
                    refresh_nonce.update(|value| *value += 1);
                    feedback.set(Some(format!("Realtime update: {}", event.summary)));
                }
                RealtimeEventKind::SessionInvalidated => {
                    if event.actor_user_id == current_user_id {
                        if let Some(existing_handle) = ws_handle.get_untracked() {
                            existing_handle.abort();
                            ws_handle.set(None);
                        }
                        session::invalidate_session(
                            &auth,
                            "The current Rust session was invalidated; sign in again.",
                        );
                        ws_connected.set(false);
                    }
                }
                _ => {}
            },
        );

        ws_connected.set(handle.is_some());
        ws_handle.set(handle);
    });

    view! {
        <article style="display:grid;gap:1.15rem;">
            <section style="display:flex;justify-content:space-between;gap:1rem;align-items:flex-start;flex-wrap:wrap;">
                <div style="display:grid;gap:0.35rem;">
                    <p style=tone_style("info")>"Dispatch Operations"</p>
                    <h2>{move || screen.get().map(|value| value.title).unwrap_or_else(|| "Dispatch Desk".into())}</h2>
                </div>
                <span style=tone_style(if ws_connected.get() { "success" } else { "secondary" })>
                    {move || if ws_connected.get() { "Realtime connected" } else { "Realtime reconnecting" }}
                </span>
            </section>

            {move || feedback.get().map(|message| view! {
                <section style="padding:0.85rem 1rem;border:1px solid #dbeafe;border-radius:0.9rem;background:#eff6ff;color:#1d4ed8;">
                    {message}
                </section>
            })}

            <section style="display:flex;gap:0.75rem;flex-wrap:wrap;">
                {move || {
                    screen
                        .get()
                        .map(|value| {
                            value
                                .desks
                                .into_iter()
                                .map(|desk| {
                                    let style = if desk.is_active {
                                        "display:inline-flex;align-items:center;padding:0.55rem 0.85rem;border-radius:999px;background:#111827;color:white;text-decoration:none;"
                                    } else {
                                        "display:inline-flex;align-items:center;padding:0.55rem 0.85rem;border-radius:999px;background:#f4f4f5;color:#111827;text-decoration:none;"
                                    };
                                    view! { <A href=desk.href attr:style=style>{desk.label}</A> }
                                })
                                .collect_view()
                        })
                }}
            </section>

            {move || {
                screen.get().map(|value| {
                    (!value.quick_links.is_empty()).then(|| {
                        value
                            .quick_links
                            .into_iter()
                            .map(|link| {
                                view! {
                                    <A
                                        href=link.href
                                        attr:style="display:inline-flex;align-items:center;padding:0.55rem 0.85rem;border-radius:999px;background:#eef2ff;color:#312e81;text-decoration:none;margin-right:0.6rem;"
                                    >
                                        {link.label}
                                    </A>
                                }
                            })
                            .collect_view()
                    })
                })
            }}

            <section style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:1rem;">
                {move || {
                    screen
                        .get()
                        .map(|value| {
                            value
                                .status_cards
                                .into_iter()
                                .map(|card| {
                                    view! {
                                        <div style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#fcfcfb;display:grid;gap:0.35rem;">
                                            <span style=tone_style(&card.tone)>{card.label}</span>
                                            <strong style="font-size:1.35rem;">{card.value.to_string()}</strong>
                                            {card.note.map(|note| view! { <p style="margin:0;">{note}</p> })}
                                        </div>
                                    }
                                })
                                .collect_view()
                        })
                }}
            </section>

            <section style="overflow:auto;border:1px solid #e5e7eb;border-radius:1rem;">
                <table style="width:100%;border-collapse:collapse;min-width:980px;">
                    <thead style="background:#f8fafc;">
                        <tr>
                            <th style="text-align:left;padding:0.9rem;">"Load #"</th>
                            <th style="text-align:left;padding:0.9rem;">"Title"</th>
                            <th style="text-align:left;padding:0.9rem;">{move || third_column_label(&active_desk.get())}</th>
                            <th style="text-align:left;padding:0.9rem;">"Leg Status"</th>
                            <th style="text-align:left;padding:0.9rem;">"STLOADS"</th>
                            <th style="text-align:left;padding:0.9rem;">{move || focus_column_label(&active_desk.get())}</th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || {
                            if loading.get() && screen.get().is_none() {
                                view! {
                                    <tr>
                                        <td colspan="6" style="padding:1rem;">"Loading dispatch desk data from the Rust backend..."</td>
                                    </tr>
                                }.into_any()
                            } else {
                                screen
                                    .get()
                                    .map(|value| {
                                        if value.rows.is_empty() {
                                            view! {
                                                <tr>
                                                    <td colspan="6" style="padding:1rem;">"No rows match this dispatch-desk stage yet."</td>
                                                </tr>
                                            }
                                            .into_any()
                                        } else {
                                            let desk_key = value.desk_key.clone();
                                            value
                                                .rows
                                                .into_iter()
                                                .map(|row| {
                                                    render_row(
                                                        &desk_key,
                                                        row,
                                                        can_manage_finance.get(),
                                                        pending_handoff_id,
                                                        pending_finance_leg_id,
                                                        pending_note_leg_id,
                                                        run_handoff_action,
                                                        run_finance_action,
                                                        submit_follow_up,
                                                    )
                                                })
                                                .collect_view()
                                                .into_any()
                                        }
                                    })
                                    .unwrap_or_else(|| view! {
                                        <tr>
                                            <td colspan="6" style="padding:1rem;">"No dispatch desk data is available yet."</td>
                                        </tr>
                                    }.into_any())
                            }
                        }}
                    </tbody>
                </table>
            </section>

        </article>
    }
}

fn third_column_label(desk_key: &str) -> &'static str {
    match desk_key {
        "quote" => "Equipment / Weight",
        "tender" | "facility" => "Carrier",
        "closeout" | "collections" => "Finance Status",
        _ => "Workflow Context",
    }
}

fn focus_column_label(desk_key: &str) -> &'static str {
    match desk_key {
        "quote" => "Board Eligibility",
        "tender" => "Duplicate Risk",
        "facility" => "Readiness",
        "closeout" => "Action Needed",
        "collections" => "Archive Status",
        _ => "Desk Signal",
    }
}

fn render_row<F>(
    desk_key: &str,
    row: DispatchDeskRow,
    can_manage_finance: bool,
    pending_handoff_id: RwSignal<Option<u64>>,
    pending_finance_leg_id: RwSignal<Option<u64>>,
    pending_note_leg_id: RwSignal<Option<u64>>,
    run_handoff_action: F,
    run_finance_action: impl Fn(u64, String, String) + Copy + 'static,
    submit_follow_up: impl Fn(u64, String, String) + Copy + 'static,
) -> impl IntoView
where
    F: Fn(u64, String) + Copy + 'static,
{
    let DispatchDeskRow {
        load_id,
        leg_id,
        handoff_id,
        load_number,
        title,
        equipment_label,
        weight_label,
        carrier_label,
        payment_label,
        leg_status_label,
        leg_status_tone,
        stloads_label,
        stloads_tone,
        focus_label,
        focus_tone,
        focus_note,
        archive_guidance_label,
        archive_guidance_tone,
        archive_guidance_note,
        latest_activity_note,
        load_href,
        primary_action_key,
        primary_action_label,
        primary_action_enabled,
        finance_action_key,
        finance_action_label,
        finance_action_enabled,
        secondary_action_label,
        secondary_action_href,
        ..
    } = row;
    let desk_key_value = desk_key.to_string();
    let follow_up_note = RwSignal::new(String::new());

    let third_column = match desk_key {
        "quote" => format!(
            "{}{}",
            equipment_label.unwrap_or_else(|| "-".into()),
            weight_label
                .map(|value| format!(" | {}", value))
                .unwrap_or_default()
        ),
        "tender" | "facility" => carrier_label.unwrap_or_else(|| "Unassigned".into()),
        "closeout" | "collections" => {
            payment_label.unwrap_or_else(|| "Finance review needed".into())
        }
        _ => "-".into(),
    };
    let resolved_load_number = load_number.unwrap_or_else(|| format!("Load {}", load_id));
    let resolved_stloads_tone = stloads_tone.unwrap_or_else(|| "secondary".into());
    let load_number_view = if let Some(href) = load_href {
        view! { <A href=href>{resolved_load_number.clone()}</A> }.into_any()
    } else {
        view! { <span>{resolved_load_number.clone()}</span> }.into_any()
    };
    let stloads_view = if let Some(label) = stloads_label {
        view! { <span style=tone_style(&resolved_stloads_tone)>{label}</span> }.into_any()
    } else {
        view! { <span style="color:#64748b;">{"-"}</span> }.into_any()
    };
    let action_view = match (handoff_id, primary_action_key, primary_action_label) {
        (Some(handoff_id), Some(action_key), Some(action_label)) if primary_action_enabled => {
            let action_label_text = action_label.clone();
            view! {
                <button
                    type="button"
                    disabled=move || pending_handoff_id.get() == Some(handoff_id)
                    on:click=move |_| run_handoff_action(handoff_id, action_key.clone())
                    style="padding:0.5rem 0.75rem;border:none;border-radius:0.75rem;background:#111827;color:white;cursor:pointer;"
                >
                    {move || if pending_handoff_id.get() == Some(handoff_id) { "Working...".to_string() } else { action_label_text.clone() }}
                </button>
            }
            .into_any()
        }
        _ => view! { <span style="color:#64748b;">"Open profile for details"</span> }.into_any(),
    };
    let secondary_action_view = match (secondary_action_label, secondary_action_href) {
        (Some(label), Some(href)) => view! {
            <A
                href=href
                attr:style="display:inline-flex;align-items:center;justify-content:center;padding:0.5rem 0.75rem;border-radius:0.75rem;border:1px solid #d1d5db;background:#f8fafc;color:#111827;text-decoration:none;"
            >
                {label}
            </A>
        }
        .into_any(),
        _ => view! { <></> }.into_any(),
    };
    let finance_action_view =
        match (finance_action_key, finance_action_label, finance_action_enabled) {
            (Some(action_key), Some(action_label), true) if can_manage_finance => {
                let action_label_text = action_label.clone();
                let desk_action_key = desk_key_value.clone();
                view! {
                    <button
                        type="button"
                        disabled=move || pending_finance_leg_id.get() == Some(leg_id)
                        on:click=move |_| {
                            run_finance_action(
                                leg_id,
                                action_key.clone(),
                                desk_action_key.clone(),
                            )
                        }
                        style="padding:0.5rem 0.75rem;border:1px solid #0f766e;border-radius:0.75rem;background:#ecfdf5;color:#0f766e;cursor:pointer;"
                    >
                        {move || {
                            if pending_finance_leg_id.get() == Some(leg_id) {
                                "Updating finance...".to_string()
                            } else {
                                action_label_text.clone()
                            }
                        }}
                    </button>
                }
                .into_any()
            }
            (Some(_), Some(label), false) => view! {
                <span style="display:inline-flex;align-items:center;padding:0.45rem 0.7rem;border-radius:0.75rem;background:#f1f5f9;color:#475569;">
                    {label}
                </span>
            }
            .into_any(),
            (Some(_), Some(_), true) => view! {
                <small style="color:#92400e;">"Payments permission required for direct finance actions."</small>
            }
            .into_any(),
            _ => view! { <></> }.into_any(),
        };

    view! {
        <tr style="border-top:1px solid #e5e7eb;">
            <td style="padding:0.9rem;">{load_number_view}</td>
            <td style="padding:0.9rem;">{title}</td>
            <td style="padding:0.9rem;">{third_column}</td>
            <td style="padding:0.9rem;">
                <span style=tone_style(&leg_status_tone)>{leg_status_label}</span>
            </td>
            <td style="padding:0.9rem;">{stloads_view}</td>
            <td style="padding:0.9rem;">
                <div style="display:grid;gap:0.35rem;">
                    <span style=tone_style(&focus_tone)>{focus_label}</span>
                    {focus_note.map(|note| view! { <small style="color:#475569;">{note}</small> })}
                    {archive_guidance_label.map(|label| {
                        let tone = archive_guidance_tone.clone().unwrap_or_else(|| "secondary".into());
                        view! { <span style=tone_style(&tone)>{label}</span> }
                    })}
                    {archive_guidance_note.map(|note| view! { <small style="color:#7c3aed;">{note}</small> })}
                    {latest_activity_note.map(|note| view! {
                        <small style="color:#0f172a;font-style:italic;">{note}</small>
                    })}
                    <div style="display:flex;gap:0.45rem;flex-wrap:wrap;">
                        {action_view}
                        {finance_action_view}
                        {secondary_action_view}
                    </div>
                    <div style="display:grid;gap:0.35rem;padding-top:0.35rem;">
                        <textarea
                            rows="2"
                            prop:value=move || follow_up_note.get()
                            on:input=move |ev| follow_up_note.set(event_target_value(&ev))
                            placeholder="Add desk follow-up note"
                            style="padding:0.55rem 0.65rem;border:1px solid #cbd5e1;border-radius:0.75rem;"
                        />
                        <div style="display:flex;justify-content:flex-end;">
                            <button
                                type="button"
                                disabled=move || pending_note_leg_id.get() == Some(leg_id)
                                on:click=move |_| {
                                    let note = follow_up_note.get();
                                    if !note.trim().is_empty() {
                                        submit_follow_up(leg_id, desk_key_value.clone(), note);
                                        follow_up_note.set(String::new());
                                    }
                                }
                                style="padding:0.45rem 0.7rem;border:1px solid #cbd5e1;border-radius:0.75rem;background:white;cursor:pointer;"
                            >
                                {move || if pending_note_leg_id.get() == Some(leg_id) { "Saving note..." } else { "Save follow-up" }}
                            </button>
                        </div>
                    </div>
                </div>
            </td>
        </tr>
    }
}
