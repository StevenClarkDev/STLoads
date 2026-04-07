use futures_util::future::AbortHandle;
use leptos::{prelude::*, tachys::view::any_view::IntoAny, task::spawn_local};
use leptos_router::components::A;
use shared::{RealtimeEventKind, RealtimeTopic};

use crate::{
    api::{self, AdminOverview},
    realtime,
    session::{self, use_auth},
};

use super::admin_guard_view;

fn tone_style(tone: &str) -> &'static str {
    match tone {
        "success" => "background:#e8fff3;padding:0.25rem 0.6rem;border-radius:999px;color:#0f766e;",
        "secondary" => {
            "background:#f1f5f9;padding:0.25rem 0.6rem;border-radius:999px;color:#475569;"
        }
        _ => "background:#e0f2fe;padding:0.25rem 0.6rem;border-radius:999px;color:#0369a1;",
    }
}

#[component]
pub fn AdminDashboardPage() -> impl IntoView {
    let auth = use_auth();
    let overview = RwSignal::new(None::<AdminOverview>);
    let is_loading = RwSignal::new(false);
    let error_message = RwSignal::new(None::<String>);
    let action_message = RwSignal::new(None::<String>);
    let refresh_nonce = RwSignal::new(0_u64);
    let ws_connected = RwSignal::new(false);
    let ws_handle = RwSignal::new(None::<AbortHandle>);

    let can_view = Signal::derive(move || {
        session::has_permission(&auth, "access_admin_portal")
            || session::has_permission(&auth, "manage_tms_operations")
            || session::has_permission(&auth, "manage_payments")
            || session::has_permission(&auth, "manage_master_data")
    });

    Effect::new(move |_| {
        let ready = auth.session_ready.get();
        let current_session = auth.session.get();
        let _refresh = refresh_nonce.get();

        if !ready || !current_session.authenticated || !can_view.get() {
            return;
        }

        is_loading.set(true);
        let auth = auth.clone();

        spawn_local(async move {
            match api::fetch_admin_overview().await {
                Ok(next_overview) => {
                    overview.set(Some(next_overview));
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
            vec![
                RealtimeTopic::AdminDashboard,
                RealtimeTopic::AdminTmsOperations,
                RealtimeTopic::AdminTmsReconciliation,
                RealtimeTopic::AdminPayments,
            ],
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
                RealtimeEventKind::AdminDashboardUpdated
                | RealtimeEventKind::TmsOperationsUpdated
                | RealtimeEventKind::TmsReconciliationUpdated
                | RealtimeEventKind::PaymentsOperationsUpdated => {
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
                "Admin Dashboard",
                &["access_admin_portal", "manage_tms_operations", "manage_payments", "manage_master_data"],
            ) {
                guard
            } else {
                view! {
                    <article style="display:grid;gap:1.25rem;">
                        <section style="display:flex;justify-content:space-between;gap:1rem;align-items:flex-start;flex-wrap:wrap;">
                            <div>
                                <h2>"Admin Dashboard"</h2>
                                <p>
                                    "This dashboard now loads its route inventory from the Rust backend and refreshes only for admin-scoped realtime events."
                                </p>
                            </div>
                            <span style=tone_style(if ws_connected.get() { "success" } else { "secondary" })>
                                {move || if ws_connected.get() { "Realtime connected" } else { "Realtime reconnecting" }}
                            </span>
                        </section>

                        {move || action_message.get().map(|message| view! {
                            <section style="padding:0.85rem 1rem;border:1px solid #dbeafe;border-radius:0.9rem;background:#eff6ff;color:#1d4ed8;">
                                {message}
                            </section>
                        })}

                        {move || error_message.get().map(|message| view! {
                            <section style="padding:0.85rem 1rem;border:1px solid #fecaca;border-radius:0.9rem;background:#fff1f2;color:#be123c;">
                                {message}
                            </section>
                        })}

                        <section style="display:grid;grid-template-columns:repeat(auto-fit,minmax(240px,1fr));gap:1rem;">
                            {move || {
                                if is_loading.get() && overview.get().is_none() {
                                    view! {
                                        <div style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#fafaf9;">
                                            "Loading admin route inventory from the Rust backend..."
                                        </div>
                                    }
                                    .into_any()
                                } else {
                                    overview
                                        .get()
                                        .map(|data| {
                                            data.screen_routes
                                                .into_iter()
                                                .map(render_admin_card)
                                                .collect_view()
                                                .into_any()
                                        })
                                        .unwrap_or_else(|| view! {
                                            <div style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#fafaf9;">
                                                "No admin route inventory is available yet."
                                            </div>
                                        }
                                        .into_any())
                                }
                            }}
                        </section>

                        <section style="display:grid;gap:0.5rem;">
                            {move || overview.get().map(|data| view! {
                                <>
                                    <strong>{format!("{} operational views are currently wired through the Rust admin surface.", data.operational_views)}</strong>
                                    {data
                                        .notes
                                        .into_iter()
                                        .map(|note| view! { <p style="margin:0;">{note}</p> })
                                        .collect_view()}
                                </>
                            })}
                        </section>
                    </article>
                }
                .into_any()
            }
        }}
    }
}

fn render_admin_card(route: String) -> impl IntoView {
    let (label, detail) = match route.as_str() {
        "/admin/stloads/operations" => (
            "STLOADS Operations",
            "Publish queue, alerts, and handoff table",
        ),
        "/admin/stloads/reconciliation" => (
            "Reconciliation Desk",
            "Mismatch counts, sync errors, and audit trail",
        ),
        "/admin/payments" => ("Escrow Operations", "Funding and payout readiness"),
        "/admin/master-data" => (
            "Master Data",
            "Read-first visibility for admin lookup catalogs",
        ),
        _ => (
            "Admin Route",
            "Additional admin surface discovered from the backend.",
        ),
    };

    view! {
        <A href=route attr:style="display:block;padding:1rem;border:1px solid #d6d3d1;border-radius:1rem;text-decoration:none;color:inherit;background:#fcfcfb;">
            <strong>{label}</strong>
            <p style="margin:0.5rem 0 0;">{detail}</p>
        </A>
    }
}
