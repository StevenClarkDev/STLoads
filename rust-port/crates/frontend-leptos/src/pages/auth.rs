use leptos::{ev, prelude::*, task::spawn_local};
use leptos_router::{components::A, hooks::use_navigate};

use crate::{
    api, document_upload,
    session::{self, use_auth},
};
use shared::{
    AuthOnboardingScreen, ForgotPasswordRequest, LoginRequest, OtpPurpose, RegisterRequest,
    ResetPasswordRequest, SubmitOnboardingRequest, VerifyOtpRequest,
};

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
        <AuthArticle
            title="Rust Login"
            subtitle="The Rust session context is now shared across the app shell, load board, chat, and admin screens."
        >
            <SharedNotice />

            {move || auth.session.get().user.map(|user| view! {
                <section style="padding:0.85rem 1rem;border:1px solid #dcfce7;border-radius:0.9rem;background:#f0fdf4;color:#166534;display:grid;gap:0.35rem;">
                    <strong>{format!("Authenticated as {}", user.name)}</strong>
                    <span>{format!("{} | {}", user.role_label, user.email)}</span>
                    <small>{user.account_status_label}</small>
                </section>
            })}

            <form on:submit=on_submit style="display:grid;gap:0.85rem;">
                <TextField label="Email" value=email input_type="email" placeholder="name@example.com" />
                <TextField label="Password" value=password input_type="password" placeholder="Enter your password" />

                <div style="display:flex;justify-content:space-between;align-items:center;gap:0.75rem;flex-wrap:wrap;">
                    <nav style="display:flex;gap:0.8rem;flex-wrap:wrap;font-size:0.95rem;">
                        <A href="/auth/register">"Create account"</A>
                        <A href="/auth/forgot-password">"Forgot password"</A>
                    </nav>
                    <button
                        type="submit"
                        style=button_style("#111827")
                        disabled=move || is_submitting.get() || auth.session_loading.get()
                    >
                        {move || if is_submitting.get() || auth.session_loading.get() { "Signing in..." } else { "Sign in" }}
                    </button>
                </div>
            </form>
        </AuthArticle>
    }
}

#[component]
pub fn RegisterPage() -> impl IntoView {
    let navigate = use_navigate();
    let auth = use_auth();
    let name = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let phone = RwSignal::new(String::new());
    let address = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let password_confirmation = RwSignal::new(String::new());
    let role = RwSignal::new("shipper".to_string());
    let feedback = RwSignal::new(None::<String>);
    let is_submitting = RwSignal::new(false);

    let on_submit = move |ev: ev::SubmitEvent| {
        let navigate = navigate.clone();
        let auth = auth.clone();
        ev.prevent_default();
        feedback.set(None);

        let payload = RegisterRequest {
            name: name.get().trim().to_string(),
            email: email.get().trim().to_string(),
            password: password.get(),
            password_confirmation: password_confirmation.get(),
            role_key: role.get(),
            phone_no: optional_string(phone.get()),
            address: optional_string(address.get()),
        };

        if payload.name.is_empty() || payload.email.is_empty() {
            feedback.set(Some(
                "Name and email are required before registration.".into(),
            ));
            return;
        }

        is_submitting.set(true);
        spawn_local(async move {
            match api::register(&payload).await {
                Ok(response) => {
                    let mut message = response.message;
                    if let Some(dev_otp) = response.dev_otp {
                        message.push_str(&format!(" Dev OTP: {}", dev_otp));
                    }
                    auth.notice.set(Some(message.clone()));
                    feedback.set(Some(message));
                    if response.success {
                        navigate("/auth/verify-otp", Default::default());
                    }
                }
                Err(error) => {
                    auth.notice.set(Some(error.clone().to_string()));
                    feedback.set(Some(error.to_string()));
                }
            }
            is_submitting.set(false);
        });
    };

    view! {
        <AuthArticle
            title="Rust Registration"
            subtitle="This is the first self-serve replacement for the Laravel register plus OTP flow."
        >
            <LocalNotice message=feedback />
            <form on:submit=on_submit style="display:grid;gap:0.85rem;">
                <TextField label="Full name" value=name input_type="text" placeholder="Your full name" />
                <TextField label="Email" value=email input_type="email" placeholder="name@example.com" />
                <TextField label="Phone" value=phone input_type="tel" placeholder="Optional contact number" />
                <TextAreaField label="Address" value=address placeholder="Business or profile address" />
                <RoleField value=role />
                <TextField label="Password" value=password input_type="password" placeholder="Create a password" />
                <TextField
                    label="Confirm password"
                    value=password_confirmation
                    input_type="password"
                    placeholder="Repeat the password"
                />
                <div style="display:flex;justify-content:space-between;align-items:center;gap:0.75rem;flex-wrap:wrap;">
                    <nav style="display:flex;gap:0.8rem;flex-wrap:wrap;font-size:0.95rem;">
                        <A href="/auth/login">"Back to login"</A>
                        <A href="/auth/verify-otp">"Already have an OTP?"</A>
                    </nav>
                    <button type="submit" style=button_style("#0f766e") disabled=move || is_submitting.get()>
                        {move || if is_submitting.get() { "Creating account..." } else { "Create account" }}
                    </button>
                </div>
            </form>
        </AuthArticle>
    }
}

