use leptos::{ev::SubmitEvent, prelude::*, task::spawn_local};
use leptos_router::{
    components::A,
    hooks::{use_location, use_navigate},
};

use crate::session::{self, use_auth};

#[component]
pub fn UserFrame(children: Children) -> impl IntoView {
    let auth = use_auth();
    let location = use_location();
    let navigate = use_navigate();
    let logout_navigate = navigate.clone();
    let search_navigate = navigate.clone();
    let quick_jump = RwSignal::new(String::new());

    let logout = move |_| {
        let auth = auth.clone();
        let navigate = logout_navigate.clone();
        spawn_local(async move {
            if let Ok(response) = session::sign_out(auth).await {
                if response.success {
                    navigate("/", Default::default());
                }
            }
        });
    };

    let show_admin_link =
        Signal::derive(move || session::has_permission(&auth, "access_admin_portal"));
    let show_stloads_links = Signal::derive(move || show_admin_link.get());

    let show_create_load_link =
        Signal::derive(move || session::has_permission(&auth, "manage_loads"));
    let show_dispatch_desk_link = Signal::derive(move || {
        session::has_permission(&auth, "manage_dispatch_desk")
            || session::has_permission(&auth, "manage_loads")
            || session::has_permission(&auth, "access_admin_portal")
    });

    let show_onboarding_link = Signal::derive(move || {
        auth.session
            .get()
            .user
            .as_ref()
            .map(|user| user.dashboard_href == "/auth/onboarding")
            .unwrap_or(false)
    });

    let pathname = move || location.pathname.get();
    let on_search = move |ev: SubmitEvent| {
        ev.prevent_default();
        let query = quick_jump.get();
        if let Some(destination) = user_quick_jump_path(
            &query,
            show_admin_link.get(),
            show_onboarding_link.get(),
            show_create_load_link.get(),
            show_dispatch_desk_link.get(),
        ) {
            quick_jump.set(String::new());
            search_navigate(&destination, Default::default());
        } else if !query.trim().is_empty() {
            auth.notice.set(Some(
                "Try dashboard, loads, create load, quote desk, chat, profile, onboarding, or admin."
                    .into(),
            ));
        }
    };

    view! {
        <main class="php-app-shell user-frame">
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
                                placeholder="Jump to dashboard, loads, chat, profile..."
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
                                    .unwrap_or('U')
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
                                <span>"General"</span>
                            </div>
                            <nav class="php-sidebar-links">
                                <A href="/dashboard" attr:class=move || sidebar_link_class(pathname() == "/dashboard")>
                                    <span class="php-sidebar-link-main">
                                        <i class="fas fa-home"></i>
                                        <span>"Dashboard"</span>
                                    </span>
                                    <i class="fas fa-chevron-right"></i>
                                </A>
                            </nav>
                        </section>

                        <section class="php-sidebar-section">
                            <div class="php-sidebar-title">
                                <i class="fas fa-box"></i>
                                <span>"Load Management"</span>
                            </div>
                            <nav class="php-sidebar-links">
                                <A href="/loads" attr:class=move || sidebar_link_class(pathname().starts_with("/loads"))>
                                    <span class="php-sidebar-link-main">
                                        <i class="fas fa-truck"></i>
                                        <span>"My Loads"</span>
                                    </span>
                                    <i class="fas fa-chevron-right"></i>
                                </A>
                                {move || show_create_load_link.get().then(|| view! {
                                    <A href="/loads/new" attr:class=sidebar_link_class(pathname() == "/loads/new")>
                                        <span class="php-sidebar-link-main">
                                            <i class="fas fa-plus-circle"></i>
                                            <span>"Create Load"</span>
                                        </span>
                                        <i class="fas fa-chevron-right"></i>
                                    </A>
                                })}
                            </nav>
                        </section>

                        {move || show_dispatch_desk_link.get().then(|| view! {
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
                        })}

                        {move || show_stloads_links.get().then(|| view! {
                            <section class="php-sidebar-section">
                                <div class="php-sidebar-title">
                                    <i class="fas fa-broadcast-tower"></i>
                                    <span>"STLOADS Integration"</span>
                                </div>
                                <nav class="php-sidebar-links">
                                    <A href="/admin/stloads" attr:class=move || sidebar_link_class(pathname().starts_with("/admin/stloads") && !pathname().contains("reconciliation"))>
                                        <span class="php-sidebar-link-main">
                                            <i class="fas fa-radio"></i>
                                            <span>"STLOADS Ops"</span>
                                        </span>
                                        <i class="fas fa-chevron-right"></i>
                                    </A>
                                    <A href="/admin/stloads/reconciliation" attr:class=move || sidebar_link_class(pathname().contains("/reconciliation"))>
                                        <span class="php-sidebar-link-main">
                                            <i class="fas fa-code-branch"></i>
                                            <span>"Reconciliation"</span>
                                        </span>
                                        <i class="fas fa-chevron-right"></i>
                                    </A>
                                </nav>
                            </section>
                        })}

                        <section class="php-sidebar-section">
                            <div class="php-sidebar-title">
                                <i class="fas fa-comments"></i>
                                <span>"Communication"</span>
                            </div>
                            <nav class="php-sidebar-links">
                                <A href="/chat" attr:class=move || sidebar_link_class(pathname() == "/chat")>
                                    <span class="php-sidebar-link-main">
                                        <i class="fas fa-comment-dots"></i>
                                        <span>"Messages"</span>
                                    </span>
                                    <i class="fas fa-chevron-right"></i>
                                </A>
                            </nav>
                        </section>

                        <section class="php-sidebar-section">
                            <div class="php-sidebar-title">
                                <i class="fas fa-user"></i>
                                <span>"Account"</span>
                            </div>
                            <nav class="php-sidebar-links">
                                <A href="/profile" attr:class=move || sidebar_link_class(pathname() == "/profile")>
                                    <span class="php-sidebar-link-main">
                                        <i class="fas fa-id-badge"></i>
                                        <span>"My Profile"</span>
                                    </span>
                                    <i class="fas fa-chevron-right"></i>
                                </A>
                                {move || show_onboarding_link.get().then(|| view! {
                                    <A href="/auth/onboarding" attr:class=sidebar_link_class(pathname() == "/auth/onboarding")>
                                        <span class="php-sidebar-link-main">
                                            <i class="fas fa-clipboard-check"></i>
                                            <span>"Onboarding"</span>
                                        </span>
                                        <i class="fas fa-chevron-right"></i>
                                    </A>
                                })}
                                {move || show_admin_link.get().then(|| view! {
                                    <A href="/admin" attr:class=sidebar_link_class(pathname().starts_with("/admin"))>
                                        <span class="php-sidebar-link-main">
                                            <i class="fas fa-user-shield"></i>
                                            <span>"Admin"</span>
                                        </span>
                                        <i class="fas fa-chevron-right"></i>
                                    </A>
                                })}
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

fn user_quick_jump_path(
    query: &str,
    allow_admin: bool,
    allow_onboarding: bool,
    allow_create_load: bool,
    allow_dispatch: bool,
) -> Option<String> {
    let query = query.trim().to_ascii_lowercase();
    if query.is_empty() {
        return None;
    }

    if query.contains("dashboard") || query.contains("home") {
        return Some("/dashboard".into());
    }
    if query.contains("create") && query.contains("load") && allow_create_load {
        return Some("/loads/new".into());
    }
    if query.contains("load") {
        return Some("/loads".into());
    }
    if (query.contains("quote") || query.contains("desk")) && allow_dispatch {
        return Some("/desk/quote".into());
    }
    if query.contains("chat") || query.contains("message") {
        return Some("/chat".into());
    }
    if query.contains("profile") || query.contains("account") {
        return Some("/profile".into());
    }
    if query.contains("onboard") && allow_onboarding {
        return Some("/auth/onboarding".into());
    }
    if query.contains("admin") && allow_admin {
        return Some("/admin".into());
    }

    None
}
