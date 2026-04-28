use futures_util::future::AbortHandle;
use leptos::{prelude::*, tachys::view::any_view::IntoAny, task::spawn_local};
use shared::{RealtimeEventKind, RealtimeTopic, StloadsReconciliationScreen};

use crate::{
    api, realtime,
    session::{self, use_auth},
};

use super::admin_guard_view;

fn tone_style(tone: &str) -> &'static str {
    match tone {
        "success" => "background:#e8fff3;padding:0.25rem 0.6rem;border-radius:999px;color:#0f766e;",
        "warning" => "background:#fff7dd;padding:0.25rem 0.6rem;border-radius:999px;color:#b45309;",
        "danger" => "background:#ffe4e6;padding:0.25rem 0.6rem;border-radius:999px;color:#be123c;",
        "info" => "background:#e0f2fe;padding:0.25rem 0.6rem;border-radius:999px;color:#0369a1;",
        "primary" => "background:#ede9fe;padding:0.25rem 0.6rem;border-radius:999px;color:#6d28d9;",
        "secondary" => {
            "background:#f1f5f9;padding:0.25rem 0.6rem;border-radius:999px;color:#475569;"
        }
        _ => "background:#111827;padding:0.25rem 0.6rem;border-radius:999px;color:white;",
    }
}

