use leptos::{prelude::*, tachys::view::any_view::IntoAny, task::spawn_local};
use leptos_router::components::A;

use crate::session::{self, use_auth};

#[component]
pub fn UserFrame(children: Children) -> impl IntoView {
    let auth = use_auth();

    let logout = move |_| {
        let auth = auth.clone();
        spawn_local(async move {
            let _ = session::sign_out(auth).await;
        });
    };

    let show_admin_link = Signal::derive(move || {
        session::has_permission(&auth, "access_admin_portal")
            || session::has_permission(&auth, "manage_tms_operations")
    });

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

    view! {
        <main class="app-shell user-frame">
            <header class="shell-header">
                <div class="shell-topbar">
                    <div class="shell-title-row">
                        <div class="brand-mark" aria-hidden="true">"ST"</div>
                        <div class="shell-brand-copy">
                            <p class="shell-kicker">"STLoads Network"</p>
                            <h1 class="shell-title">"Freight Portal"</h1>
                            <p class="shell-subtitle">
                                "Loads. Chat. Execution. Accounts."
                            </p>
                        </div>
                    </div>
                    <div class="session-card">
                        {move || auth.session.get().user.map(|user| view! {
                            <>
                                <strong class="session-name">{user.name}</strong>
                                <small>{format!("{} | {}", user.role_label, user.email)}</small>
                            </>
                        })}
                        {move || {
                            let session = auth.session.get();
                            if session.authenticated {
                                view! {
                                    <button
                                        type="button"
                                        class="shell-button"
                                        on:click=logout
                                        disabled=move || auth.session_loading.get()
                                    >
                                        {move || if auth.session_loading.get() { "Working..." } else { "Logout" }}
                                    </button>
                                }
                                .into_any()
                            } else if auth.session_ready.get() {
                                view! { <span class="session-pill">"Not signed in"</span> }.into_any()
                            } else {
                                view! { <span class="session-pill">"Loading session"</span> }.into_any()
                            }
                        }}
                    </div>
                </div>
                <nav class="shell-nav" aria-label="Primary">
                    <A href="/" attr:class="shell-nav-link">"Dashboard"</A>
                    <A href="/loads" attr:class="shell-nav-link">"Loads"</A>
                    {move || {
                        show_create_load_link.get().then(|| view! {
                            <A href="/loads/new" attr:class="shell-nav-link">"Create Load"</A>
                        })
                    }}
                    {move || {
                        show_dispatch_desk_link.get().then(|| view! {
                            <A href="/desk/quote" attr:class="shell-nav-link">"Dispatch Desk"</A>
                        })
                    }}
                    <A href="/profile" attr:class="shell-nav-link">"Profile"</A>
                    <A href="/chat" attr:class="shell-nav-link">"Chat"</A>
                    {move || {
                        show_onboarding_link.get().then(|| view! {
                            <A href="/auth/onboarding" attr:class="shell-nav-link">"Onboarding"</A>
                        })
                    }}
                    <A href="/auth/login" attr:class="shell-nav-link">"Auth"</A>
                    {move || {
                        show_admin_link.get().then(|| view! {
                            <A href="/admin" attr:class="shell-nav-link">"Admin"</A>
                        })
                    }}
                </nav>
                {move || auth.notice.get().map(|message| view! {
                    <small class="notice-pill">{message}</small>
                })}
            </header>
            <section>{children()}</section>
        </main>
    }
}
