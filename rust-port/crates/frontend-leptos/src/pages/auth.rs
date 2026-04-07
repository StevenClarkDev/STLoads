use leptos::{ev, prelude::*, task::spawn_local};
use leptos_router::hooks::use_navigate;

use crate::session::{self, use_auth};
use shared::LoginRequest;

#[component]
pub fn LoginPage() -> impl IntoView {
    let navigate = use_navigate();
    let auth = use_auth();
    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let is_submitting = RwSignal::new(false);

    let on_submit = move |ev: ev::SubmitEvent| {
        let navigate = navigate.clone();
        let auth = auth.clone();
        ev.prevent_default();
        let email_value = email.get().trim().to_string();
        let password_value = password.get();

        if email_value.is_empty() || password_value.is_empty() {
            auth.notice.set(Some(
                "Enter both email and password before signing in.".into(),
            ));
            return;
        }

        is_submitting.set(true);

        spawn_local(async move {
            let result = session::sign_in(
                auth.clone(),
                LoginRequest {
                    email: email_value,
                    password: password_value,
                },
            )
            .await;

            if let Ok(response) = result {
                if response.success {
                    let destination = response
                        .session
                        .user
                        .as_ref()
                        .map(|user| user.dashboard_href.clone())
                        .unwrap_or_else(|| "/".into());
                    navigate(&destination, Default::default());
                }
            }

            is_submitting.set(false);
        });
    };

    view! {
        <article style="display:grid;gap:1rem;max-width:540px;">
            <section>
                <h2>"Rust Login"</h2>
                <p>
                    "This form now writes into a single app-level Rust auth context, so the shell, load board, and chat screens all share the same session state."
                </p>
            </section>

            {move || auth.notice.get().map(|message| view! {
                <section style="padding:0.85rem 1rem;border:1px solid #dbeafe;border-radius:0.9rem;background:#eff6ff;color:#1d4ed8;">
                    {message}
                </section>
            })}

            {move || auth.session.get().user.map(|user| view! {
                <section style="padding:0.85rem 1rem;border:1px solid #dcfce7;border-radius:0.9rem;background:#f0fdf4;color:#166534;display:grid;gap:0.35rem;">
                    <strong>{format!("Authenticated as {}", user.name)}</strong>
                    <span>{format!("{} | {}", user.role_label, user.email)}</span>
                    <small>{user.account_status_label}</small>
                </section>
            })}

            <form on:submit=on_submit style="display:grid;gap:0.85rem;">
                <label style="display:grid;gap:0.35rem;">
                    <span>"Email"</span>
                    <input
                        type="email"
                        prop:value=move || email.get()
                        on:input=move |ev| email.set(event_target_value(&ev))
                        placeholder="name@example.com"
                        style="padding:0.8rem 1rem;border:1px solid #d1d5db;border-radius:0.85rem;"
                    />
                </label>

                <label style="display:grid;gap:0.35rem;">
                    <span>"Password"</span>
                    <input
                        type="password"
                        prop:value=move || password.get()
                        on:input=move |ev| password.set(event_target_value(&ev))
                        placeholder="Enter your password"
                        style="padding:0.8rem 1rem;border:1px solid #d1d5db;border-radius:0.85rem;"
                    />
                </label>

                <div style="display:flex;justify-content:flex-end;">
                    <button
                        type="submit"
                        style="padding:0.7rem 1rem;border:none;border-radius:0.85rem;background:#111827;color:white;cursor:pointer;"
                        disabled=move || is_submitting.get() || auth.session_loading.get()
                    >
                        {move || if is_submitting.get() || auth.session_loading.get() { "Signing in..." } else { "Sign in" }}
                    </button>
                </div>
            </form>
        </article>
    }
}