#[component]
pub fn StloadsReconciliationPage() -> impl IntoView {
    let auth = use_auth();
    let screen = RwSignal::new(None::<StloadsReconciliationScreen>);
    let selected_action = RwSignal::new(None::<String>);
    let is_loading = RwSignal::new(false);
    let error_message = RwSignal::new(None::<String>);
    let action_message = RwSignal::new(None::<String>);
    let refresh_nonce = RwSignal::new(0_u64);
    let ws_connected = RwSignal::new(false);
    let ws_handle = RwSignal::new(None::<AbortHandle>);

    let can_view = Signal::derive(move || {
        session::has_permission(&auth, "access_admin_portal")
            || session::has_permission(&auth, "manage_tms_operations")
    });

    Effect::new(move |_| {
        let ready = auth.session_ready.get();
        let current_session = auth.session.get();
        let action_filter = selected_action.get();
        let _refresh = refresh_nonce.get();

        if !ready || !current_session.authenticated || !can_view.get() {
            return;
        }

        is_loading.set(true);
        let auth = auth.clone();

        spawn_local(async move {
            match api::fetch_stloads_reconciliation_screen(action_filter.as_deref()).await {
                Ok(next_screen) => {
                    screen.set(Some(next_screen));
                    error_message.set(None);
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
            vec![RealtimeTopic::AdminTmsReconciliation],
            move |event| match event.kind {
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
                RealtimeEventKind::TmsReconciliationUpdated => {
                    refresh_nonce.update(|value| *value += 1);
                    action_message.set(Some(format!("Realtime update: {}", event.summary)));
                }
                _ => {}
            },
        );

        ws_connected.set(handle.is_some());
        ws_handle.set(handle);
    });

    view! {
        {move || {
            if let Some(guard) = admin_guard_view(
                &auth,
                "STLOADS Reconciliation",
                &["access_admin_portal", "manage_tms_operations"],
            ) {
                guard
            } else {
                view! {
                    <article style="display:grid;gap:1.25rem;">
                        <section style="display:flex;justify-content:space-between;gap:1rem;align-items:flex-start;flex-wrap:wrap;">
                            <div>
                                <h2>{move || screen.get().map(|value| value.title).unwrap_or_else(|| "STLOADS Reconciliation".into())}</h2>
                            </div>
                            <span style=tone_style(if ws_connected.get() { "success" } else { "secondary" })>
                                {move || if ws_connected.get() { "Realtime connected" } else { "Realtime reconnecting" }}
                            </span>
                        </section>

                        {move || error_message.get().map(|message| view! {
                            <section style="padding:0.85rem 1rem;border:1px solid #fecaca;border-radius:0.9rem;background:#fff1f2;color:#be123c;">
                                {message}
                            </section>
                        })}

                        <section style="display:grid;grid-template-columns:repeat(auto-fit,minmax(170px,1fr));gap:0.85rem;">
                            {move || screen.get().map(|value| {
                                value.mismatch_cards
                                    .into_iter()
                                    .map(|card| {
                                        view! {
                                            <div style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#ffffff;display:grid;gap:0.35rem;">
                                                <span style=tone_style(&card.tone)>{card.label}</span>
                                                <strong style="font-size:1.3rem;">{card.value}</strong>
                                            </div>
                                        }
                                    })
                                    .collect_view()
                            })}
                        </section>

                        <section style="display:flex;gap:0.5rem;flex-wrap:wrap;">
                            {move || screen.get().map(|value| {
                                value.action_filters
                                    .into_iter()
                                    .map(|action| {
                                        let action_key = action.clone();
                                        let is_active = value.active_action.as_deref() == Some(action.as_str());
                                        let style = if is_active {
                                            "padding:0.45rem 0.8rem;border-radius:999px;background:#111827;color:white;border:none;cursor:pointer;"
                                        } else {
                                            "padding:0.45rem 0.8rem;border-radius:999px;background:#f4f4f5;color:#111827;border:none;cursor:pointer;"
                                        };
                                        view! {
                                            <button
                                                type="button"
                                                style=style
                                                on:click=move |_| {
                                                    if action_key == "all" {
                                                        selected_action.set(None);
                                                    } else {
                                                        selected_action.set(Some(action_key.clone()));
                                                    }
                                                    action_message.set(None);
                                                }
                                            >
                                                {action}
                                            </button>
                                        }
                                    })
                                    .collect_view()
                            })}
                        </section>

                        <section style="display:grid;grid-template-columns:minmax(260px,340px) minmax(0,1fr);gap:1rem;align-items:start;">
                            <div style="border:1px solid #e5e7eb;border-radius:1rem;padding:1rem;background:#fcfcfb;display:grid;gap:0.75rem;">
                                <strong>"Unresolved sync errors by class"</strong>
                                {move || {
                                    if is_loading.get() && screen.get().is_none() {
                                        view! { <p style="margin:0;">"Loading reconciliation breakdown from the Rust backend..."</p> }.into_any()
                                    } else {
                                        screen
                                            .get()
                                            .map(|value| {
                                                value.error_breakdown
                                                    .into_iter()
                                                    .map(|row| {
                                                        let severity = row.severity;
                                                        let severity_style = tone_style(&severity);
                                                        view! {
                                                            <div style="display:flex;justify-content:space-between;gap:0.75rem;align-items:center;padding:0.75rem 0;border-top:1px solid #f1f5f9;">
                                                                <div>
                                                                    <code>{row.error_class}</code>
                                                                    <div><small>{row.count}</small></div>
                                                                </div>
                                                                <span style=severity_style>{severity}</span>
                                                            </div>
                                                        }
                                                    })
                                                    .collect_view()
                                                    .into_any()
                                            })
                                            .unwrap_or_else(|| view! { <p style="margin:0;">"No reconciliation error data is available yet."</p> }.into_any())
                                    }
                                }}
                            </div>

                            <div style="overflow:auto;border:1px solid #e5e7eb;border-radius:1rem;">
                                <div style="padding:1rem;border-bottom:1px solid #e5e7eb;display:flex;justify-content:space-between;gap:1rem;flex-wrap:wrap;align-items:center;">
                                    <strong>"Reconciliation log"</strong>
                                    <small>{move || screen.get().map(|value| format!("{} total audit entries", value.pagination.total)).unwrap_or_else(|| "Loading entries...".into())}</small>
                                </div>
                                <table style="width:100%;border-collapse:collapse;min-width:940px;">
                                    <thead style="background:#f8fafc;">
                                        <tr>
                                            <th style="text-align:left;padding:0.9rem;">"#"</th>
                                            <th style="text-align:left;padding:0.9rem;">"Action"</th>
                                            <th style="text-align:left;padding:0.9rem;">"Handoff"</th>
                                            <th style="text-align:left;padding:0.9rem;">"TMS"</th>
                                            <th style="text-align:left;padding:0.9rem;">"STLOADS"</th>
                                            <th style="text-align:left;padding:0.9rem;">"Detail"</th>
                                            <th style="text-align:left;padding:0.9rem;">"By"</th>
                                            <th style="text-align:left;padding:0.9rem;">"When"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {move || {
                                            if is_loading.get() && screen.get().is_none() {
                                                view! {
                                                    <tr>
                                                        <td colspan="8" style="padding:1rem;">"Loading reconciliation logs from the Rust backend..."</td>
                                                    </tr>
                                                }
                                                .into_any()
                                            } else {
                                                screen
                                                    .get()
                                                    .map(|value| {
                                                        value.logs
                                                            .into_iter()
                                                            .map(|log| {
                                                                view! {
                                                                    <tr style="border-top:1px solid #f1f5f9;vertical-align:top;">
                                                                        <td style="padding:0.9rem;">{log.id}</td>
                                                                        <td style="padding:0.9rem;">
                                                                            <span style=tone_style(&log.action_tone)>{log.action}</span>
                                                                        </td>
                                                                        <td style="padding:0.9rem;">{log.handoff_ref.unwrap_or_else(|| "n/a".into())}</td>
                                                                        <td style="padding:0.9rem;">{log.tms_transition.unwrap_or_else(|| "n/a".into())}</td>
                                                                        <td style="padding:0.9rem;">{log.stloads_transition.unwrap_or_else(|| "n/a".into())}</td>
                                                                        <td style="padding:0.9rem;">{log.detail}</td>
                                                                        <td style="padding:0.9rem;">{log.triggered_by}</td>
                                                                        <td style="padding:0.9rem;">{log.created_at_label}</td>
                                                                    </tr>
                                                                }
                                                            })
                                                            .collect_view()
                                                            .into_any()
                                                    })
                                                    .unwrap_or_else(|| view! {
                                                        <tr>
                                                            <td colspan="8" style="padding:1rem;">"No reconciliation log data is available yet."</td>
                                                        </tr>
                                                    }
                                                    .into_any())
                                            }
                                        }}
                                    </tbody>
                                </table>
                            </div>
                        </section>

                    </article>
                }
                .into_any()
            }
        }}
    }
}
