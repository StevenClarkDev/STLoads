use leptos::{ev::SubmitEvent, prelude::*, tachys::view::any_view::IntoAny, task::spawn_local};

use crate::{
    api, document_upload,
    session::{self, use_auth},
};
use shared::{
    AdminCreateUserRequest, AdminUpdateUserProfileRequest, AdminUpdateUserRequest,
    AdminUserDirectoryScreen, AdminUserDirectoryUser, AdminUserProfileScreen, OtpPurpose,
    ResendOtpRequest, ReviewOnboardingRequest,
};

use super::admin_guard_view;

fn tone_style(tone: &str) -> &'static str {
    match tone {
        "success" => "background:#e8fff3;padding:0.25rem 0.6rem;border-radius:999px;color:#0f766e;",
        "warning" => "background:#fff7dd;padding:0.25rem 0.6rem;border-radius:999px;color:#b45309;",
        "danger" => "background:#ffe4e6;padding:0.25rem 0.6rem;border-radius:999px;color:#be123c;",
        "info" => "background:#e0f2fe;padding:0.25rem 0.6rem;border-radius:999px;color:#0369a1;",
        "primary" => "background:#ede9fe;padding:0.25rem 0.6rem;border-radius:999px;color:#6d28d9;",
        "dark" => "background:#e5e7eb;padding:0.25rem 0.6rem;border-radius:999px;color:#111827;",
        _ => "background:#f1f5f9;padding:0.25rem 0.6rem;border-radius:999px;color:#475569;",
    }
}

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
        let auth = auth.clone();
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

                        {move || feedback.get().map(|message| view! {
                            <section style="padding:0.85rem 1rem;border:1px solid #dbeafe;border-radius:0.9rem;background:#eff6ff;color:#1d4ed8;white-space:pre-wrap;">
                                {message}
                            </section>
                        })}

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
                            view! { <></> }.into_any()
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
                                    <strong>{screen_data.summary}</strong>
                                    {render_admin_user_attention_summary(selected_role.clone(), filtered_users.clone())}
                                    <section style="padding:0.85rem 1rem;border:1px solid #e5e7eb;border-radius:0.95rem;background:#fcfcfb;display:grid;gap:0.25rem;">
                                        <strong>"Queue guidance"</strong>
                                        <small style="color:#64748b;">{admin_user_role_guidance(&selected_role)}</small>
                                    </section>
                                    {screen_data.notes.into_iter().map(|note| view! { <p style="margin:0;">{note}</p> }).collect_view()}
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

fn render_admin_user_attention_summary(
    role_filter: String,
    users: Vec<AdminUserDirectoryUser>,
) -> impl IntoView {
    let pending_review = users
        .iter()
        .filter(|user| user.status_key == "pending_review")
        .count();
    let pending_otp = users
        .iter()
        .filter(|user| user.status_key == "pending_otp")
        .count();
    let revision_requested = users
        .iter()
        .filter(|user| user.status_key == "revision_requested")
        .count();
    let doc_gaps = users.iter().filter(|user| user.document_count == 0).count();
    let approved = users
        .iter()
        .filter(|user| user.status_key == "approved")
        .count();

    let cards = vec![
        (
            "Visible accounts",
            users.len().to_string(),
            "dark",
            "Accounts matching the current role filter and search query.",
        ),
        (
            "Pending review",
            pending_review.to_string(),
            if pending_review > 0 {
                "warning"
            } else {
                "success"
            },
            "Accounts that still need approve, reject, or revision action.",
        ),
        (
            "Pending OTP",
            pending_otp.to_string(),
            if pending_otp > 0 { "info" } else { "success" },
            "Accounts that have not completed OTP verification yet.",
        ),
        (
            "Needs revision",
            revision_requested.to_string(),
            if revision_requested > 0 {
                "warning"
            } else {
                "success"
            },
            "Profiles that were sent back to the user for updates.",
        ),
        (
            "No KYC docs",
            doc_gaps.to_string(),
            if doc_gaps > 0 { "danger" } else { "success" },
            "Accounts still missing uploaded KYC support files.",
        ),
        (
            "Approved",
            approved.to_string(),
            "primary",
            "Accounts already clear to use the Rust product surface.",
        ),
    ];

    view! {
        <section style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:0.75rem;">
            {cards.into_iter().map(|(label, value, tone, note)| {
                let badge_text = if role_filter == "all" && label == "Visible accounts" {
                    "all roles".to_string()
                } else {
                    tone.replace('_', " ")
                };
                view! {
                    <div style="padding:0.9rem 1rem;border:1px solid #e5e7eb;border-radius:0.95rem;background:#ffffff;display:grid;gap:0.3rem;">
                        <div style="display:flex;justify-content:space-between;gap:0.6rem;align-items:center;flex-wrap:wrap;">
                            <strong>{label}</strong>
                            <span style=tone_style(tone)>{badge_text}</span>
                        </div>
                        <div style="font-size:1.3rem;font-weight:700;color:#111827;">{value}</div>
                        <small style="color:#64748b;">{note}</small>
                    </div>
                }
            }).collect_view()}
        </section>
    }
}