#[component]
pub fn VerifyOtpPage() -> impl IntoView {
    let navigate = use_navigate();
    let auth = use_auth();
    let email = RwSignal::new(String::new());
    let otp = RwSignal::new(String::new());
    let purpose = RwSignal::new(OtpPurpose::Registration);
    let reset_token = RwSignal::new(String::new());
    let feedback = RwSignal::new(None::<String>);
    let is_submitting = RwSignal::new(false);
    let is_resending = RwSignal::new(false);

    let on_submit = move |ev: ev::SubmitEvent| {
        let navigate = navigate.clone();
        let auth = auth.clone();
        ev.prevent_default();
        feedback.set(None);

        let payload = VerifyOtpRequest {
            email: email.get().trim().to_string(),
            otp: otp.get().trim().to_string(),
            purpose: purpose.get(),
        };

        if payload.email.is_empty() || payload.otp.is_empty() {
            feedback.set(Some(
                "Email and OTP are required before verification.".into(),
            ));
            return;
        }

        is_submitting.set(true);
        spawn_local(async move {
            match api::verify_otp(&payload).await {
                Ok(response) => {
                    let mut message = response.message;
                    if let Some(token) = response.reset_token.clone() {
                        reset_token.set(token.clone());
                        message.push_str(&format!(" Reset token: {}", token));
                    }
                    auth.notice.set(Some(message.clone()));
                    feedback.set(Some(message));
                    if let Some(session_state) = response.session.clone() {
                        auth.session.set(session_state);
                        auth.session_ready.set(true);
                    }
                    if response.success {
                        navigate(&response.next_step, Default::default());
                    }
                }
                Err(error) => {
                    auth.notice.set(Some(error.clone().to_string()));
                    feedback.set(Some(error.to_string()));
                }
            }
            is_submitting.set(false);
        });
    };

    let on_resend = move |_| {
        let auth = auth.clone();
        let email_value = email.get().trim().to_string();
        if email_value.is_empty() {
            feedback.set(Some(
                "Enter the email address first so Rust knows where to resend the OTP.".into(),
            ));
            return;
        }

        is_resending.set(true);
        let purpose_value = purpose.get();
        spawn_local(async move {
            match api::resend_otp(&shared::ResendOtpRequest {
                email: email_value,
                purpose: purpose_value,
            })
            .await
            {
                Ok(response) => {
                    let mut message = response.message;
                    if let Some(dev_otp) = response.dev_otp {
                        message.push_str(&format!(" Dev OTP: {}", dev_otp));
                    }
                    auth.notice.set(Some(message.clone()));
                    feedback.set(Some(message));
                }
                Err(error) => {
                    auth.notice.set(Some(error.clone().to_string()));
                    feedback.set(Some(error.to_string()));
                }
            }
            is_resending.set(false);
        });
    };

    view! {
        <AuthArticle
            title="Verify OTP"
            subtitle="Use the same screen for new-account OTP verification and password-reset OTP verification."
        >
            <LocalNotice message=feedback />
            <form on:submit=on_submit style="display:grid;gap:0.85rem;">
                <TextField label="Email" value=email input_type="email" placeholder="name@example.com" />
                <OtpPurposeField value=purpose />
                <TextField label="OTP" value=otp input_type="text" placeholder="6-digit code" />
                {move || if reset_token.get().is_empty() {
                    view! { <></> }.into_any()
                } else {
                    view! {
                        <section style="padding:0.75rem 0.9rem;border:1px solid #dbeafe;border-radius:0.9rem;background:#eff6ff;color:#1d4ed8;display:grid;gap:0.25rem;">
                            <strong>"Development reset token"</strong>
                            <code style="font-size:0.9rem;word-break:break-all;">{move || reset_token.get()}</code>
                        </section>
                    }.into_any()
                }}
                <div style="display:flex;justify-content:space-between;align-items:center;gap:0.75rem;flex-wrap:wrap;">
                    <nav style="display:flex;gap:0.8rem;flex-wrap:wrap;font-size:0.95rem;">
                        <A href="/auth/register">"Create account"</A>
                        <A href="/auth/forgot-password">"Forgot password"</A>
                    </nav>
                    <div style="display:flex;gap:0.75rem;flex-wrap:wrap;">
                        <button type="button" on:click=on_resend style=button_style("#475569") disabled=move || is_resending.get()>
                            {move || if is_resending.get() { "Resending..." } else { "Resend OTP" }}
                        </button>
                        <button type="submit" style=button_style("#111827") disabled=move || is_submitting.get()>
                            {move || if is_submitting.get() { "Verifying..." } else { "Verify OTP" }}
                        </button>
                    </div>
                </div>
            </form>
        </AuthArticle>
    }
}

