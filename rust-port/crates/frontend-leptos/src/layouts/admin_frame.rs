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
        <main class="admin-frame">
            <header>
                <div>
                    <p>"STLoads Admin Shell"</p>
                    <h1>"Admin Portal"</h1>
                </div>
                <nav>
                    {move || can_open_admin_dashboard.get().then(|| view! {
                        <>
                            <A href="/admin">"Admin Dashboard"</A>
                            " | "
                        </>
                    })}
                    {move || can_manage_payments.get().then(|| view! {
                        <>
                            <A href="/admin/payments">"Escrows"</A>
                            " | "
                        </>
                    })}
                    {move || can_manage_master_data.get().then(|| view! {
                        <>
                            <A href="/admin/master-data">"Master Data"</A>
                            " | "
                        </>
                    })}
                    {move || can_manage_users.get().then(|| view! {
                        <>
                            <A href="/admin/account-lifecycle">"Lifecycle QA"</A>
                            " | "
                            <A href="/admin/users">"Users"</A>
                            " | "
                            <A href="/admin/change-password">"Change Password"</A>
                            " | "
                            <A href="/admin/users/role/carrier">"Carriers"</A>
                            " | "
                            <A href="/admin/users/role/shipper">"Shippers"</A>
                            " | "
                            <A href="/admin/users/role/broker">"Brokers"</A>
                            " | "
                            <A href="/admin/users/role/freight_forwarder">"Freight Forwarders"</A>
                            " | "
                            <A href="/admin/onboarding-reviews">"Onboarding Reviews"</A>
                            " | "
                        </>
                    })}
                    {move || can_manage_loads.get().then(|| view! {
                        <>
                            <A href="/admin/loads">"Loads"</A>
                            " | "
                        </>
                    })}
                    {move || can_manage_roles.get().then(|| view! {
                        <>
                            <A href="/admin/roles">"Roles & Permissions"</A>
                            " | "
                        </>
                    })}
                    {move || can_manage_tms.get().then(|| view! {
                        <>
                            <A href="/admin/stloads/operations">"STLOADS Ops"</A>
                            " | "
                            <A href="/admin/stloads/reconciliation">"Reconciliation"</A>
                            " | "
                        </>
                    })}
                    <A href="/">"User Dashboard"</A>
                </nav>
            </header>
            <section>{children()}</section>
        </main>
    }
}