fn admin_user_role_guidance(role_filter: &str) -> &'static str {
    match role_filter {
        "carrier" => {
            "Carrier accounts usually need DOT, MC, and KYC detail the most. Start with Pending Review and No KYC Docs before approving new carriers."
        }
        "shipper" => {
            "Shipper accounts should be checked for company detail and onboarding completeness first, then moved through approval or revision from the same Rust screen."
        }
        "broker" | "freight_forwarder" => {
            "Broker and freight-forwarder accounts typically need the most careful compliance review. Use the profile panel to inspect facts and KYC before approving."
        }
        "admin" => {
            "Admin accounts deserve the most caution. Confirm status, role, and password-related changes deliberately before saving directory edits."
        }
        _ => {
            "Start with Pending Review and Pending OTP, then work through revision-requested accounts and profiles that still have no KYC uploads."
        }
    }
}

fn admin_status_tone(status_key: &str) -> &'static str {
    match status_key {
        "approved" => "success",
        "pending_review" => "warning",
        "pending_otp" => "info",
        "revision_requested" => "warning",
        "rejected" => "danger",
        _ => "secondary",
    }
}

fn admin_profile_next_step(status_key: &str) -> &'static str {
    match status_key {
        "pending_otp" => {
            "This account is blocked before onboarding review. Send a fresh OTP first, then wait for the user to verify and continue."
        }
        "pending_review" => {
            "This account is ready for an admin decision. Review facts and KYC, then approve, reject, or request revision."
        }
        "revision_requested" => {
            "This account is waiting on the user's revised submission. Keep the review note clear so the next pass is faster."
        }
        "approved" => {
            "This account is already active. The main admin job now is profile upkeep and permissions accuracy."
        }
        "rejected" => {
            "This account is rejected in the current lifecycle. Only reopen it deliberately if the business decision changes."
        }
        _ => {
            "This account is in a non-standard state. Review the profile facts and history before making further changes."
        }
    }
}

fn admin_user_card_attention(user: &AdminUserDirectoryUser) -> Option<String> {
    let mut items = Vec::new();

    if user.document_count == 0 {
        items.push("KYC upload");
    }
    if user.phone_no.as_deref().unwrap_or("").trim().is_empty() {
        items.push("phone");
    }
    if user.company_name.as_deref().unwrap_or("").trim().is_empty() && user.role_key != "admin" {
        items.push("company");
    }

    if items.is_empty() {
        None
    } else {
        Some(format!("Still missing: {}.", items.join(", ")))
    }
}

fn profile_has_fact(facts: &[shared::AdminUserProfileFact], label: &str) -> bool {
    facts.iter().any(|fact| fact.label == label)
}