#[component]
pub fn ForgotPasswordPage() -> impl IntoView {
    let navigate = use_navigate();
    let auth = use_auth();
    let email = RwSignal::new(String::new());
    let feedback = RwSignal::new(None::<String>);
    let is_submitting = RwSignal::new(false);

    let on_submit = move |ev: ev::SubmitEvent| {
        let navigate = navigate.clone();
        let auth = auth.clone();
        ev.prevent_default();
        feedback.set(None);
        let payload = ForgotPasswordRequest {
            email: email.get().trim().to_string(),
        };

        if payload.email.is_empty() {
            feedback.set(Some(
                "Enter the account email before requesting a password reset OTP.".into(),
            ));
            return;
        }

        is_submitting.set(true);
        spawn_local(async move {
            match api::forgot_password(&payload).await {
                Ok(response) => {
                    let mut message = response.message;
                    if let Some(dev_otp) = response.dev_otp {
                        message.push_str(&format!(" Dev OTP: {}", dev_otp));
                    }
                    auth.notice.set(Some(message.clone()));
                    feedback.set(Some(message));
                    if response.success {
                        navigate("/auth/verify-otp", Default::default());
                    }
                }
                Err(error) => {
                    auth.notice.set(Some(error.clone().to_string()));
                    feedback.set(Some(error.to_string()));
                }
            }
            is_submitting.set(false);
        });
    };

    view! {
        <AuthArticle
            title="Forgot Password"
            subtitle="This replaces the legacy forgot-password entry point with the Rust OTP-first reset flow."
        >
            <LocalNotice message=feedback />
            <form on:submit=on_submit style="display:grid;gap:0.85rem;">
                <TextField label="Email" value=email input_type="email" placeholder="name@example.com" />
                <div style="display:flex;justify-content:space-between;align-items:center;gap:0.75rem;flex-wrap:wrap;">
                    <nav style="display:flex;gap:0.8rem;flex-wrap:wrap;font-size:0.95rem;">
                        <A href="/auth/login">"Back to login"</A>
                        <A href="/auth/verify-otp">"Already have an OTP?"</A>
                    </nav>
                    <button type="submit" style=button_style("#7c3aed") disabled=move || is_submitting.get()>
                        {move || if is_submitting.get() { "Sending OTP..." } else { "Send reset OTP" }}
                    </button>
                </div>
            </form>
        </AuthArticle>
    }
}

