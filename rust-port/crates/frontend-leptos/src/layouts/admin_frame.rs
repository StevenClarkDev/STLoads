use leptos::prelude::*;
use leptos_router::components::A;

use crate::session::{self, use_auth};

#[component]
pub fn AdminFrame(children: Children) -> impl IntoView {
    let auth = use_auth();

    let can_open_admin_dashboard = Signal::derive(move || {
        session::has_permission(&auth, "access_admin_portal")
            || session::has_permission(&auth, "manage_tms_operations")
            || session::has_permission(&auth, "manage_payments")
            || session::has_permission(&auth, "manage_master_data")
            || session::has_permission(&auth, "manage_roles")
    });
    let can_manage_tms = Signal::derive(move || {
        session::has_permission(&auth, "access_admin_portal")
            || session::has_permission(&auth, "manage_tms_operations")
    });
    let can_manage_payments = Signal::derive(move || {
        session::has_permission(&auth, "access_admin_portal")
            || session::has_permission(&auth, "manage_payments")
    });
    let can_manage_master_data = Signal::derive(move || {
        session::has_permission(&auth, "access_admin_portal")
            || session::has_permission(&auth, "manage_master_data")
    });
    let can_manage_users = Signal::derive(move || {
        session::has_permission(&auth, "access_admin_portal")
            || session::has_permission(&auth, "manage_users")
    });
    let can_manage_loads = Signal::derive(move || {
        session::has_permission(&auth, "access_admin_portal")
            || session::has_permission(&auth, "manage_loads")
    });
    let can_manage_roles = Signal::derive(move || {
        session::has_permission(&auth, "access_admin_portal")
            || session::has_permission(&auth, "manage_roles")
    });

    view! {
        <main class="app-shell admin-frame">
            <header class="shell-header">
                <div class="shell-topbar">
                    <div class="shell-title-row">
                        <div class="brand-mark" aria-hidden="true">"OPS"</div>
                        <div class="shell-brand-copy">
                            <p class="shell-kicker">"STLoads Command"</p>
                            <h1 class="shell-title">"Admin Control"</h1>
                            <p class="shell-subtitle">
                                "Escrows. Users. Loads. STLOADS."
                            </p>
                        </div>
                    </div>
                    <div class="session-card">
                        <span class="session-pill">"Operator Console"</span>
                        <A href="/" attr:class="shell-secondary">"User Dashboard"</A>
                    </div>
                </div>
                <nav class="shell-nav" aria-label="Admin">
                    {move || can_open_admin_dashboard.get().then(|| view! {
                        <A href="/admin" attr:class="shell-nav-link">"Dashboard"</A>
                    })}
                    {move || can_manage_payments.get().then(|| view! {
                        <A href="/admin/payments" attr:class="shell-nav-link">"Escrows"</A>
                    })}
                    {move || can_manage_master_data.get().then(|| view! {
                        <A href="/admin/master-data" attr:class="shell-nav-link">"Master Data"</A>
                    })}
                    {move || can_manage_users.get().then(|| view! {
                        <>
                            <A href="/admin/account-lifecycle" attr:class="shell-nav-link">"Lifecycle QA"</A>
                            <A href="/admin/users" attr:class="shell-nav-link">"Users"</A>
                            <A href="/admin/change-password" attr:class="shell-nav-link">"Passwords"</A>
                            <A href="/admin/users/role/carrier" attr:class="shell-nav-link">"Carriers"</A>
                            <A href="/admin/users/role/shipper" attr:class="shell-nav-link">"Shippers"</A>
                            <A href="/admin/users/role/broker" attr:class="shell-nav-link">"Brokers"</A>
                            <A href="/admin/users/role/freight_forwarder" attr:class="shell-nav-link">"Forwarders"</A>
                            <A href="/admin/onboarding-reviews" attr:class="shell-nav-link">"Reviews"</A>
                        </>
                    })}
                    {move || can_manage_loads.get().then(|| view! {
                        <A href="/admin/loads" attr:class="shell-nav-link">"Loads"</A>
                    })}
                    {move || can_manage_roles.get().then(|| view! {
                        <A href="/admin/roles" attr:class="shell-nav-link">"Roles"</A>
                    })}
                    {move || can_manage_tms.get().then(|| view! {
                        <>
                            <A href="/admin/stloads/operations" attr:class="shell-nav-link">"STLOADS Ops"</A>
                            <A href="/admin/stloads/reconciliation" attr:class="shell-nav-link">"Reconciliation"</A>
                        </>
                    })}
                </nav>
            </header>
            <section>{children()}</section>
        </main>
    }
}