fn admin_profile_readiness_items(
    role_key: &str,
    status_key: &str,
    personal_facts: &[shared::AdminUserProfileFact],
    company_facts: &[shared::AdminUserProfileFact],
    document_count: usize,
) -> Vec<String> {
    let mut items = Vec::new();

    if !profile_has_fact(personal_facts, "Phone") {
        items.push("Phone number is still missing.".to_string());
    }
    if !profile_has_fact(personal_facts, "Address") {
        items.push("Address is still missing.".to_string());
    }
    if document_count == 0 {
        items.push("No KYC documents are attached yet.".to_string());
    }

    if role_key != "admin" {
        if !profile_has_fact(company_facts, "Company") {
            items.push("Company name is still missing.".to_string());
        }
        if !profile_has_fact(company_facts, "Company address") {
            items.push("Company address is still missing.".to_string());
        }
    }

    match role_key {
        "carrier" => {
            if !profile_has_fact(company_facts, "DOT number")
                && !profile_has_fact(company_facts, "USDOT")
            {
                items.push("Carrier profile still needs a DOT or USDOT number.".to_string());
            }
            if !profile_has_fact(company_facts, "MC number") {
                items.push("Carrier profile still needs an MC number.".to_string());
            }
        }
        "broker" | "freight_forwarder" => {
            if !profile_has_fact(company_facts, "MC number") {
                items.push(
                    "Broker or freight-forwarder profile still needs an MC number.".to_string(),
                );
            }
            if !profile_has_fact(company_facts, "Tax ID")
                && !profile_has_fact(company_facts, "Registration number")
            {
                items.push(
                    "Business compliance detail is thin here; tax ID or registration detail is still missing."
                        .to_string(),
                );
            }
        }
        _ => {}
    }

    match status_key {
        "pending_otp" => items.push(
            "OTP verification still has to complete before this account can move into a full onboarding review."
                .to_string(),
        ),
        "revision_requested" => items.push(
            "Revision was already requested, so the next clean admin move is to confirm the user actually fixed the missing items above."
                .to_string(),
        ),
        _ => {}
    }

    items
}