#[component]
pub fn ResetPasswordPage() -> impl IntoView {
    let navigate = use_navigate();
    let auth = use_auth();
    let email = RwSignal::new(String::new());
    let reset_token = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let password_confirmation = RwSignal::new(String::new());
    let feedback = RwSignal::new(None::<String>);
    let is_submitting = RwSignal::new(false);

    let on_submit = move |ev: ev::SubmitEvent| {
        let navigate = navigate.clone();
        let auth = auth.clone();
        ev.prevent_default();
        feedback.set(None);
        let payload = ResetPasswordRequest {
            email: email.get().trim().to_string(),
            reset_token: reset_token.get().trim().to_string(),
            password: password.get(),
            password_confirmation: password_confirmation.get(),
        };

        if payload.email.is_empty() || payload.reset_token.is_empty() {
            feedback.set(Some(
                "Email and reset token are required before setting a new password.".into(),
            ));
            return;
        }

        is_submitting.set(true);
        spawn_local(async move {
            match api::reset_password(&payload).await {
                Ok(response) => {
                    auth.notice.set(Some(response.message.clone()));
                    feedback.set(Some(response.message.clone()));
                    if response.success {
                        navigate("/auth/login", Default::default());
                    }
                }
                Err(error) => {
                    auth.notice.set(Some(error.clone().to_string()));
                    feedback.set(Some(error.to_string()));
                }
            }
            is_submitting.set(false);
        });
    };

    view! {
        <AuthArticle
            title="Reset Password"
            subtitle="Complete the Rust password reset after OTP verification by setting a fresh password here."
        >
            <LocalNotice message=feedback />
            <form on:submit=on_submit style="display:grid;gap:0.85rem;">
                <TextField label="Email" value=email input_type="email" placeholder="name@example.com" />
                <TextAreaField label="Reset token" value=reset_token placeholder="Paste the reset token from the verification step" />
                <TextField label="New password" value=password input_type="password" placeholder="New password" />
                <TextField label="Confirm new password" value=password_confirmation input_type="password" placeholder="Repeat the new password" />
                <div style="display:flex;justify-content:space-between;align-items:center;gap:0.75rem;flex-wrap:wrap;">
                    <nav style="display:flex;gap:0.8rem;flex-wrap:wrap;font-size:0.95rem;">
                        <A href="/auth/verify-otp">"Back to OTP verify"</A>
                        <A href="/auth/login">"Back to login"</A>
                    </nav>
                    <button type="submit" style=button_style("#111827") disabled=move || is_submitting.get()>
                        {move || if is_submitting.get() { "Updating..." } else { "Update password" }}
                    </button>
                </div>
            </form>
        </AuthArticle>
    }
}

