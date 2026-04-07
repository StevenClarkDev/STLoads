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

    view! {
        <main class="user-frame">
            <header style="display:grid;gap:0.75rem;">
                <div style="display:flex;justify-content:space-between;gap:1rem;align-items:flex-start;flex-wrap:wrap;">
                    <div>
                        <p>"STLoads User Shell"</p>
                        <h1>"User Portal"</h1>
                    </div>
                    <div style="display:grid;gap:0.35rem;text-align:right;">
                        {move || auth.session.get().user.map(|user| view! {
                            <>
                                <strong>{user.name}</strong>
                                <small>{format!("{} | {}", user.role_label, user.email)}</small>
                            </>
                        })}
                        {move || {
                            let session = auth.session.get();
                            if session.authenticated {
                                view! {
                                    <button
                                        type="button"
                                        on:click=logout
                                        style="padding:0.45rem 0.8rem;border-radius:0.75rem;border:1px solid #111827;background:#111827;color:white;cursor:pointer;"
                                        disabled=move || auth.session_loading.get()
                                    >
                                        {move || if auth.session_loading.get() { "Working..." } else { "Logout" }}
                                    </button>
                                }
                                .into_any()
                            } else if auth.session_ready.get() {
                                view! { <small>"Not signed in"</small> }.into_any()
                            } else {
                                view! { <small>"Loading session..."</small> }.into_any()
                            }
                        }}
                    </div>
                </div>
                <nav>
                    <A href="/">"Dashboard"</A>
                    " | "
                    <A href="/loads">"Loads"</A>
                    {move || {
                        show_create_load_link.get().then(|| view! {
                            <>
                                " | "
                                <A href="/loads/new">"Create Load"</A>
                            </>
                        })
                    }}
                    " | "
                    <A href="/chat">"Chat"</A>
                    " | "
                    <A href="/auth/login">"Auth"</A>
                    {move || {
                        show_admin_link.get().then(|| view! {
                            <>
                                " | "
                                <A href="/admin">"Admin"</A>
                            </>
                        })
                    }}
                </nav>
                {move || auth.notice.get().map(|message| view! {
                    <small style="color:#475569;">{message}</small>
                })}
            </header>
            <section>{children()}</section>
        </main>
    }
}
