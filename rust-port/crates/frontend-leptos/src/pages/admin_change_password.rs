use leptos::{ev::SubmitEvent, prelude::*, tachys::view::any_view::IntoAny, task::spawn_local};

use crate::{
    api,
    session::{self, use_auth},
};
use shared::ChangePasswordRequest;

use super::admin_guard_view;

#[component]
pub fn AdminChangePasswordPage() -> impl IntoView {
    let auth = use_auth();
    let current_password = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let password_confirmation = RwSignal::new(String::new());
    let feedback = RwSignal::new(None::<String>);
    let saving = RwSignal::new(false);

    let show_current = RwSignal::new(false);
    let show_password = RwSignal::new(false);
    let show_confirmation = RwSignal::new(false);

    view! {
        {move || {
            if let Some(guard) = admin_guard_view(&auth, "Change Password", &["access_admin_portal", "manage_users", "manage_roles", "manage_master_data", "manage_payments", "manage_tms_operations"]) {
                guard
            } else {
                let submit = move |ev: SubmitEvent| {
                    ev.prevent_default();
                    saving.set(true);
                    let auth = auth.clone();

                    let payload = ChangePasswordRequest {
                        current_password: current_password.get(),
                        password: password.get(),
                        password_confirmation: password_confirmation.get(),
                    };

                    spawn_local(async move {
                        match api::change_password(&payload).await {
                            Ok(response) => {
                                feedback.set(Some(response.message.clone()));
                                if response.success {
                                    current_password.set(String::new());
                                    password.set(String::new());
                                    password_confirmation.set(String::new());
                                }
                            }
                            Err(error) => {
                                if error.contains("returned 401") {
                                    session::invalidate_session(
                                        &auth,
                                        "Your Rust session expired; sign in again.",
                                    );
                                }
                                feedback.set(Some(error));
                            }
                        }

                        saving.set(false);
                    });
                };

                view! {
                    <article style="display:grid;gap:1rem;max-width:48rem;">
                        <section style="display:grid;gap:0.35rem;">
                            <h2>"Change Password"</h2>
                            <p>"This Rust admin security screen replaces the old standalone Blade password page with the same current-password confirmation flow."</p>
                        </section>

                        {move || feedback.get().map(|message| view! {
                            <section style="padding:0.85rem 1rem;border:1px solid #dbeafe;border-radius:0.9rem;background:#eff6ff;color:#1d4ed8;white-space:pre-wrap;">
                                {message}
                            </section>
                        })}

                        <form on:submit=submit style="display:grid;gap:1rem;padding:1rem;border:1px solid #e7e5e4;border-radius:1rem;background:#fff;">
                            <label style="display:grid;gap:0.4rem;">
                                <span>"Current password"</span>
                                <div style="display:grid;grid-template-columns:minmax(0,1fr) auto;gap:0.5rem;align-items:center;">
                                    <input
                                        type=move || if show_current.get() { "text" } else { "password" }
                                        placeholder="Enter your current password"
                                        prop:value=move || current_password.get()
                                        on:input=move |ev| current_password.set(event_target_value(&ev))
                                    />
                                    <button type="button" on:click=move |_| show_current.update(|value| *value = !*value)>
                                        {move || if show_current.get() { "Hide" } else { "Show" }}
                                    </button>
                                </div>
                            </label>

                            <label style="display:grid;gap:0.4rem;">
                                <span>"New password"</span>
                                <div style="display:grid;grid-template-columns:minmax(0,1fr) auto;gap:0.5rem;align-items:center;">
                                    <input
                                        type=move || if show_password.get() { "text" } else { "password" }
                                        placeholder="At least 8 characters"
                                        prop:value=move || password.get()
                                        on:input=move |ev| password.set(event_target_value(&ev))
                                    />
                                    <button type="button" on:click=move |_| show_password.update(|value| *value = !*value)>
                                        {move || if show_password.get() { "Hide" } else { "Show" }}
                                    </button>
                                </div>
                            </label>

                            <label style="display:grid;gap:0.4rem;">
                                <span>"Confirm new password"</span>
                                <div style="display:grid;grid-template-columns:minmax(0,1fr) auto;gap:0.5rem;align-items:center;">
                                    <input
                                        type=move || if show_confirmation.get() { "text" } else { "password" }
                                        placeholder="Repeat the new password"
                                        prop:value=move || password_confirmation.get()
                                        on:input=move |ev| password_confirmation.set(event_target_value(&ev))
                                    />
                                    <button type="button" on:click=move |_| show_confirmation.update(|value| *value = !*value)>
                                        {move || if show_confirmation.get() { "Hide" } else { "Show" }}
                                    </button>
                                </div>
                            </label>

                            <div style="display:flex;justify-content:flex-end;">
                                <button type="submit" disabled=move || saving.get()>
                                    {move || if saving.get() { "Updating..." } else { "Update password" }}
                                </button>
                            </div>
                        </form>
                    </article>
                }.into_any()
            }
        }}
    }
}