#[component]
pub fn OnboardingPage() -> impl IntoView {
    let navigate = use_navigate();
    let auth = use_auth();
    let screen = RwSignal::new(None::<AuthOnboardingScreen>);
    let feedback = RwSignal::new(None::<String>);
    let loading = RwSignal::new(false);
    let saving = RwSignal::new(false);

    let company_name = RwSignal::new(String::new());
    let company_address = RwSignal::new(String::new());
    let dot_number = RwSignal::new(String::new());
    let mc_number = RwSignal::new(String::new());
    let equipment_types = RwSignal::new(String::new());
    let business_entity_id = RwSignal::new(String::new());
    let facility_address = RwSignal::new(String::new());
    let fulfillment_contact_info = RwSignal::new(String::new());
    let fmcsa_broker_license_no = RwSignal::new(String::new());
    let mc_authority_number = RwSignal::new(String::new());
    let freight_forwarder_license = RwSignal::new(String::new());
    let customs_license = RwSignal::new(String::new());
    let kyc_document_name = RwSignal::new(String::new());
    let kyc_document_type = RwSignal::new("standard".to_string());
    let upload_in_progress = RwSignal::new(false);
    let refresh_nonce = RwSignal::new(0_u64);

    let load_screen = move || {
        let auth = auth.clone();
        loading.set(true);
        spawn_local(async move {
            match api::fetch_onboarding_screen().await {
                Ok(response) => {
                    company_name.set(response.draft.company_name.clone().unwrap_or_default());
                    company_address.set(response.draft.company_address.clone().unwrap_or_default());
                    dot_number.set(response.draft.dot_number.clone().unwrap_or_default());
                    mc_number.set(response.draft.mc_number.clone().unwrap_or_default());
                    equipment_types.set(response.draft.equipment_types.clone().unwrap_or_default());
                    business_entity_id.set(
                        response
                            .draft
                            .business_entity_id
                            .clone()
                            .unwrap_or_default(),
                    );
                    facility_address
                        .set(response.draft.facility_address.clone().unwrap_or_default());
                    fulfillment_contact_info.set(
                        response
                            .draft
                            .fulfillment_contact_info
                            .clone()
                            .unwrap_or_default(),
                    );
                    fmcsa_broker_license_no.set(
                        response
                            .draft
                            .fmcsa_broker_license_no
                            .clone()
                            .unwrap_or_default(),
                    );
                    mc_authority_number.set(
                        response
                            .draft
                            .mc_authority_number
                            .clone()
                            .unwrap_or_default(),
                    );
                    freight_forwarder_license.set(
                        response
                            .draft
                            .freight_forwarder_license
                            .clone()
                            .unwrap_or_default(),
                    );
                    customs_license.set(response.draft.customs_license.clone().unwrap_or_default());
                    if !response.notes.is_empty() {
                        feedback.set(Some(response.notes.join("\n")));
                    }
                    screen.set(Some(response));
                }
                Err(error) => {
                    auth.notice.set(Some(error.clone().to_string()));
                    feedback.set(Some(error.to_string()));
                }
            }
            loading.set(false);
        });
    };

    Effect::new(move |_| {
        let _refresh = refresh_nonce.get();
        if auth.session_ready.get()
            && auth.session.get().authenticated
            && screen.get().is_none()
            && !loading.get()
        {
            load_screen();
        }
    });

    view! {
        <AuthArticle
            title="Rust Onboarding"
            subtitle="OTP-complete accounts continue here until the company profile is submitted for review."
        >
            <LocalNotice message=feedback />
            {move || if !auth.session_ready.get() || loading.get() {
                view! { <p>"Loading Rust onboarding..."</p> }.into_any()
            } else if !auth.session.get().authenticated {
                view! {
                    <section style="display:grid;gap:0.75rem;">
                        <p>"Sign in first, then continue the onboarding flow from this page."</p>
                        <A href="/auth/login">"Go to login"</A>
                    </section>
                }.into_any()
            } else if let Some(screen_state) = screen.get() {
                let can_submit = screen_state.can_submit;
                let status_label = screen_state.status_label.clone();
                let requires_otp = screen_state.requires_otp;
                let is_carrier = screen_state.role_key == "carrier";
                let is_shipper = screen_state.role_key == "shipper";
                let is_broker = screen_state.role_key == "broker";
                let is_forwarder = screen_state.role_key == "freight_forwarder";
                let submit_navigate = navigate.clone();
                let submit_auth = auth.clone();
                view! {
                    <>
                        <section style="padding:0.85rem 1rem;border:1px solid #e5e7eb;border-radius:0.9rem;background:#f8fafc;display:grid;gap:0.35rem;">
                            <strong>{format!("{} onboarding", screen_state.role_label)}</strong>
                            <span>{format!("Status: {}", status_label)}</span>
                            <small>{if requires_otp { "OTP is still required before this form can be submitted." } else { "OTP continuity is satisfied for this account." }}</small>
                        </section>
                        <section style="display:grid;gap:0.75rem;padding:0.9rem;border:1px solid #dbeafe;border-radius:0.9rem;background:#f8fbff;">
                            <strong>"KYC Documents"</strong>
                            <div style="display:grid;grid-template-columns:2fr 1fr auto;gap:0.65rem;align-items:end;">
                                <TextField label="Document name" value=kyc_document_name input_type="text" placeholder="Government ID, proof of address, insurance" />
                                <label style="display:grid;gap:0.35rem;">
                                    <span>"Document type"</span>
                                    <select prop:value=move || kyc_document_type.get() on:change=move |ev| kyc_document_type.set(event_target_value(&ev)) style="padding:0.8rem 1rem;border:1px solid #d1d5db;border-radius:0.85rem;background:white;">
                                        <option value="standard">"Standard"</option>
                                        <option value="blockchain">"Blockchain"</option>
                                    </select>
                                </label>
                                <div style="display:grid;gap:0.35rem;">
                                    <span>"Choose file"</span>
                                    <input id=document_upload::kyc_upload_input_id() type="file" style="padding:0.65rem;border:1px solid #d1d5db;border-radius:0.85rem;background:white;" />
                                </div>
                            </div>
                            <div style="display:flex;justify-content:flex-end;">
                                <button
                                    type="button"
                                    on:click=move |_| {
                                        let auth = auth.clone();
                                        let document_name = kyc_document_name.get().trim().to_string();
                                        let document_type = kyc_document_type.get();
                                        if document_name.is_empty() {
                                            feedback.set(Some("Enter a KYC document name before uploading.".into()));
                                            return;
                                        }
                                        upload_in_progress.set(true);
                                        spawn_local(async move {
                                            match document_upload::upload_kyc_document(&document_name, &document_type, document_upload::kyc_upload_input_id()).await {
                                                Ok(document) => {
                                                    auth.notice.set(Some(format!("Uploaded {} to the Rust KYC intake flow.", document.document_name)));
                                                    feedback.set(Some("KYC document uploaded. The onboarding screen is refreshing now.".into()));
                                                    kyc_document_name.set(String::new());
                                                    screen.set(None);
                                                    refresh_nonce.update(|value| *value += 1);
                                                }
                                                Err(error) => {
                                                    auth.notice.set(Some(error.clone()));
                                                    feedback.set(Some(error));
                                                }
                                            }
                                            upload_in_progress.set(false);
                                        });
                                    }
                                    style=button_style("#1d4ed8")
                                    disabled=move || upload_in_progress.get() || !can_submit
                                >
                                    {move || if upload_in_progress.get() { "Uploading..." } else if can_submit { "Upload KYC file" } else { "Read only" }}
                                </button>
                            </div>
                            <div style="display:grid;gap:0.5rem;">
                                {if screen_state.documents.is_empty() {
                                    view! { <small>"No KYC documents uploaded yet."</small> }.into_any()
                                } else {
                                    screen_state.documents.into_iter().map(|document| view! {
                                        <div style="display:flex;justify-content:space-between;gap:0.75rem;align-items:center;flex-wrap:wrap;padding:0.65rem 0.8rem;border:1px solid #e5e7eb;border-radius:0.85rem;background:white;">
                                            <div style="display:grid;gap:0.15rem;">
                                                <strong>{document.document_name}</strong>
                                                <small>{format!("{} | {}", document.document_type, document.uploaded_at_label)}</small>
                                            </div>
                                            {document.download_path.map(|path| view! {
                                                <button type="button" on:click=move |_| {
                                                    let path = path.clone();
                                                    spawn_local(async move {
                                                        let _ = document_upload::open_protected_document(&path).await;
                                                    });
                                                } style="padding:0.45rem 0.7rem;border:1px solid #cbd5e1;border-radius:0.75rem;background:white;cursor:pointer;">"View file"</button>
                                            })}
                                        </div>
                                    }).collect_view().into_any()
                                }}
                            </div>
                        </section>
                        <form on:submit=move |ev: ev::SubmitEvent| {
                            let navigate = submit_navigate.clone();
                            let auth = submit_auth.clone();
                            ev.prevent_default();
                            let payload = SubmitOnboardingRequest {
                                company_name: company_name.get().trim().to_string(),
                                company_address: company_address.get().trim().to_string(),
                                dot_number: optional_string(dot_number.get()),
                                mc_number: optional_string(mc_number.get()),
                                equipment_types: optional_string(equipment_types.get()),
                                business_entity_id: optional_string(business_entity_id.get()),
                                facility_address: optional_string(facility_address.get()),
                                fulfillment_contact_info: optional_string(fulfillment_contact_info.get()),
                                fmcsa_broker_license_no: optional_string(fmcsa_broker_license_no.get()),
                                mc_authority_number: optional_string(mc_authority_number.get()),
                                freight_forwarder_license: optional_string(freight_forwarder_license.get()),
                                customs_license: optional_string(customs_license.get()),
                            };

                            saving.set(true);
                            feedback.set(None);
                            spawn_local(async move {
                                match api::submit_onboarding(&payload).await {
                                    Ok(response) => {
                                        if let Some(session_state) = response.session.clone() {
                                            auth.session.set(session_state);
                                            auth.session_ready.set(true);
                                        }
                                        auth.notice.set(Some(response.message.clone()));
                                        feedback.set(Some(response.message.clone()));
                                        if response.success {
                                            navigate(&response.next_step, Default::default());
                                        }
                                    }
                                    Err(error) => {
                                        auth.notice.set(Some(error.clone().to_string()));
                                        feedback.set(Some(error.to_string()));
                                    }
                                }
                                saving.set(false);
                            });
                        } style="display:grid;gap:0.85rem;">
                            <TextField label="Company name" value=company_name input_type="text" placeholder="Company name" />
                            <TextAreaField label="Company address" value=company_address placeholder="Company address" />
                            {move || if is_carrier {
                                view! {
                                    <>
                                        <TextField label="DOT number" value=dot_number input_type="text" placeholder="DOT number" />
                                        <TextField label="MC number" value=mc_number input_type="text" placeholder="MC number" />
                                        <TextAreaField label="Equipment types" value=equipment_types placeholder="Dry van, reefer, flatbed" />
                                    </>
                                }.into_any()
                            } else { view! { <></> }.into_any() }}
                            {move || if is_shipper {
                                view! {
                                    <>
                                        <TextField label="Business entity ID" value=business_entity_id input_type="text" placeholder="Entity id" />
                                        <TextAreaField label="Facility address" value=facility_address placeholder="Primary facility address" />
                                        <TextAreaField label="Fulfillment contact info" value=fulfillment_contact_info placeholder="Name, phone, email" />
                                    </>
                                }.into_any()
                            } else { view! { <></> }.into_any() }}
                            {move || if is_broker {
                                view! {
                                    <>
                                        <TextField label="FMCSA broker license" value=fmcsa_broker_license_no input_type="text" placeholder="Broker license number" />
                                        <TextField label="MC authority number" value=mc_authority_number input_type="text" placeholder="MC authority number" />
                                    </>
                                }.into_any()
                            } else { view! { <></> }.into_any() }}
                            {move || if is_forwarder {
                                view! {
                                    <>
                                        <TextField label="Freight forwarder license" value=freight_forwarder_license input_type="text" placeholder="Forwarder license" />
                                        <TextField label="Customs license" value=customs_license input_type="text" placeholder="Customs license" />
                                    </>
                                }.into_any()
                            } else { view! { <></> }.into_any() }}
                            <div style="display:flex;justify-content:space-between;align-items:center;gap:0.75rem;flex-wrap:wrap;">
                                <nav style="display:flex;gap:0.8rem;flex-wrap:wrap;font-size:0.95rem;">
                                    <A href="/auth/login">"Back to login"</A>
                                    <A href="/">"Dashboard"</A>
                                </nav>
                                <button type="submit" style=button_style("#0f766e") disabled=move || saving.get() || !can_submit>
                                    {move || if saving.get() { "Submitting..." } else if can_submit { "Submit onboarding" } else { "Read only" }}
                                </button>
                            </div>
                        </form>
                    </>
                }.into_any()
            } else {
                view! { <p>"Unable to load the onboarding screen."</p> }.into_any()
            }}
        </AuthArticle>
    }
}
#[component]
fn AuthArticle(title: &'static str, subtitle: &'static str, children: Children) -> impl IntoView {
    view! {
        <article style="display:grid;gap:1rem;max-width:640px;">
            <section style="display:grid;gap:0.45rem;">
                <h2>{title}</h2>
                <p>{subtitle}</p>
            </section>
            {children()}
        </article>
    }
}

