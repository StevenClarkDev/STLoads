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
            || session::has_permission(&auth, "manage_users")
            || session::has_permission(&auth, "manage_roles")
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
                &["access_admin_portal", "manage_tms_operations", "manage_payments", "manage_master_data", "manage_users", "manage_roles"],
            ) {
                guard
            } else {
                let route_count = overview.get().as_ref().map(|data| data.screen_routes.len()).unwrap_or(0);
                let operational_views = overview.get().as_ref().map(|data| data.operational_views).unwrap_or(0);
                let user_total = overview.get().as_ref().map(|data| data.user_total).unwrap_or(0);
                let shipper_total = overview.get().as_ref().map(|data| data.shipper_total).unwrap_or(0);
                let carrier_total = overview.get().as_ref().map(|data| data.carrier_total).unwrap_or(0);
                let broker_total = overview.get().as_ref().map(|data| data.broker_total).unwrap_or(0);
                let freight_forwarder_total = overview.get().as_ref().map(|data| data.freight_forwarder_total).unwrap_or(0);
                let admin_total = overview.get().as_ref().map(|data| data.admin_total).unwrap_or(0);

                view! {
                    <article class="php-grid">
                        <section class="php-page-title">
                            <div>
                                <h4>"Admin Dashboard"</h4>
                                <p>"Overview"</p>
                            </div>
                            <ol class="php-breadcrumb">
                                <li><i class="fas fa-home"></i></li>
                                <li>"Admin"</li>
                                <li><strong>"Overview"</strong></li>
                            </ol>
                        </section>

                        {move || error_message.get().map(|message| view! {
                            <section class="auth-notice" style="border-color:#fecaca;background:#fff1f2;color:#be123c;">{message}</section>
                        })}

                        <section class="php-grid columns-3">
                            <div class="php-card php-widget-card">
                                <div class="php-card-body">
                                    <span class="label">"User Accounts"</span>
                                    <span class="value">{move || user_total.to_string()}</span>
                                    <span class="sub">"Admin-visible accounts across shipper, carrier, broker, freight forwarder, and admin roles"</span>
                                    <div class="icon php-icon-primary"><i class="fas fa-users"></i></div>
                                </div>
                            </div>
                            <div class="php-card php-widget-card">
                                <div class="php-card-body">
                                    <span class="label">"Dashboard Routes"</span>
                                    <span class="value">{move || route_count.to_string()}</span>
                                    <span class="sub">"Backend-discovered admin routes"</span>
                                    <div class="icon php-icon-warning"><i class="fas fa-route"></i></div>
                                </div>
                            </div>
                            <div class="php-card php-widget-card">
                                <div class="php-card-body">
                                    <span class="label">"Realtime"</span>
                                    <span class="value">{move || if ws_connected.get() { "Live" } else { "Retry" }}</span>
                                    <span class="sub">"Admin-scoped event channel state"</span>
                                    <div class="icon php-icon-success"><i class="fas fa-signal"></i></div>
                                </div>
                            </div>
                        </section>

                        <section class="php-grid columns-3">
                            <div class="php-card">
                                <div class="php-card-body">
                                    <h5 style="margin:0 0 1rem;">"Role Breakdown"</h5>
                                    <div class="php-grid columns-2" style="gap:0.75rem;">
                                        <div class="dashboard-card">
                                            <span class="status-pill warning">"Shipper"</span>
                                            <strong class="dashboard-card-title">{move || shipper_total.to_string()}</strong>
                                            <p class="dashboard-card-copy">"Shipper accounts visible to admin"</p>
                                        </div>
                                        <div class="dashboard-card">
                                            <span class="status-pill warning">"Carrier"</span>
                                            <strong class="dashboard-card-title">{move || carrier_total.to_string()}</strong>
                                            <p class="dashboard-card-copy">"Carrier accounts visible to admin"</p>
                                        </div>
                                        <div class="dashboard-card">
                                            <span class="status-pill warning">"Broker"</span>
                                            <strong class="dashboard-card-title">{move || broker_total.to_string()}</strong>
                                            <p class="dashboard-card-copy">"Broker accounts visible to admin"</p>
                                        </div>
                                        <div class="dashboard-card">
                                            <span class="status-pill warning">"Freight Forwarder"</span>
                                            <strong class="dashboard-card-title">{move || freight_forwarder_total.to_string()}</strong>
                                            <p class="dashboard-card-copy">"Forwarder accounts visible to admin"</p>
                                        </div>
                                        <div class="dashboard-card">
                                            <span class="status-pill warning">"Admin"</span>
                                            <strong class="dashboard-card-title">{move || admin_total.to_string()}</strong>
                                            <p class="dashboard-card-copy">"Admin accounts in the current Rust directory"</p>
                                        </div>
                                        <div class="dashboard-card">
                                            <span class="status-pill warning">"Ops Surfaces"</span>
                                            <strong class="dashboard-card-title">{move || operational_views.to_string()}</strong>
                                            <p class="dashboard-card-copy">"Operational workspaces currently wired through Rust admin"</p>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </section>

                        <section class="php-grid columns-2">
                            <div class="php-card">
                                <div class="php-card-body">
                                    <h5 style="margin:0 0 1rem;">"Admin Workspaces"</h5>
                                    {move || {
                                        if is_loading.get() && overview.get().is_none() {
                                            view! { <p style="margin:0;color:#64748b;">"Loading admin route inventory from the Rust backend..."</p> }.into_any()
                                        } else {
                                            overview
                                                .get()
                                                .map(|data| {
                                                    view! {
                                                        <div class="php-grid">
                                                            {data.screen_routes
                                                                .into_iter()
                                                                .map(render_admin_card)
                                                                .collect_view()}
                                                        </div>
                                                    }
                                                        .into_any()
                                                })
                                                .unwrap_or_else(|| view! {
                                                    <p style="margin:0;color:#64748b;">"No admin route inventory is available yet."</p>
                                                }
                                                .into_any())
                                        }
                                    }}
                                </div>
                            </div>
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
        "/admin/users" => (
            "User Directory",
            "Role and account-state controls for all users",
        ),
        "/admin/loads" => (
            "Admin Loads",
            "Approval-stage, active, completed, and release-ready oversight",
        ),
        "/admin/roles/permissions" | "/admin/roles" => (
            "Roles & Permissions",
            "Database-backed role permission matrix for live sessions",
        ),
        "/admin/stloads/operations" => (
            "STLOADS Operations",
            "Publish queue, alerts, and handoff table",
        ),
        "/admin/stloads/reconciliation" => (
            "Reconciliation",
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
        <A href=route attr:class="dashboard-card">
            <span class="status-pill warning">"Admin"</span>
            <strong class="dashboard-card-title">{label}</strong>
            <p class="dashboard-card-copy">{detail}</p>
        </A>
    }
}
