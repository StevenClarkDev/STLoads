use leptos::{ev::SubmitEvent, prelude::*, tachys::view::any_view::IntoAny, task::spawn_local};

use crate::{
    api,
    session::{self, use_auth},
};
use shared::{AdminCreateUserRequest, AdminUserDirectoryScreen, AdminUserProfileScreen};

use super::admin_guard_view;

use super::admin_users_helpers::*;

#[component]
pub fn AdminUsersPage() -> impl IntoView {
    let auth = use_auth();
    let screen = RwSignal::new(None::<AdminUserDirectoryScreen>);
    let loading = RwSignal::new(false);
    let feedback = RwSignal::new(None::<String>);
    let search_query = RwSignal::new(String::new());
    let role_filter = RwSignal::new(String::from("all"));
    let action_loading_user_id = RwSignal::new(None::<u64>);
    let refresh_nonce = RwSignal::new(0_u64);
    let selected_profile = RwSignal::new(None::<AdminUserProfileScreen>);
    let profile_loading = RwSignal::new(false);
    let active_edit_user_id = RwSignal::new(None::<u64>);
    let confirm_delete_user_id = RwSignal::new(None::<u64>);

    let create_name = RwSignal::new(String::new());
    let create_email = RwSignal::new(String::new());
    let create_password = RwSignal::new(String::new());
    let create_password_confirmation = RwSignal::new(String::new());
    let create_role = RwSignal::new(String::from("shipper"));
    let create_status = RwSignal::new(String::from("pending_review"));
    let create_phone = RwSignal::new(String::new());
    let create_address = RwSignal::new(String::new());
    let create_loading = RwSignal::new(false);

    let can_view = Signal::derive(move || {
        session::has_permission(&auth, "access_admin_portal")
            || session::has_permission(&auth, "manage_users")
    });

    Effect::new(move |_| {
        let _refresh = refresh_nonce.get();
        if !auth.session_ready.get() || !auth.session.get().authenticated || !can_view.get() {
            return;
        }

        loading.set(true);
        let auth = auth;
        spawn_local(async move {
            match api::fetch_admin_user_directory().await {
                Ok(next) => {
                    screen.set(Some(next));
                    feedback.set(None);
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
            loading.set(false);
        });
    });

    let submit_create = move |ev: SubmitEvent| {
        ev.prevent_default();
        create_loading.set(true);

        let payload = AdminCreateUserRequest {
            name: create_name.get(),
            email: create_email.get(),
            password: create_password.get(),
            password_confirmation: create_password_confirmation.get(),
            role_key: create_role.get(),
            status_key: create_status.get(),
            phone_no: optional_string(create_phone.get()),
            address: optional_string(create_address.get()),
        };

        spawn_local(async move {
            match api::create_admin_user(&payload).await {
                Ok(response) => {
                    feedback.set(Some(response.message.clone()));
                    if response.success {
                        create_name.set(String::new());
                        create_email.set(String::new());
                        create_password.set(String::new());
                        create_password_confirmation.set(String::new());
                        create_phone.set(String::new());
                        create_address.set(String::new());
                        refresh_nonce.update(|value| *value += 1);
                    }
                }
                Err(error) => feedback.set(Some(error)),
            }
            create_loading.set(false);
        });
    };

    view! {
        {move || {
            if let Some(guard) = admin_guard_view(&auth, "User Directory", &["access_admin_portal", "manage_users"]) {
                guard
            } else {
                view! {
                    <article style="display:grid;gap:1rem;">
                        <section style="display:flex;justify-content:space-between;gap:1rem;flex-wrap:wrap;">
                            <div>
                                <h2>"User Directory"</h2>
                                <p>"Create accounts, inspect profile and KYC detail, and manage live user operations without falling back to PHP."</p>
                            </div>
                            <div style="display:grid;gap:0.5rem;min-width:300px;">
                                <input
                                    type="text"
                                    placeholder="Search by name, email, company, role, or status"
                                    prop:value=move || search_query.get()
                                    on:input=move |ev| search_query.set(event_target_value(&ev))
                                />
                                <select
                                    prop:value=move || role_filter.get()
                                    on:change=move |ev| role_filter.set(event_target_value(&ev))
                                >
                                    <option value="all">"All roles"</option>
                                    {move || screen.get().map(|screen_data| {
                                        screen_data.role_options.into_iter().map(|option| view! {
                                            <option value=option.key>{option.label}</option>
                                        }).collect_view()
                                    })}
                                </select>
                            </div>
                        </section>

                        <form on:submit=submit_create style="display:grid;gap:0.75rem;padding:1rem;border:1px solid #d6d3d1;border-radius:1rem;background:#fafaf9;">
                            <strong>"Create user"</strong>
                            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:0.75rem;">
                                <input type="text" placeholder="Name" prop:value=move || create_name.get() on:input=move |ev| create_name.set(event_target_value(&ev)) />
                                <input type="email" placeholder="Email" prop:value=move || create_email.get() on:input=move |ev| create_email.set(event_target_value(&ev)) />
                                <input type="text" placeholder="Phone" prop:value=move || create_phone.get() on:input=move |ev| create_phone.set(event_target_value(&ev)) />
                                <select prop:value=move || create_role.get() on:change=move |ev| create_role.set(event_target_value(&ev))>
                                    <option value="shipper">"Shipper"</option>
                                    <option value="carrier">"Carrier"</option>
                                    <option value="broker">"Broker"</option>
                                    <option value="freight_forwarder">"Freight Forwarder"</option>
                                    <option value="admin">"Admin"</option>
                                </select>
                                <select prop:value=move || create_status.get() on:change=move |ev| create_status.set(event_target_value(&ev))>
                                    <option value="pending_review">"Pending Review"</option>
                                    <option value="approved">"Approved"</option>
                                    <option value="email_verified">"Email Verified"</option>
                                    <option value="pending_otp">"Pending OTP"</option>
                                    <option value="revision_requested">"Revision Requested"</option>
                                    <option value="rejected">"Rejected"</option>
                                </select>
                                <input type="text" placeholder="Address" prop:value=move || create_address.get() on:input=move |ev| create_address.set(event_target_value(&ev)) />
                                <input type="password" placeholder="Password" prop:value=move || create_password.get() on:input=move |ev| create_password.set(event_target_value(&ev)) />
                                <input type="password" placeholder="Confirm password" prop:value=move || create_password_confirmation.get() on:input=move |ev| create_password_confirmation.set(event_target_value(&ev)) />
                            </div>
                            <div style="display:flex;justify-content:flex-end;">
                                <button type="submit" disabled=move || create_loading.get()>{move || if create_loading.get() { "Creating..." } else { "Create user" }}</button>
                            </div>
                        </form>

                        {move || if profile_loading.get() {
                            view! { <section>"Loading profile..."</section> }.into_any()
                        } else if let Some(profile) = selected_profile.get() {
                            render_profile_panel(
                                profile,
                                feedback,
                                action_loading_user_id,
                                refresh_nonce,
                                selected_profile,
                                profile_loading,
                            ).into_any()
                        } else {
                            ().into_any()
                        }}

                        {move || if loading.get() && screen.get().is_none() {
                            view! { <p>"Loading user directory from the Rust backend..."</p> }.into_any()
                        } else if let Some(screen_data) = screen.get() {
                            let role_options = screen_data.role_options.clone();
                            let status_options = screen_data.status_options.clone();
                            let query = search_query.get().to_ascii_lowercase();
                            let selected_role = role_filter.get();
                            let filtered_users = screen_data
                                .users
                                .into_iter()
                                .filter(|user| user_matches_query(user, &query))
                                .filter(|user| selected_role == "all" || user.role_key == selected_role)
                                .collect::<Vec<_>>();

                            view! {
                                <section style="display:grid;gap:1rem;">
                                    {filtered_users.into_iter().map(|user| {
                                        let role_options = role_options.clone();
                                        let status_options = status_options.clone();
                                        render_user_card(
                                            user,
                                            role_options,
                                            status_options,
                                            feedback,
                                            action_loading_user_id,
                                            refresh_nonce,
                                            selected_profile,
                                            profile_loading,
                                            active_edit_user_id,
                                            confirm_delete_user_id,
                                        )
                                    }).collect_view()}
                                </section>
                            }.into_any()
                        } else {
                            view! { <p>"No user directory data is available yet."</p> }.into_any()
                        }}
                    </article>
                }.into_any()
            }
        }}
    }
}
