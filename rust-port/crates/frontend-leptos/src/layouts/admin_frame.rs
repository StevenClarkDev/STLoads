use leptos::{ev::SubmitEvent, prelude::*, task::spawn_local};
use leptos_router::{
    components::A,
    hooks::{use_location, use_navigate},
};

use crate::session::{self, use_auth};

#[component]
pub fn AdminFrame(children: Children) -> impl IntoView {
    let auth = use_auth();
    let location = use_location();
    let navigate = use_navigate();
    let logout_navigate = navigate.clone();
    let search_navigate = navigate.clone();
    let quick_jump = RwSignal::new(String::new());

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

    let pathname = move || location.pathname.get();
    let logout = move |_| {
        let auth = auth.clone();
        let navigate = logout_navigate.clone();
        spawn_local(async move {
            if let Ok(response) = session::sign_out(auth).await {
                if response.success {
                    navigate("/auth/login?portal=admin", Default::default());
                }
            }
        });
    };
    let on_search = move |ev: SubmitEvent| {
        ev.prevent_default();
        let query = quick_jump.get();
        if let Some(destination) = admin_quick_jump_path(&query) {
            quick_jump.set(String::new());
            search_navigate(&destination, Default::default());
        } else if !query.trim().is_empty() {
            auth.notice.set(Some(
                "Try overview, approvals, users, loads, master data, roles, lifecycle, reconciliation, payments, or quote desk."
                    .into(),
            ));
        }
    };

    view! {
        <main class="php-app-shell admin-frame" style="background:url('https://portal.stloads.com/assets/images/login/texture-bg.jpg') no-repeat center center / cover;">
            <div class="php-page-wrapper">
                <header class="php-page-header">
                    <div class="php-header-left">
                        <div class="php-logo-badge">
                            <img src="https://portal.stloads.com/assets/images/stloads/logo-bg_none-small.png" alt="LoadBoard" />
                        </div>
                        <form class="php-searchbar" on:submit=on_search>
                            <i class="fas fa-search"></i>
                            <input
                                type="text"
                                prop:value=move || quick_jump.get()
                                on:input=move |ev| quick_jump.set(event_target_value(&ev))
                                placeholder="Jump to approvals, users, loads, lifecycle..."
                            />
                        </form>
                    </div>
                    <div class="php-header-right">
                        <button type="button" class="php-mode-toggle" title="Theme toggle">
                            <i class="fas fa-moon"></i>
                        </button>
                        <div class="php-profile-nav">
                            <div class="php-profile-avatar">
                                {move || auth
                                    .session
                                    .get()
                                    .user
                                    .as_ref()
                                    .and_then(|user| user.name.chars().next())
                                    .unwrap_or('A')
                                    .to_string()}
                            </div>
                            <div class="php-profile-copy">
                                {move || auth.session.get().user.map(|user| view! {
                                    <>
                                        <strong>{user.name}</strong>
                                        <small>{format!("{} | {}", user.role_label, user.email)}</small>
                                    </>
                                })}
                            </div>
                            <button
                                type="button"
                                class="shell-action secondary"
                                on:click=logout
                                disabled=move || auth.session_loading.get()
                            >
                                {move || if auth.session_loading.get() { "Working..." } else { "Logout" }}
                            </button>
                        </div>
                    </div>
                </header>

                <div class="php-page-body-wrapper">
                    <aside class="php-sidebar">
                        <div class="php-sidebar-logo">
                            <img src="https://portal.stloads.com/assets/images/stloads/logo-bg_none-small.png" alt="LoadBoard" />
                        </div>

                        <section class="php-sidebar-section">
                            <div class="php-sidebar-title">
                                <i class="fas fa-th"></i>
                                <span>"Dashboard"</span>
                            </div>
                            <nav class="php-sidebar-links">
                                {move || can_open_admin_dashboard.get().then(|| view! {
                                    <A href="/admin" attr:class=sidebar_link_class(pathname() == "/admin")>
                                        <span class="php-sidebar-link-main">
                                            <i class="fas fa-home"></i>
                                            <span>"Overview"</span>
                                        </span>
                                        <i class="fas fa-chevron-right"></i>
                                    </A>
                                })}
                            </nav>
                        </section>

                        {move || can_manage_users.get().then(|| view! {
                            <section class="php-sidebar-section">
                                <div class="php-sidebar-title">
                                    <i class="fas fa-users"></i>
                                    <span>"User Management"</span>
                                </div>
                                <nav class="php-sidebar-links">
                                    <A href="/admin/onboarding-reviews" attr:class=sidebar_link_class(pathname() == "/admin/onboarding-reviews")>
                                        <span class="php-sidebar-link-main">
                                            <i class="fas fa-user-check"></i>
                                            <span>"Pending Approvals"</span>
                                        </span>
                                        <i class="fas fa-chevron-right"></i>
                                    </A>
                                    <A href="/admin/users" attr:class=sidebar_link_class(pathname() == "/admin/users")>
                                        <span class="php-sidebar-link-main">
                                            <i class="fas fa-users"></i>
                                            <span>"All Users"</span>
                                        </span>
                                        <i class="fas fa-chevron-right"></i>
                                    </A>
                                    <A href="/admin/users/role/carrier" attr:class=sidebar_link_class(pathname() == "/admin/users/role/carrier")>
                                        <span class="php-sidebar-link-main">
                                            <i class="fas fa-truck"></i>
                                            <span>"Carriers"</span>
                                        </span>
                                        <i class="fas fa-chevron-right"></i>
                                    </A>
                                    <A href="/admin/users/role/shipper" attr:class=sidebar_link_class(pathname() == "/admin/users/role/shipper")>
                                        <span class="php-sidebar-link-main">
                                            <i class="fas fa-box"></i>
                                            <span>"Shippers"</span>
                                        </span>
                                        <i class="fas fa-chevron-right"></i>
                                    </A>
                                    <A href="/admin/users/role/broker" attr:class=sidebar_link_class(pathname() == "/admin/users/role/broker")>
                                        <span class="php-sidebar-link-main">
                                            <i class="fas fa-briefcase"></i>
                                            <span>"Brokers"</span>
                                        </span>
                                        <i class="fas fa-chevron-right"></i>
                                    </A>
                                    <A href="/admin/users/role/freight_forwarder" attr:class=sidebar_link_class(pathname() == "/admin/users/role/freight_forwarder")>
                                        <span class="php-sidebar-link-main">
                                            <i class="fas fa-paper-plane"></i>
                                            <span>"Freight Forwarders"</span>
                                        </span>
                                        <i class="fas fa-chevron-right"></i>
                                    </A>
                                    <A href="/admin/account-lifecycle" attr:class=sidebar_link_class(pathname() == "/admin/account-lifecycle")>
                                        <span class="php-sidebar-link-main">
                                            <i class="fas fa-route"></i>
                                            <span>"Lifecycle QA"</span>
                                        </span>
                                        <i class="fas fa-chevron-right"></i>
                                    </A>
                                    <A href="/admin/change-password" attr:class=sidebar_link_class(pathname() == "/admin/change-password")>
                                        <span class="php-sidebar-link-main">
                                            <i class="fas fa-key"></i>
                                            <span>"Change Password"</span>
                                        </span>
                                        <i class="fas fa-chevron-right"></i>
                                    </A>
                                    {move || can_manage_roles.get().then(|| view! {
                                        <A href="/admin/roles" attr:class=sidebar_link_class(pathname() == "/admin/roles")>
                                            <span class="php-sidebar-link-main">
                                                <i class="fas fa-shield-alt"></i>
                                                <span>"Roles & Permissions"</span>
                                            </span>
                                            <i class="fas fa-chevron-right"></i>
                                        </A>
                                    })}
                                </nav>
                            </section>
                        })}

                        <section class="php-sidebar-section">
                            <div class="php-sidebar-title">
                                <i class="fas fa-box-open"></i>
                                <span>"Load Operations"</span>
                            </div>
                            <nav class="php-sidebar-links">
                                {move || can_manage_loads.get().then(|| view! {
                                    <A href="/admin/loads" attr:class=sidebar_link_class(pathname().starts_with("/admin/loads"))>
                                        <span class="php-sidebar-link-main">
                                            <i class="fas fa-truck"></i>
                                            <span>"Manage Loads"</span>
                                        </span>
                                        <i class="fas fa-chevron-right"></i>
                                    </A>
                                })}
                                {move || can_manage_tms.get().then(|| view! {
                                    <>
                                        <A href="/admin/stloads/operations" attr:class=sidebar_link_class(pathname() == "/admin/stloads/operations" || pathname() == "/admin/stloads")>
                                            <span class="php-sidebar-link-main">
                                                <i class="fas fa-broadcast-tower"></i>
                                                <span>"STLOADS Operations"</span>
                                            </span>
                                            <i class="fas fa-chevron-right"></i>
                                        </A>
                                        <A href="/admin/stloads/reconciliation" attr:class=sidebar_link_class(pathname() == "/admin/stloads/reconciliation")>
                                            <span class="php-sidebar-link-main">
                                                <i class="fas fa-code-branch"></i>
                                                <span>"Reconciliation"</span>
                                            </span>
                                            <i class="fas fa-chevron-right"></i>
                                        </A>
                                    </>
                                })}
                                {move || can_manage_payments.get().then(|| view! {
                                    <A href="/admin/payments" attr:class=sidebar_link_class(pathname() == "/admin/payments")>
                                        <span class="php-sidebar-link-main">
                                            <i class="fas fa-dollar-sign"></i>
                                            <span>"Escrow Operations"</span>
                                        </span>
                                        <i class="fas fa-chevron-right"></i>
                                    </A>
                                })}
                            </nav>
                        </section>

                        <section class="php-sidebar-section">
                            <div class="php-sidebar-title">
                                <i class="fas fa-columns"></i>
                                <span>"Dispatch Desks"</span>
                            </div>
                            <nav class="php-sidebar-links">
                                <A href="/desk/quote" attr:class=sidebar_link_class(pathname() == "/desk/quote")>
                                    <span class="php-sidebar-link-main">
                                        <i class="fas fa-dollar-sign"></i>
                                        <span>"Quote Desk"</span>
                                    </span>
                                    <i class="fas fa-chevron-right"></i>
                                </A>
                            </nav>
                        </section>

                        {move || can_manage_master_data.get().then(|| view! {
                            <section class="php-sidebar-section">
                                <div class="php-sidebar-title">
                                    <i class="fas fa-cog"></i>
                                    <span>"System Configuration"</span>
                                </div>
                                <nav class="php-sidebar-links">
                                    <A href="/admin/master-data" attr:class=sidebar_link_class(pathname() == "/admin/master-data")>
                                        <span class="php-sidebar-link-main">
                                            <i class="fas fa-database"></i>
                                            <span>"Master Data"</span>
                                        </span>
                                        <i class="fas fa-chevron-right"></i>
                                    </A>
                                </nav>
                            </section>
                        })}

                        <section class="php-sidebar-section">
                            <nav class="php-sidebar-links">
                                <A href="/" attr:class="php-sidebar-link">
                                    <span class="php-sidebar-link-main">
                                        <i class="fas fa-arrow-left"></i>
                                        <span>"User Dashboard"</span>
                                    </span>
                                    <i class="fas fa-chevron-right"></i>
                                </A>
                            </nav>
                        </section>
                    </aside>

                    <section class="php-page-body">{children()}</section>
                </div>

                <footer class="php-footer">
                    "© 2025 Load Board All Rights Reserved"
                </footer>
            </div>
        </main>
    }
}