fn render_user_card(
    user: AdminUserDirectoryUser,
    role_options: Vec<shared::AdminUserDirectoryRoleOption>,
    status_options: Vec<shared::AdminUserDirectoryStatusOption>,
    feedback: RwSignal<Option<String>>,
    action_loading_user_id: RwSignal<Option<u64>>,
    refresh_nonce: RwSignal<u64>,
    selected_profile: RwSignal<Option<AdminUserProfileScreen>>,
    profile_loading: RwSignal<bool>,
    active_edit_user_id: RwSignal<Option<u64>>,
    confirm_delete_user_id: RwSignal<Option<u64>>,
) -> impl IntoView {
    let user_email = user.email.clone();
    let role_value = RwSignal::new(user.role_key.clone());
    let status_value = RwSignal::new(user.status_key.clone());
    let remarks_value = RwSignal::new(user.latest_review_note.clone().unwrap_or_default());

    let detail_name = RwSignal::new(user.name.clone());
    let detail_email = RwSignal::new(user.email.clone());
    let detail_phone = RwSignal::new(user.phone_no.clone().unwrap_or_default());
    let detail_address = RwSignal::new(String::new());
    let detail_password = RwSignal::new(String::new());
    let detail_password_confirmation = RwSignal::new(String::new());
    let detail_remarks = RwSignal::new(String::new());
    let can_run_review = matches!(
        user.status_key.as_str(),
        "pending_review" | "revision_requested"
    );
    let can_resend_otp = user.status_key == "pending_otp";

    let submit_account = move |ev: SubmitEvent| {
        ev.prevent_default();
        action_loading_user_id.set(Some(user.user_id));
        let payload = AdminUpdateUserRequest {
            role_key: role_value.get(),
            status_key: status_value.get(),
            remarks: optional_string(remarks_value.get()),
        };
        spawn_local(async move {
            match api::update_admin_user_account(user.user_id, &payload).await {
                Ok(response) => {
                    feedback.set(Some(response.message));
                    if response.success {
                        refresh_nonce.update(|value| *value += 1);
                    }
                }
                Err(error) => feedback.set(Some(error)),
            }
            action_loading_user_id.set(None);
        });
    };

    let save_details = move |ev: SubmitEvent| {
        ev.prevent_default();
        action_loading_user_id.set(Some(user.user_id));
        let payload = AdminUpdateUserProfileRequest {
            name: detail_name.get(),
            email: detail_email.get(),
            password: optional_string(detail_password.get()),
            password_confirmation: optional_string(detail_password_confirmation.get()),
            phone_no: optional_string(detail_phone.get()),
            address: optional_string(detail_address.get()),
            remarks: optional_string(detail_remarks.get()),
        };
        spawn_local(async move {
            match api::update_admin_user_profile(user.user_id, &payload).await {
                Ok(response) => {
                    feedback.set(Some(response.message));
                    if response.success {
                        active_edit_user_id.set(None);
                        refresh_nonce.update(|value| *value += 1);
                        profile_loading.set(true);
                        match api::fetch_admin_user_profile(user.user_id).await {
                            Ok(profile) => selected_profile.set(Some(profile)),
                            Err(error) => feedback.set(Some(error)),
                        }
                        profile_loading.set(false);
                    }
                }
                Err(error) => feedback.set(Some(error)),
            }
            action_loading_user_id.set(None);
        });
    };

    view! {
        <article style="padding:1rem;border:1px solid #d6d3d1;border-radius:1rem;background:#fcfcfb;display:grid;gap:0.85rem;">
            <div style="display:flex;justify-content:space-between;gap:1rem;flex-wrap:wrap;align-items:flex-start;">
                <div style="display:grid;gap:0.2rem;">
                    <strong>{format!("{} ({})", user.name, user.role_label)}</strong>
                    <span>{user.email.clone()}</span>
                    <small>{format!("{} | Joined {} | {} KYC document(s)", user.status_label, user.joined_at_label, user.document_count)}</small>
                    {user.company_name.clone().map(|value| view! { <small>{format!("Company: {}", value)}</small> })}
                    {user.phone_no.clone().map(|value| view! { <small>{format!("Phone: {}", value)}</small> })}
                    {admin_user_card_attention(&user).map(|message| view! {
                        <small style="color:#92400e;">{message}</small>
                    })}
                </div>
                <div style="display:flex;gap:0.5rem;flex-wrap:wrap;">
                    <button
                        type="button"
                        on:click=move |_| {
                            action_loading_user_id.set(Some(user.user_id));
                            profile_loading.set(true);
                            spawn_local(async move {
                                match api::fetch_admin_user_profile(user.user_id).await {
                                    Ok(profile) => {
                                        detail_address.set(profile.address.clone().unwrap_or_default());
                                        selected_profile.set(Some(profile));
                                    }
                                    Err(error) => feedback.set(Some(error)),
                                }
                                action_loading_user_id.set(None);
                                profile_loading.set(false);
                            });
                        }
                    >
                        "Profile"
                    </button>
                    <button type="button" on:click=move |_| {
                        if active_edit_user_id.get() == Some(user.user_id) {
                            active_edit_user_id.set(None);
                        } else {
                            active_edit_user_id.set(Some(user.user_id));
                        }
                    }>
                        {move || if active_edit_user_id.get() == Some(user.user_id) { "Close edit" } else { "Edit details" }}
                    </button>
                    {can_resend_otp.then(|| view! {
                        <button
                            type="button"
                            disabled=move || action_loading_user_id.get() == Some(user.user_id)
                            on:click=move |_| {
                                let resend_email = user_email.clone();
                                action_loading_user_id.set(Some(user.user_id));
                                spawn_local(async move {
                                    match api::resend_otp(&ResendOtpRequest {
                                        email: resend_email,
                                        purpose: OtpPurpose::Registration,
                                    }).await {
                                        Ok(response) => feedback.set(Some(response.message)),
                                        Err(error) => feedback.set(Some(error)),
                                    }
                                    action_loading_user_id.set(None);
                                });
                            }
                        >
                            "Resend OTP"
                        </button>
                    })}
                    <button
                        type="button"
                        disabled=move || action_loading_user_id.get() == Some(user.user_id)
                        on:click=move |_| {
                            if confirm_delete_user_id.get() == Some(user.user_id) {
                                action_loading_user_id.set(Some(user.user_id));
                                spawn_local(async move {
                                    match api::delete_admin_user(user.user_id).await {
                                        Ok(response) => {
                                            feedback.set(Some(response.message));
                                            if response.success {
                                                selected_profile.set(None);
                                                confirm_delete_user_id.set(None);
                                                refresh_nonce.update(|value| *value += 1);
                                            }
                                        }
                                        Err(error) => feedback.set(Some(error)),
                                    }
                                    action_loading_user_id.set(None);
                                });
                            } else {
                                confirm_delete_user_id.set(Some(user.user_id));
                            }
                        }
                    >
                        {move || if confirm_delete_user_id.get() == Some(user.user_id) { "Confirm delete" } else { "Delete" }}
                    </button>
                    {move || {
                        if confirm_delete_user_id.get() == Some(user.user_id)
                            && action_loading_user_id.get() != Some(user.user_id)
                        {
                            view! {
                                <button
                                    type="button"
                                    on:click=move |_| confirm_delete_user_id.set(None)
                                >
                                    "Cancel delete"
                                </button>
                            }.into_any()
                        } else {
                            view! { <></> }.into_any()
                        }
                    }}
                </div>
            </div>

            <form on:submit=submit_account style="display:grid;gap:0.75rem;">
                <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:0.75rem;">
                    <select prop:value=move || role_value.get() on:change=move |ev| role_value.set(event_target_value(&ev))>
                        {role_options.into_iter().map(|option| view! { <option value=option.key>{option.label}</option> }).collect_view()}
                    </select>
                    <select prop:value=move || status_value.get() on:change=move |ev| status_value.set(event_target_value(&ev))>
                        {status_options.into_iter().map(|option| view! { <option value=option.key>{option.label}</option> }).collect_view()}
                    </select>
                </div>
                <textarea rows="2" prop:value=move || remarks_value.get() on:input=move |ev| remarks_value.set(event_target_value(&ev)) placeholder="Admin remarks for role or status changes"></textarea>
                <div style="display:flex;justify-content:flex-end;">
                    <button type="submit" disabled=move || action_loading_user_id.get() == Some(user.user_id)>{move || if action_loading_user_id.get() == Some(user.user_id) { "Saving..." } else { "Save role and status" }}</button>
                </div>
            </form>

            {can_run_review.then(|| view! {
                <section style="display:grid;gap:0.55rem;padding-top:0.75rem;border-top:1px solid #e7e5e4;">
                    <strong>"Review shortcuts"</strong>
                    <small style="color:#64748b;">"This account is still in an onboarding-review state, so admin can approve, reject, or send it back for revision without leaving the directory."</small>
                    <div style="display:flex;gap:0.5rem;flex-wrap:wrap;">
                        <button
                            type="button"
                            disabled=move || action_loading_user_id.get() == Some(user.user_id)
                            on:click=move |_| run_directory_review_action(
                                user.user_id,
                                "approve",
                                optional_string(remarks_value.get()),
                                feedback,
                                action_loading_user_id,
                                refresh_nonce,
                                selected_profile,
                                profile_loading,
                            )
                            style="padding:0.5rem 0.75rem;border:none;border-radius:0.75rem;background:#166534;color:white;cursor:pointer;"
                        >
                            "Approve"
                        </button>
                        <button
                            type="button"
                            disabled=move || action_loading_user_id.get() == Some(user.user_id)
                            on:click=move |_| run_directory_review_action(
                                user.user_id,
                                "revision",
                                optional_string(remarks_value.get()).or_else(|| Some("Please revise and resubmit from the Rust onboarding flow.".into())),
                                feedback,
                                action_loading_user_id,
                                refresh_nonce,
                                selected_profile,
                                profile_loading,
                            )
                            style="padding:0.5rem 0.75rem;border:none;border-radius:0.75rem;background:#b45309;color:white;cursor:pointer;"
                        >
                            "Request revision"
                        </button>
                        <button
                            type="button"
                            disabled=move || action_loading_user_id.get() == Some(user.user_id)
                            on:click=move |_| run_directory_review_action(
                                user.user_id,
                                "reject",
                                optional_string(remarks_value.get()).or_else(|| Some("Rejected from the Rust admin user directory.".into())),
                                feedback,
                                action_loading_user_id,
                                refresh_nonce,
                                selected_profile,
                                profile_loading,
                            )
                            style="padding:0.5rem 0.75rem;border:none;border-radius:0.75rem;background:#be123c;color:white;cursor:pointer;"
                        >
                            "Reject"
                        </button>
                    </div>
                </section>
            })}

            {can_resend_otp.then(|| view! {
                <section style="display:grid;gap:0.55rem;padding-top:0.75rem;border-top:1px solid #e7e5e4;">
                    <strong>"Pending OTP support"</strong>
                    <small style="color:#64748b;">"This account is stuck before onboarding review. Admin can resend the registration OTP from the Rust directory instead of leaving the workflow here unfinished."</small>
                </section>
            })}

            {move || if active_edit_user_id.get() == Some(user.user_id) {
                view! {
                    <form on:submit=save_details style="display:grid;gap:0.75rem;padding-top:0.75rem;border-top:1px solid #e7e5e4;">
                        <strong>"Profile details"</strong>
                        <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:0.75rem;">
                            <input type="text" placeholder="Name" prop:value=move || detail_name.get() on:input=move |ev| detail_name.set(event_target_value(&ev)) />
                            <input type="email" placeholder="Email" prop:value=move || detail_email.get() on:input=move |ev| detail_email.set(event_target_value(&ev)) />
                            <input type="text" placeholder="Phone" prop:value=move || detail_phone.get() on:input=move |ev| detail_phone.set(event_target_value(&ev)) />
                            <input type="text" placeholder="Address" prop:value=move || detail_address.get() on:input=move |ev| detail_address.set(event_target_value(&ev)) />
                            <input type="password" placeholder="New password" prop:value=move || detail_password.get() on:input=move |ev| detail_password.set(event_target_value(&ev)) />
                            <input type="password" placeholder="Confirm new password" prop:value=move || detail_password_confirmation.get() on:input=move |ev| detail_password_confirmation.set(event_target_value(&ev)) />
                        </div>
                        <textarea rows="2" placeholder="Profile update note" prop:value=move || detail_remarks.get() on:input=move |ev| detail_remarks.set(event_target_value(&ev))></textarea>
                        <div style="display:flex;justify-content:flex-end;">
                            <button type="submit" disabled=move || action_loading_user_id.get() == Some(user.user_id)>{move || if action_loading_user_id.get() == Some(user.user_id) { "Saving..." } else { "Save profile" }}</button>
                        </div>
                    </form>
                }.into_any()
            } else {
                view! { <></> }.into_any()
            }}
        </article>
    }
}