#[component]
fn SharedNotice() -> impl IntoView {
    let auth = use_auth();
    view! {
        {move || auth.notice.get().map(|message| view! {
            <section style="padding:0.85rem 1rem;border:1px solid #dbeafe;border-radius:0.9rem;background:#eff6ff;color:#1d4ed8;">
                {message}
            </section>
        })}
    }
}

#[component]
fn LocalNotice(message: RwSignal<Option<String>>) -> impl IntoView {
    view! {
        {move || message.get().map(|message| view! {
            <section style="padding:0.85rem 1rem;border:1px solid #dbeafe;border-radius:0.9rem;background:#eff6ff;color:#1d4ed8;white-space:pre-wrap;">
                {message}
            </section>
        })}
    }
}

#[component]
fn TextField(
    label: &'static str,
    value: RwSignal<String>,
    input_type: &'static str,
    placeholder: &'static str,
) -> impl IntoView {
    view! {
        <label style="display:grid;gap:0.35rem;">
            <span>{label}</span>
            <input
                type=input_type
                prop:value=move || value.get()
                on:input=move |ev| value.set(event_target_value(&ev))
                placeholder=placeholder
                style="padding:0.8rem 1rem;border:1px solid #d1d5db;border-radius:0.85rem;"
            />
        </label>
    }
}