fn sidebar_link_class(active: bool) -> &'static str {
    if active {
        "php-sidebar-link is-active"
    } else {
        "php-sidebar-link"
    }
}

fn admin_quick_jump_path(query: &str) -> Option<String> {
    let query = query.trim().to_ascii_lowercase();
    if query.is_empty() {
        return None;
    }

    if query.contains("dashboard") || query.contains("overview") || query == "admin" {
        return Some("/admin".into());
    }
    if query.contains("approval") || query.contains("onboarding") || query.contains("review") {
        return Some("/admin/onboarding-reviews".into());
    }
    if query.contains("lifecycle") {
        return Some("/admin/account-lifecycle".into());
    }
    if query.contains("role") || query.contains("permission") {
        return Some("/admin/roles".into());
    }
    if query.contains("master") || query.contains("catalog") || query.contains("data") {
        return Some("/admin/master-data".into());
    }
    if query.contains("reconciliation") {
        return Some("/admin/stloads/reconciliation".into());
    }
    if query.contains("stloads") || query.contains("operation") {
        return Some("/admin/stloads/operations".into());
    }
    if query.contains("payment") || query.contains("escrow") {
        return Some("/admin/payments".into());
    }
    if query.contains("load") {
        return Some("/admin/loads".into());
    }
    if query.contains("carrier") {
        return Some("/admin/users/role/carrier".into());
    }
    if query.contains("shipper") {
        return Some("/admin/users/role/shipper".into());
    }
    if query.contains("broker") {
        return Some("/admin/users/role/broker".into());
    }
    if query.contains("forwarder") || query.contains("freight") {
        return Some("/admin/users/role/freight_forwarder".into());
    }
    if query.contains("user") {
        return Some("/admin/users".into());
    }
    if query.contains("quote") || query.contains("desk") {
        return Some("/desk/quote".into());
    }

    None
}