fn render_profile_panel(
    profile: AdminUserProfileScreen,
    feedback: RwSignal<Option<String>>,
    action_loading_user_id: RwSignal<Option<u64>>,
    refresh_nonce: RwSignal<u64>,
    selected_profile: RwSignal<Option<AdminUserProfileScreen>>,
    profile_loading: RwSignal<bool>,
) -> impl IntoView {
    let profile_email = profile.email.clone();
    let personal_facts = profile.personal_facts.clone();
    let company_facts = profile.company_facts.clone();
    let documents = profile.documents.clone();
    let readiness_items = admin_profile_readiness_items(
        &profile.role_key,
        &profile.status_key,
        &personal_facts,
        &company_facts,
        documents.len(),
    );
    let review_note = RwSignal::new(String::new());
    let can_run_review = matches!(
        profile.status_key.as_str(),
        "pending_review" | "revision_requested"
    );
    let can_resend_otp = profile.status_key == "pending_otp";

    view! {
        <section style="padding:1rem;border:1px solid #cbd5e1;border-radius:1rem;background:#f8fafc;display:grid;gap:0.75rem;">
            <div>
                <h3 style="margin:0;">{profile.name.clone()}</h3>
                <p style="margin:0.2rem 0;">{format!("{} | {} | Joined {}", profile.email, profile.role_label, profile.joined_at_label)}</p>
                <small>{profile.status_label.clone()}</small>
            </div>
            <section style="padding:0.85rem 1rem;border:1px solid #e7e5e4;border-radius:0.9rem;background:#ffffff;display:grid;gap:0.35rem;">
                <div style="display:flex;justify-content:space-between;gap:0.75rem;align-items:center;flex-wrap:wrap;">
                    <strong>"Account next step"</strong>
                    <span style=tone_style(admin_status_tone(&profile.status_key))>{profile.status_label.clone()}</span>
                </div>
                <small style="color:#64748b;">{admin_profile_next_step(&profile.status_key)}</small>
            </section>
            {(!readiness_items.is_empty()).then(|| view! {
                <section style="display:grid;gap:0.55rem;padding:0.85rem 1rem;border:1px solid #fde68a;border-radius:0.9rem;background:#fffbeb;">
                    <strong>"Readiness gaps"</strong>
                    <small style="color:#92400e;">"These are the account details that still look incomplete from the Rust admin profile."</small>
                    <ul style="margin:0;padding-left:1.1rem;display:grid;gap:0.3rem;color:#92400e;">
                        {readiness_items.into_iter().map(|item| view! { <li>{item}</li> }).collect_view()}
                    </ul>
                </section>
            })}
            {can_run_review.then(|| view! {
                <section style="display:grid;gap:0.55rem;padding:0.85rem 1rem;border:1px solid #e7e5e4;border-radius:0.9rem;background:#ffffff;">
                    <strong>"Review account"</strong>
                    <small style="color:#64748b;">"The selected user is still in an onboarding review state. These shortcuts mirror the dedicated review queue without leaving this profile."</small>
                    <textarea
                        rows="2"
                        placeholder="Review note"
                        prop:value=move || review_note.get()
                        on:input=move |ev| review_note.set(event_target_value(&ev))
                    />
                    <div style="display:flex;gap:0.5rem;flex-wrap:wrap;">
                        <button
                            type="button"
                            disabled=move || action_loading_user_id.get() == Some(profile.user_id)
                            on:click=move |_| run_directory_review_action(
                                profile.user_id,
                                "approve",
                                optional_string(review_note.get()),
                                feedback,
                                action_loading_user_id,
                                refresh_nonce,
                                selected_profile,
                                profile_loading,
                            )
                            style="padding:0.5rem 0.75rem;border:none;border-radius:0.75rem;background:#166534;color:white;cursor:pointer;"
                        >
                            "Approve"
                        </button>
                        <button
                            type="button"
                            disabled=move || action_loading_user_id.get() == Some(profile.user_id)
                            on:click=move |_| run_directory_review_action(
                                profile.user_id,
                                "revision",
                                optional_string(review_note.get()).or_else(|| Some("Please revise and resubmit from the Rust onboarding flow.".into())),
                                feedback,
                                action_loading_user_id,
                                refresh_nonce,
                                selected_profile,
                                profile_loading,
                            )
                            style="padding:0.5rem 0.75rem;border:none;border-radius:0.75rem;background:#b45309;color:white;cursor:pointer;"
                        >
                            "Request revision"
                        </button>
                        <button
                            type="button"
                            disabled=move || action_loading_user_id.get() == Some(profile.user_id)
                            on:click=move |_| run_directory_review_action(
                                profile.user_id,
                                "reject",
                                optional_string(review_note.get()).or_else(|| Some("Rejected from the Rust admin profile view.".into())),
                                feedback,
                                action_loading_user_id,
                                refresh_nonce,
                                selected_profile,
                                profile_loading,
                            )
                            style="padding:0.5rem 0.75rem;border:none;border-radius:0.75rem;background:#be123c;color:white;cursor:pointer;"
                        >
                            "Reject"
                        </button>
                    </div>
                </section>
            })}
            {can_resend_otp.then(|| view! {
                <section style="display:grid;gap:0.55rem;padding:0.85rem 1rem;border:1px solid #e7e5e4;border-radius:0.9rem;background:#ffffff;">
                    <strong>"Pending OTP support"</strong>
                    <small style="color:#64748b;">"This account has not completed OTP verification yet, so it cannot progress into onboarding review. Admin can send a fresh registration OTP from here."</small>
                    <div style="display:flex;gap:0.5rem;flex-wrap:wrap;">
                        <button
                            type="button"
                            disabled=move || action_loading_user_id.get() == Some(profile.user_id)
                            on:click=move |_| {
                                let resend_email = profile_email.clone();
                                action_loading_user_id.set(Some(profile.user_id));
                                spawn_local(async move {
                                    match api::resend_otp(&ResendOtpRequest {
                                        email: resend_email,
                                        purpose: OtpPurpose::Registration,
                                    }).await {
                                        Ok(response) => feedback.set(Some(response.message)),
                                        Err(error) => feedback.set(Some(error)),
                                    }
                                    action_loading_user_id.set(None);
                                });
                            }
                            style="padding:0.5rem 0.75rem;border:none;border-radius:0.75rem;background:#0369a1;color:white;cursor:pointer;"
                        >
                            "Resend registration OTP"
                        </button>
                    </div>
                </section>
            })}
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(260px,1fr));gap:1rem;">
                <div>
                    <strong>"Personal facts"</strong>
                    {personal_facts.into_iter().map(|fact| view! { <p style="margin:0.2rem 0;"><strong>{fact.label}</strong>" : "{fact.value}</p> }).collect_view()}
                </div>
                <div>
                    <strong>"Company facts"</strong>
                    {company_facts.into_iter().map(|fact| view! { <p style="margin:0.2rem 0;"><strong>{fact.label}</strong>" : "{fact.value}</p> }).collect_view()}
                </div>
            </div>
            <div>
                <strong>"KYC documents"</strong>
                {if documents.is_empty() {
                    view! { <p>"No KYC documents are attached yet."</p> }.into_any()
                } else {
                    documents.into_iter().map(|document| {
                        let download_path = document.download_path.clone();
                        view! {
                            <div style="display:flex;justify-content:space-between;gap:1rem;flex-wrap:wrap;padding:0.65rem 0;border-bottom:1px solid #e7e5e4;">
                                <div>
                                    <strong>{document.document_name}</strong>
                                    <p style="margin:0.15rem 0;">{document.original_name.unwrap_or_else(|| "Unnamed file".into())}</p>
                                    <small>{format!("{} | {}", document.document_type, document.uploaded_at_label)}</small>
                                </div>
                                {download_path.map(|path| view! {
                                    <button type="button" on:click=move |_| {
                                        let path = path.clone();
                                        spawn_local(async move {
                                            if let Err(error) = document_upload::open_protected_document(&path).await {
                                                feedback.set(Some(error));
                                            }
                                        });
                                    }>"View file"</button>
                                })}
                            </div>
                        }
                    }).collect_view().into_any()
                }}
            </div>
            <div>
                <strong>"History"</strong>
                {profile.history.into_iter().map(|item| view! {
                    <div style="padding:0.65rem 0;border-bottom:1px solid #e7e5e4;">
                        <strong>{item.status_label}</strong>
                        <small style="display:block;">{item.created_at_label}</small>
                        {item.admin_name.map(|name| view! { <small style="display:block;">{format!("By {}", name)}</small> })}
                        {item.remarks.map(|remarks| view! { <p style="margin:0.25rem 0 0;">{remarks}</p> })}
                    </div>
                }).collect_view()}
            </div>
        </section>
    }
}