#[component]
fn TextAreaField(
    label: &'static str,
    value: RwSignal<String>,
    placeholder: &'static str,
) -> impl IntoView {
    view! {
        <label style="display:grid;gap:0.35rem;">
            <span>{label}</span>
            <textarea
                prop:value=move || value.get()
                on:input=move |ev| value.set(event_target_value(&ev))
                placeholder=placeholder
                rows="3"
                style="padding:0.8rem 1rem;border:1px solid #d1d5db;border-radius:0.85rem;resize:vertical;"
            ></textarea>
        </label>
    }
}

#[component]
fn RoleField(value: RwSignal<String>) -> impl IntoView {
    view! {
        <label style="display:grid;gap:0.35rem;">
            <span>"Role"</span>
            <select
                prop:value=move || value.get()
                on:change=move |ev| value.set(event_target_value(&ev))
                style="padding:0.8rem 1rem;border:1px solid #d1d5db;border-radius:0.85rem;background:white;"
            >
                <option value="shipper">"Shipper"</option>
                <option value="carrier">"Carrier"</option>
                <option value="broker">"Broker"</option>
                <option value="freight_forwarder">"Freight Forwarder"</option>
            </select>
        </label>
    }
}

#[component]
fn OtpPurposeField(value: RwSignal<OtpPurpose>) -> impl IntoView {
    view! {
        <label style="display:grid;gap:0.35rem;">
            <span>"OTP purpose"</span>
            <select
                on:change=move |ev| {
                    let next = match event_target_value(&ev).as_str() {
                        "password_reset" => OtpPurpose::PasswordReset,
                        _ => OtpPurpose::Registration,
                    };
                    value.set(next);
                }
                style="padding:0.8rem 1rem;border:1px solid #d1d5db;border-radius:0.85rem;background:white;"
            >
                <option value="registration">"Registration"</option>
                <option value="password_reset">"Password reset"</option>
            </select>
        </label>
    }
}

fn optional_string(value: String) -> Option<String> {
    let trimmed = value.trim().to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

fn button_style(color: &'static str) -> String {
    format!(
        "padding:0.7rem 1rem;border:none;border-radius:0.85rem;background:{};color:white;cursor:pointer;",
        color
    )
}