fn user_matches_query(user: &AdminUserDirectoryUser, query: &str) -> bool {
    if query.trim().is_empty() {
        return true;
    }
    [
        user.name.as_str(),
        user.email.as_str(),
        user.role_label.as_str(),
        user.status_label.as_str(),
        user.company_name.as_deref().unwrap_or_default(),
        user.phone_no.as_deref().unwrap_or_default(),
    ]
    .into_iter()
    .any(|value| value.to_ascii_lowercase().contains(query))
}

fn optional_string(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn run_directory_review_action(
    user_id: u64,
    decision: &'static str,
    remarks: Option<String>,
    feedback: RwSignal<Option<String>>,
    action_loading_user_id: RwSignal<Option<u64>>,
    refresh_nonce: RwSignal<u64>,
    selected_profile: RwSignal<Option<AdminUserProfileScreen>>,
    profile_loading: RwSignal<bool>,
) {
    action_loading_user_id.set(Some(user_id));
    spawn_local(async move {
        let result = api::review_onboarding_user(
            user_id,
            &ReviewOnboardingRequest {
                decision: decision.into(),
                remarks,
            },
        )
        .await;

        match result {
            Ok(response) => {
                feedback.set(Some(response.message));
                if response.success {
                    refresh_nonce.update(|value| *value += 1);
                    if selected_profile
                        .get_untracked()
                        .as_ref()
                        .map(|profile| profile.user_id == user_id)
                        .unwrap_or(false)
                    {
                        profile_loading.set(true);
                        match api::fetch_admin_user_profile(user_id).await {
                            Ok(profile) => selected_profile.set(Some(profile)),
                            Err(error) => feedback.set(Some(error)),
                        }
                        profile_loading.set(false);
                    }
                }
            }
            Err(error) => feedback.set(Some(error)),
        }

        action_loading_user_id.set(None);
    });
}
