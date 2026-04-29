use leptos::{ev, prelude::*, task::spawn_local};
use leptos_router::{
    components::A,
    hooks::{use_navigate, use_query_map},
};
use serde::{Deserialize, Serialize};

use crate::{
    api, document_upload, google_places, runtime_config,
    session::{self, use_auth},
};
use shared::{
    AuthOnboardingScreen, ForgotPasswordRequest, LoginRequest, OtpPurpose, RegisterRequest,
    ResetPasswordRequest, SubmitOnboardingRequest, VerifyOtpRequest,
};

#[component]
pub fn PortalLandingPage() -> impl IntoView {
    view! {
        <section class="portal-home">
            <div class="portal-topbar">
                <img
                    class="portal-logo"
                    src="https://portal.stloads.com/assets/images/stloads/logo-bg_none-small.png"
                    alt="LoadBoard Logo"
                />
                <nav class="portal-nav">
                    <A href="/" attr:class="portal-nav-link is-active">"Home"</A>
                    <A href="https://stloads.com/about-us" attr:class="portal-nav-link">"About"</A>
                    <A href="https://stloads.com/services" attr:class="portal-nav-link">"Services"</A>
                    <A href="https://stloads.com/contact-us" attr:class="portal-nav-link">"Contact"</A>
                    <A href="/auth/login?portal=admin" attr:class="portal-nav-link portal-admin-link">"Admin Portal"</A>
                </nav>
            </div>

            <div class="portal-heading">
                <h2 class="portal-title">"Welcome to LoadBoard - Where Smart Logistics Begin."</h2>
                <h5 class="portal-subtitle">"Select your role"</h5>
                <p class="portal-description">"To start your project we need to customize your preferences."</p>
            </div>

            <section class="portal-role-grid">
                <RoleSignupCard
                    href="/auth/register?role=shipper"
                    icon_class="fas fa-boxes"
                    title="Shipper"
                    role_count="Count 7"
                    description="Get your shipper account set up"
                />
                <RoleSignupCard
                    href="/auth/register?role=carrier"
                    icon_class="fas fa-truck-fast"
                    title="Carrier"
                    role_count="Count 4"
                    description="Start carrier signup"
                />
                <RoleSignupCard
                    href="/auth/register?role=broker"
                    icon_class="fas fa-handshake-angle"
                    title="Broker"
                    role_count="Count 5"
                    description="Start broker signup"
                />
                <RoleSignupCard
                    href="/auth/register?role=freight_forwarder"
                    icon_class="fas fa-ship"
                    title="Freight Forwarder"
                    role_count="Count 3"
                    description="Start forwarder signup"
                />
            </section>
        </section>
    }
}

#[component]
pub fn LoginPage() -> impl IntoView {
    let navigate = use_navigate();
    let query = use_query_map();
    let auth = use_auth();
    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let is_submitting = RwSignal::new(false);
    let is_admin_portal = Memo::new(move |_| {
        query.with(|params| {
            params
                .get("portal")
                .map(|value| value == "admin")
                .unwrap_or(false)
        })
    });
    let title = Signal::derive(move || {
        if is_admin_portal.get() {
            "Admin Portal Login".to_string()
        } else {
            "Welcome back".to_string()
        }
    });
    let subtitle = Signal::derive(move || {
        if is_admin_portal.get() {
            "Use the separate admin access lane for operations, onboarding review, and internal control screens.".to_string()
        } else {
            "Sign in to continue from the public portal into your dashboard, onboarding flow, and account workspace.".to_string()
        }
    });

    Effect::new(move |_| {
        query.with(|params| {
            if email.get_untracked().is_empty() {
                if let Some(prefill) = params.get("email") {
                    email.set(prefill.clone());
                }
            }
        });
    });

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
            title=title
            subtitle=subtitle
        >
            <SharedNotice />

            <form on:submit=on_submit class="auth-form">
                <TextField label="Email" value=email input_type="email" placeholder="name@example.com" />
                <PasswordField label="Password" value=password placeholder="Enter your password" />

                <div class="auth-actions">
                    <nav class="auth-links">
                        <A href="/" attr:class="auth-link">"Back to portal"</A>
                        <A href="/auth/register" attr:class="auth-link">"Create account"</A>
                        <A href="/auth/forgot-password" attr:class="auth-link">"Forgot password"</A>
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
    let query = use_query_map();
    let auth = use_auth();
    let name = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let phone = RwSignal::new(String::new());
    let address = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let password_confirmation = RwSignal::new(String::new());
    let role = RwSignal::new("shipper".to_string());
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
    let feedback = RwSignal::new(None::<String>);
    let is_submitting = RwSignal::new(false);
    let register_step = RwSignal::new(0_usize);
    let selected_role_label = Memo::new(move |_| role_label(role.get()));
    let title = Signal::derive(move || format!("{} Signup", selected_role_label.get()));
    let subtitle = Signal::derive(move || {
        format!(
            "Create a {} account with a role-specific signup flow, then continue into OTP verification and onboarding.",
            selected_role_label.get().to_lowercase()
        )
    });

    Effect::new(move |_| {
        query.with(|params| {
            if let Some(role_key) = params.get("role") {
                if is_supported_role(role_key.as_str()) {
                    role.set(role_key.clone());
                }
            }
            if email.get_untracked().is_empty() {
                if let Some(prefill) = params.get("email") {
                    email.set(prefill.clone());
                }
            }
        });
    });

    let on_next = move |_| {
        feedback.set(None);
        if let Some(message) = validate_register_step(
            register_step.get(),
            role.get().as_str(),
            &name.get(),
            &email.get(),
            &phone.get(),
            &address.get(),
            &password.get(),
            &password_confirmation.get(),
            &company_name.get(),
            &company_address.get(),
            &dot_number.get(),
            &mc_number.get(),
            &equipment_types.get(),
            &business_entity_id.get(),
            &facility_address.get(),
            &fulfillment_contact_info.get(),
            &fmcsa_broker_license_no.get(),
            &mc_authority_number.get(),
            &freight_forwarder_license.get(),
            &customs_license.get(),
        ) {
            feedback.set(Some(message));
            return;
        }
        register_step.set(1);
    };

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

        if let Some(message) = validate_register_step(
            register_step.get(),
            payload.role_key.as_str(),
            &payload.name,
            &payload.email,
            &phone.get(),
            &address.get(),
            &payload.password,
            &payload.password_confirmation,
            &company_name.get(),
            &company_address.get(),
            &dot_number.get(),
            &mc_number.get(),
            &equipment_types.get(),
            &business_entity_id.get(),
            &facility_address.get(),
            &fulfillment_contact_info.get(),
            &fmcsa_broker_license_no.get(),
            &mc_authority_number.get(),
            &freight_forwarder_license.get(),
            &customs_license.get(),
        ) {
            feedback.set(Some(message));
            return;
        }

        let pending_draft = PendingOnboardingDraft {
            role_key: payload.role_key.clone(),
            company_name: optional_string(company_name.get()),
            company_address: optional_string(company_address.get()),
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
                        save_pending_onboarding_draft(&pending_draft);
                        navigate(
                            &build_verify_otp_path(&response.email, OtpPurpose::Registration),
                            Default::default(),
                        );
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
            title=title
            subtitle=subtitle
        >
            <LocalNotice message=feedback />
            <RoleRegisterSwitcher current_role=role />
            <RegisterStepHeader role=role current_step=register_step />
            <form on:submit=on_submit class="auth-form">
                {move || if register_step.get() == 0 {
                    view! {
                        <>
                            <TextField label="Full name" value=name input_type="text" placeholder="Your full name" />
                            <EmailField label="Email" value=email />
                            <PhoneField label="Phone" value=phone />
                            <AddressAutocompleteField
                                label="Address"
                                value=address
                                placeholder="Search business or profile address"
                                input_id="register-primary-address"
                            />
                            <PasswordField label="Password" value=password placeholder="Create a password" />
                            <PasswordField
                                label="Confirm password"
                                value=password_confirmation
                                placeholder="Repeat the password"
                            />
                        </>
                    }.into_any()
                } else {
                    view! {
                        <>
                            <TextField label="Company name" value=company_name input_type="text" placeholder="Legal company or brokerage name" />
                            <AddressAutocompleteField
                                label="Company address"
                                value=company_address
                                placeholder="Search registered or operating address"
                                input_id="register-company-address"
                            />
                            {move || match role.get().as_str() {
                                "carrier" => view! {
                                    <>
                                        <TextField label="DOT number" value=dot_number input_type="text" placeholder="USDOT / DOT number" />
                                        <TextField label="MC number" value=mc_number input_type="text" placeholder="Motor carrier number" />
                                        <TextAreaField label="Equipment types" value=equipment_types placeholder="Dry van, reefer, flatbed" />
                                    </>
                                }.into_any(),
                                "shipper" => view! {
                                    <>
                                        <TextField label="Business entity ID" value=business_entity_id input_type="text" placeholder="Registration or entity ID" />
                                        <AddressAutocompleteField
                                            label="Facility address"
                                            value=facility_address
                                            placeholder="Search primary facility or warehouse address"
                                            input_id="register-facility-address"
                                        />
                                        <TextAreaField label="Fulfillment contact info" value=fulfillment_contact_info placeholder="Dispatcher or warehouse contact" />
                                    </>
                                }.into_any(),
                                "broker" => view! {
                                    <>
                                        <TextField label="FMCSA broker license" value=fmcsa_broker_license_no input_type="text" placeholder="Broker license number" />
                                        <TextField label="MC authority number" value=mc_authority_number input_type="text" placeholder="MC authority number" />
                                    </>
                                }.into_any(),
                                "freight_forwarder" => view! {
                                    <>
                                        <TextField label="Freight forwarder license" value=freight_forwarder_license input_type="text" placeholder="Forwarder license number" />
                                        <TextField label="Customs license" value=customs_license input_type="text" placeholder="Customs or clearance license" />
                                    </>
                                }.into_any(),
                                _ => view! { <></> }.into_any(),
                            }}
                        </>
                    }.into_any()
                }}
                <div class="auth-actions">
                    <nav class="auth-links">
                        <A href="/" attr:class="auth-link">"Back to portal"</A>
                        <A href="/auth/login" attr:class="auth-link">"Back to login"</A>
                        <A href="/auth/forgot-password" attr:class="auth-link">"Forgot password"</A>
                        <A href="/auth/verify-otp" attr:class="auth-link">"Already have an OTP?"</A>
                    </nav>
                    <div class="auth-inline-actions">
                        <button
                            type="button"
                            style=button_style("#475569")
                            disabled=move || register_step.get() == 0
                            on:click=move |_| register_step.set(0)
                        >
                            "Previous"
                        </button>
                        {move || if register_step.get() == 0 {
                            view! {
                                <button type="button" style=button_style("#0f766e") on:click=on_next>
                                    "Next"
                                </button>
                            }.into_any()
                        } else {
                            view! {
                                <button type="submit" style=button_style("#0f766e") disabled=move || is_submitting.get()>
                                    {move || if is_submitting.get() { "Creating account..." } else { "Create account" }}
                                </button>
                            }.into_any()
                        }}
                    </div>
                </div>
            </form>
        </AuthArticle>
    }
}

#[component]
pub fn VerifyOtpPage() -> impl IntoView {
    let navigate = use_navigate();
    let query = use_query_map();
    let auth = use_auth();
    let email = RwSignal::new(String::new());
    let otp = RwSignal::new(String::new());
    let purpose = RwSignal::new(OtpPurpose::Registration);
    let reset_token = RwSignal::new(String::new());
    let feedback = RwSignal::new(None::<String>);
    let is_submitting = RwSignal::new(false);
    let is_resending = RwSignal::new(false);

    Effect::new(move |_| {
        query.with(|params| {
            if email.get_untracked().is_empty() {
                if let Some(prefill) = params.get("email") {
                    email.set(prefill.clone());
                }
            }
            if reset_token.get_untracked().is_empty() {
                if let Some(token) = params.get("token") {
                    reset_token.set(token.clone());
                }
            }
            if let Some(value) = params.get("purpose") {
                if let Some(next_purpose) = otp_purpose_from_query(&value) {
                    purpose.set(next_purpose);
                }
            }
        });
    });

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
                    let reset_token_from_response = response.reset_token.clone();
                    if let Some(token) = reset_token_from_response.clone() {
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
                        let destination = match response.purpose {
                            OtpPurpose::PasswordReset => {
                                if let Some(token) = reset_token_from_response {
                                    build_reset_password_path(&response.email, &token)
                                } else {
                                    build_reset_password_path(&response.email, &reset_token.get())
                                }
                            }
                            OtpPurpose::Registration => response.next_step,
                        };
                        if matches!(response.purpose, OtpPurpose::PasswordReset) {
                            clear_pending_onboarding_draft();
                        }
                        navigate(&destination, Default::default());
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
            title=Signal::derive(|| "Verify OTP".to_string())
            subtitle=Signal::derive(|| {
                "Use the same screen for new-account OTP verification and password-reset OTP verification.".to_string()
            })
        >
            <LocalNotice message=feedback />
            <form on:submit=on_submit class="auth-form">
                <TextField label="Email" value=email input_type="email" placeholder="name@example.com" />
                <OtpPurposeField value=purpose />
                <TextField label="OTP" value=otp input_type="text" placeholder="6-digit code" />
                {move || if reset_token.get().is_empty() {
                    view! { <></> }.into_any()
                } else {
                    view! {
                        <section class="auth-notice" style="display:grid;gap:0.25rem;">
                            <strong>"Development reset token"</strong>
                            <code style="font-size:0.9rem;word-break:break-all;">{move || reset_token.get()}</code>
                        </section>
                    }.into_any()
                }}
                <div class="auth-actions">
                    <nav class="auth-links">
                        <A href="/auth/login" attr:class="auth-link">"Back to login"</A>
                        <A href="/auth/register" attr:class="auth-link">"Create account"</A>
                        <A href="/auth/forgot-password" attr:class="auth-link">"Forgot password"</A>
                    </nav>
                    <div class="auth-inline-actions">
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
    let query = use_query_map();
    let auth = use_auth();
    let email = RwSignal::new(String::new());
    let feedback = RwSignal::new(None::<String>);
    let is_submitting = RwSignal::new(false);

    Effect::new(move |_| {
        query.with(|params| {
            if email.get_untracked().is_empty() {
                if let Some(prefill) = params.get("email") {
                    email.set(prefill.clone());
                }
            }
        });
    });

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
                        navigate(
                            &build_verify_otp_path(&response.email, OtpPurpose::PasswordReset),
                            Default::default(),
                        );
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
            title=Signal::derive(|| "Forgot Password".to_string())
            subtitle=Signal::derive(|| {
                "This replaces the legacy forgot-password entry point with the Rust OTP-first reset flow.".to_string()
            })
        >
            <LocalNotice message=feedback />
            <form on:submit=on_submit class="auth-form">
                <TextField label="Email" value=email input_type="email" placeholder="name@example.com" />
                <div class="auth-actions">
                    <nav class="auth-links">
                        <A href="/auth/login" attr:class="auth-link">"Back to login"</A>
                        <A href="/auth/verify-otp" attr:class="auth-link">"Already have an OTP?"</A>
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
    let query = use_query_map();
    let auth = use_auth();
    let email = RwSignal::new(String::new());
    let reset_token = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let password_confirmation = RwSignal::new(String::new());
    let feedback = RwSignal::new(None::<String>);
    let is_submitting = RwSignal::new(false);

    Effect::new(move |_| {
        query.with(|params| {
            if email.get_untracked().is_empty() {
                if let Some(prefill) = params.get("email") {
                    email.set(prefill.clone());
                }
            }
            if reset_token.get_untracked().is_empty() {
                if let Some(prefill) = params.get("token") {
                    reset_token.set(prefill.clone());
                }
            }
        });
    });

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
                        navigate(
                            &format!("/auth/login?email={}", encode_query_value(&response.email)),
                            Default::default(),
                        );
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
            title=Signal::derive(|| "Reset Password".to_string())
            subtitle=Signal::derive(|| {
                "Complete the Rust password reset after OTP verification by setting a fresh password here.".to_string()
            })
        >
            <LocalNotice message=feedback />
            <form on:submit=on_submit class="auth-form">
                <TextField label="Email" value=email input_type="email" placeholder="name@example.com" />
                <TextAreaField label="Reset token" value=reset_token placeholder="Paste the reset token from the verification step" />
                <TextField label="New password" value=password input_type="password" placeholder="New password" />
                <TextField label="Confirm new password" value=password_confirmation input_type="password" placeholder="Repeat the new password" />
                <div class="auth-actions">
                    <nav class="auth-links">
                        <A href="/auth/verify-otp" attr:class="auth-link">"Back to OTP verify"</A>
                        <A href="/auth/login" attr:class="auth-link">"Back to login"</A>
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
                    let pending = load_pending_onboarding_draft()
                        .filter(|draft| draft.role_key == response.role_key);
                    company_name.set(prefill_auth_field(
                        response.draft.company_name.clone(),
                        pending
                            .as_ref()
                            .and_then(|draft| draft.company_name.clone()),
                    ));
                    company_address.set(prefill_auth_field(
                        response.draft.company_address.clone(),
                        pending
                            .as_ref()
                            .and_then(|draft| draft.company_address.clone()),
                    ));
                    dot_number.set(prefill_auth_field(
                        response.draft.dot_number.clone(),
                        pending.as_ref().and_then(|draft| draft.dot_number.clone()),
                    ));
                    mc_number.set(prefill_auth_field(
                        response.draft.mc_number.clone(),
                        pending.as_ref().and_then(|draft| draft.mc_number.clone()),
                    ));
                    equipment_types.set(prefill_auth_field(
                        response.draft.equipment_types.clone(),
                        pending
                            .as_ref()
                            .and_then(|draft| draft.equipment_types.clone()),
                    ));
                    business_entity_id.set(prefill_auth_field(
                        response.draft.business_entity_id.clone(),
                        pending
                            .as_ref()
                            .and_then(|draft| draft.business_entity_id.clone()),
                    ));
                    facility_address.set(prefill_auth_field(
                        response.draft.facility_address.clone(),
                        pending
                            .as_ref()
                            .and_then(|draft| draft.facility_address.clone()),
                    ));
                    fulfillment_contact_info.set(prefill_auth_field(
                        response.draft.fulfillment_contact_info.clone(),
                        pending
                            .as_ref()
                            .and_then(|draft| draft.fulfillment_contact_info.clone()),
                    ));
                    fmcsa_broker_license_no.set(prefill_auth_field(
                        response.draft.fmcsa_broker_license_no.clone(),
                        pending
                            .as_ref()
                            .and_then(|draft| draft.fmcsa_broker_license_no.clone()),
                    ));
                    mc_authority_number.set(prefill_auth_field(
                        response.draft.mc_authority_number.clone(),
                        pending
                            .as_ref()
                            .and_then(|draft| draft.mc_authority_number.clone()),
                    ));
                    freight_forwarder_license.set(prefill_auth_field(
                        response.draft.freight_forwarder_license.clone(),
                        pending
                            .as_ref()
                            .and_then(|draft| draft.freight_forwarder_license.clone()),
                    ));
                    customs_license.set(prefill_auth_field(
                        response.draft.customs_license.clone(),
                        pending
                            .as_ref()
                            .and_then(|draft| draft.customs_license.clone()),
                    ));
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
            title=Signal::derive(|| "Rust Onboarding".to_string())
            subtitle=Signal::derive(|| {
                "OTP-complete accounts continue here until the company profile is submitted for review.".to_string()
            })
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
                                            clear_pending_onboarding_draft();
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
                            <AddressAutocompleteField
                                label="Company address"
                                value=company_address
                                placeholder="Search company address"
                                input_id="onboarding-company-address"
                            />
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
                                        <AddressAutocompleteField
                                            label="Facility address"
                                            value=facility_address
                                            placeholder="Search primary facility address"
                                            input_id="onboarding-facility-address"
                                        />
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
fn AuthArticle(
    #[prop(into)] title: Signal<String>,
    #[prop(into)] subtitle: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <article class="auth-article">
            <section class="auth-title-block">
                <span class="auth-kicker">"STLoads Portal"</span>
                <h2 class="auth-title">{move || title.get()}</h2>
                <p class="auth-subtitle">{move || subtitle.get()}</p>
            </section>
            <div class="auth-surface">{children()}</div>
        </article>
    }
}

#[component]
fn RoleSignupCard(
    href: &'static str,
    icon_class: &'static str,
    title: &'static str,
    role_count: &'static str,
    description: &'static str,
) -> impl IntoView {
    view! {
        <article class="portal-role-card">
            <div class="portal-role-content">
                <i class=format!("{icon_class} portal-role-icon")></i>
                <h3 class="portal-role-title">{title}</h3>
                <p class="portal-role-count">{role_count}</p>
                <p class="portal-role-copy">{description}</p>
                <A href=href attr:class="portal-role-cta">
                    "Sign up"
                    <i class="fas fa-arrow-right"></i>
                </A>
            </div>
        </article>
    }
}

#[component]
fn RoleRegisterSwitcher(current_role: RwSignal<String>) -> impl IntoView {
    view! {
        <section class="auth-field">
            <span class="auth-label">"Signup role"</span>
            <div class="auth-role-picker">
                <A href="/auth/register?role=shipper" attr:class=move || selection_option_class(current_role.get() == "shipper")>
                    <span class="auth-role-option-title">"Shipper"</span>
                    <span class="auth-role-option-copy">"Warehouse, fulfillment, and facility details."</span>
                </A>
                <A href="/auth/register?role=carrier" attr:class=move || selection_option_class(current_role.get() == "carrier")>
                    <span class="auth-role-option-title">"Carrier"</span>
                    <span class="auth-role-option-copy">"DOT, MC, and equipment-focused onboarding."</span>
                </A>
                <A href="/auth/register?role=broker" attr:class=move || selection_option_class(current_role.get() == "broker")>
                    <span class="auth-role-option-title">"Broker"</span>
                    <span class="auth-role-option-copy">"Broker license and authority details."</span>
                </A>
                <A href="/auth/register?role=freight_forwarder" attr:class=move || selection_option_class(current_role.get() == "freight_forwarder")>
                    <span class="auth-role-option-title">"Freight Forwarder"</span>
                    <span class="auth-role-option-copy">"Forwarder and customs licensing information."</span>
                </A>
            </div>
        </section>
    }
}

#[component]
fn RegisterStepHeader(role: RwSignal<String>, current_step: RwSignal<usize>) -> impl IntoView {
    let account_label = Signal::derive(move || register_step_labels(role.get())[0].to_string());
    let detail_label = Signal::derive(move || register_step_labels(role.get())[1].to_string());
    view! {
        <section class="auth-field">
            <span class="auth-label">"Signup steps"</span>
            <div class="auth-role-picker">
                <div class=move || selection_option_class(current_step.get() == 0)>
                    <span class="auth-role-option-title">{move || account_label.get()}</span>
                    <span class="auth-role-option-copy">"Create the account credentials first."</span>
                </div>
                <div class=move || selection_option_class(current_step.get() == 1)>
                    <span class="auth-role-option-title">{move || detail_label.get()}</span>
                    <span class="auth-role-option-copy">"Match the role-specific business fields from the PHP flow."</span>
                </div>
            </div>
        </section>
    }
}

#[component]
fn SharedNotice() -> impl IntoView {
    let auth = use_auth();
    view! {
        {move || auth.notice.get().map(|message| view! {
            <section class="auth-notice">
                {message}
            </section>
        })}
    }
}

#[component]
fn LocalNotice(message: RwSignal<Option<String>>) -> impl IntoView {
    view! {
        {move || message.get().map(|message| view! {
            <section class="auth-notice" style="white-space:pre-wrap;">
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
        <label class="auth-field">
            <span class="auth-label">{label}</span>
            <input
                type=input_type
                prop:value=move || value.get()
                on:input=move |ev| value.set(event_target_value(&ev))
                placeholder=placeholder
                class="auth-input"
            />
        </label>
    }
}

#[component]
fn EmailField(label: &'static str, value: RwSignal<String>) -> impl IntoView {
    view! {
        <label class="auth-field">
            <span class="auth-label">{label}</span>
            <input
                type="email"
                prop:value=move || value.get()
                on:input=move |ev| value.set(event_target_value(&ev).trim().to_string())
                placeholder="abc@xyz.com"
                class="auth-input"
                inputmode="email"
                autocomplete="email"
            />
            <small class="auth-help">"Use a valid email like abc@xyz.com"</small>
        </label>
    }
}

#[component]
fn PhoneField(label: &'static str, value: RwSignal<String>) -> impl IntoView {
    view! {
        <label class="auth-field">
            <span class="auth-label">{label}</span>
            <input
                type="tel"
                prop:value=move || value.get()
                on:input=move |ev| value.set(event_target_value(&ev))
                placeholder="+1(123) 456-6789"
                class="auth-input"
                inputmode="tel"
                autocomplete="tel"
            />
            <small class="auth-help">"Include your own country code, for example +1(123) 456-6789"</small>
        </label>
    }
}

#[component]
fn PasswordField(
    label: &'static str,
    value: RwSignal<String>,
    placeholder: &'static str,
) -> impl IntoView {
    let visible = RwSignal::new(false);
    view! {
        <label class="auth-field">
            <span class="auth-label">{label}</span>
            <div class="auth-input-shell">
                <input
                    type=move || if visible.get() { "text" } else { "password" }
                    prop:value=move || value.get()
                    on:input=move |ev| value.set(event_target_value(&ev))
                    placeholder=placeholder
                    class="auth-input auth-input-with-action"
                    autocomplete="current-password"
                />
                <button
                    type="button"
                    class="auth-input-action"
                    on:click=move |_| visible.update(|state| *state = !*state)
                    aria-label=move || if visible.get() { "Hide password" } else { "Show password" }
                    title=move || if visible.get() { "Hide password" } else { "Show password" }
                >
                    <i class=move || if visible.get() { "fas fa-eye-slash" } else { "fas fa-eye" }></i>
                </button>
            </div>
        </label>
    }
}

#[component]
fn AddressAutocompleteField(
    label: &'static str,
    value: RwSignal<String>,
    placeholder: &'static str,
    input_id: &'static str,
) -> impl IntoView {
    let city = RwSignal::new(String::new());
    let country = RwSignal::new(String::new());
    let place_id = RwSignal::new(String::new());
    let latitude = RwSignal::new(String::new());
    let longitude = RwSignal::new(String::new());
    let google_status = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        let api_key = runtime_config::google_maps_api_key();
        let input_id = input_id.to_string();
        let city_id = format!("{}-city", input_id);
        let country_id = format!("{}-country", input_id);
        let place_id_id = format!("{}-place-id", input_id);
        let latitude_id = format!("{}-latitude", input_id);
        let longitude_id = format!("{}-longitude", input_id);

        spawn_local(async move {
            match api_key {
                Some(api_key) => {
                    if let Err(error) = google_places::ensure_loaded(&api_key).await {
                        google_status.set(Some(error));
                        return;
                    }
                    if let Err(error) = google_places::attach_address_autocomplete(
                        &input_id,
                        &city_id,
                        &country_id,
                        &place_id_id,
                        &latitude_id,
                        &longitude_id,
                    )
                    .await
                    {
                        google_status.set(Some(error));
                    }
                }
                None => {
                    google_status.set(Some(
                        "Google Maps suggestions are unavailable because GOOGLE_MAPS_API_KEY is missing."
                            .into(),
                    ));
                }
            }
        });
    });

    view! {
        <label class="auth-field">
            <span class="auth-label">{label}</span>
            <input
                id=input_id
                type="text"
                prop:value=move || value.get()
                on:input=move |ev| value.set(event_target_value(&ev))
                placeholder=placeholder
                class="auth-input"
                autocomplete="street-address"
            />
            <input id=format!("{}-city", input_id) type="hidden" prop:value=move || city.get() on:input=move |ev| city.set(event_target_value(&ev)) />
            <input id=format!("{}-country", input_id) type="hidden" prop:value=move || country.get() on:input=move |ev| country.set(event_target_value(&ev)) />
            <input id=format!("{}-place-id", input_id) type="hidden" prop:value=move || place_id.get() on:input=move |ev| place_id.set(event_target_value(&ev)) />
            <input id=format!("{}-latitude", input_id) type="hidden" prop:value=move || latitude.get() on:input=move |ev| latitude.set(event_target_value(&ev)) />
            <input id=format!("{}-longitude", input_id) type="hidden" prop:value=move || longitude.get() on:input=move |ev| longitude.set(event_target_value(&ev)) />
            <small class="auth-help">"Start typing and choose an address from Google suggestions."</small>
            {move || google_status.get().map(|message| view! {
                <small class="auth-help">{message}</small>
            })}
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
        <label class="auth-field">
            <span class="auth-label">{label}</span>
            <textarea
                prop:value=move || value.get()
                on:input=move |ev| value.set(event_target_value(&ev))
                placeholder=placeholder
                rows="3"
                class="auth-textarea"
            ></textarea>
        </label>
    }
}

#[component]
fn OtpPurposeField(value: RwSignal<OtpPurpose>) -> impl IntoView {
    view! {
        <section class="auth-field">
            <span class="auth-label">"OTP purpose"</span>
            <div class="auth-role-picker">
                <button
                    type="button"
                    class=move || selection_option_class(matches!(value.get(), OtpPurpose::Registration))
                    on:click=move |_| value.set(OtpPurpose::Registration)
                >
                    <span class="auth-role-option-title">"Registration"</span>
                    <span class="auth-role-option-copy">"Verify a newly created account and continue onboarding."</span>
                </button>
                <button
                    type="button"
                    class=move || selection_option_class(matches!(value.get(), OtpPurpose::PasswordReset))
                    on:click=move |_| value.set(OtpPurpose::PasswordReset)
                >
                    <span class="auth-role-option-title">"Password reset"</span>
                    <span class="auth-role-option-copy">"Use OTP to recover access and set a new password."</span>
                </button>
            </div>
        </section>
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
        "padding:0.7rem 1rem;border:none;border-radius:0.85rem;background:{};color:white;cursor:pointer;font-weight:700;box-shadow:0 14px 32px rgba(15,23,42,0.08);",
        color
    )
}

fn selection_option_class(active: bool) -> &'static str {
    if active {
        "auth-role-option is-active"
    } else {
        "auth-role-option"
    }
}

fn is_supported_role(value: &str) -> bool {
    matches!(
        value,
        "shipper" | "carrier" | "broker" | "freight_forwarder"
    )
}

fn register_step_labels(role_key: String) -> [&'static str; 2] {
    match role_key.as_str() {
        "carrier" => ["Identity", "Carrier Details"],
        "broker" => ["Identity", "Brokerage Details"],
        "freight_forwarder" => ["Company", "Licensing Details"],
        _ => ["Account", "Company Details"],
    }
}

fn role_label(value: String) -> &'static str {
    match value.as_str() {
        "carrier" => "Carrier",
        "broker" => "Broker",
        "freight_forwarder" => "Freight Forwarder",
        _ => "Shipper",
    }
}

fn encode_query_value(value: &str) -> String {
    urlencoding::encode(value).into_owned()
}

fn build_verify_otp_path(email: &str, purpose: OtpPurpose) -> String {
    format!(
        "/auth/verify-otp?email={}&purpose={}",
        encode_query_value(email),
        otp_purpose_query_value(purpose)
    )
}

fn build_reset_password_path(email: &str, token: &str) -> String {
    format!(
        "/auth/reset-password?email={}&token={}",
        encode_query_value(email),
        encode_query_value(token)
    )
}

fn otp_purpose_query_value(purpose: OtpPurpose) -> &'static str {
    match purpose {
        OtpPurpose::Registration => "registration",
        OtpPurpose::PasswordReset => "password_reset",
    }
}

fn otp_purpose_from_query(value: &str) -> Option<OtpPurpose> {
    match value {
        "registration" => Some(OtpPurpose::Registration),
        "password_reset" | "reset" => Some(OtpPurpose::PasswordReset),
        _ => None,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct PendingOnboardingDraft {
    role_key: String,
    company_name: Option<String>,
    company_address: Option<String>,
    dot_number: Option<String>,
    mc_number: Option<String>,
    equipment_types: Option<String>,
    business_entity_id: Option<String>,
    facility_address: Option<String>,
    fulfillment_contact_info: Option<String>,
    fmcsa_broker_license_no: Option<String>,
    mc_authority_number: Option<String>,
    freight_forwarder_license: Option<String>,
    customs_license: Option<String>,
}

fn validate_register_step(
    step: usize,
    role_key: &str,
    name: &str,
    email: &str,
    phone: &str,
    address: &str,
    password: &str,
    password_confirmation: &str,
    company_name: &str,
    company_address: &str,
    dot_number: &str,
    mc_number: &str,
    equipment_types: &str,
    business_entity_id: &str,
    facility_address: &str,
    fulfillment_contact_info: &str,
    fmcsa_broker_license_no: &str,
    mc_authority_number: &str,
    freight_forwarder_license: &str,
    customs_license: &str,
) -> Option<String> {
    if step == 0 {
        if name.trim().is_empty()
            || email.trim().is_empty()
            || phone.trim().is_empty()
            || address.trim().is_empty()
            || password.is_empty()
            || password_confirmation.is_empty()
        {
            return Some(
                "Complete all account fields before moving to the role-specific step.".into(),
            );
        }
        if password != password_confirmation {
            return Some("Passwords do not match.".into());
        }
        if password.len() < 8 {
            return Some("Use a password with at least 8 characters.".into());
        }
        if !is_valid_email_format(email) {
            return Some("Enter a valid email in the format abc@xyz.com.".into());
        }
        if !is_valid_phone_format(phone) {
            return Some("Enter the phone number in the format +1(123) 456-6789.".into());
        }
        return None;
    }

    if company_name.trim().is_empty() || company_address.trim().is_empty() {
        return Some("Company name and company address are required.".into());
    }

    match role_key {
        "carrier"
            if dot_number.trim().is_empty()
                || mc_number.trim().is_empty()
                || equipment_types.trim().is_empty() =>
        {
            Some("Carrier signup requires DOT number, MC number, and equipment types.".into())
        }
        "shipper"
            if business_entity_id.trim().is_empty()
                || facility_address.trim().is_empty()
                || fulfillment_contact_info.trim().is_empty() =>
        {
            Some("Shipper signup requires business entity ID, facility address, and fulfillment contact info.".into())
        }
        "broker"
            if fmcsa_broker_license_no.trim().is_empty()
                || mc_authority_number.trim().is_empty() =>
        {
            Some("Broker signup requires FMCSA broker license and MC authority number.".into())
        }
        "freight_forwarder"
            if freight_forwarder_license.trim().is_empty()
                || customs_license.trim().is_empty() =>
        {
            Some("Freight forwarder signup requires freight forwarder and customs license values.".into())
        }
        _ => None,
    }
}

fn is_valid_email_format(value: &str) -> bool {
    let trimmed = value.trim();
    let mut parts = trimmed.split('@');
    let local = parts.next().unwrap_or_default();
    let domain = parts.next().unwrap_or_default();
    parts.next().is_none()
        && !local.is_empty()
        && domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
}

fn is_valid_phone_format(value: &str) -> bool {
    let trimmed = value.trim();
    if !trimmed.starts_with('+') {
        return false;
    }

    let Some(open_index) = trimmed.find('(') else {
        return false;
    };
    let Some(close_index) = trimmed.find(')') else {
        return false;
    };
    if close_index <= open_index || close_index + 1 >= trimmed.len() {
        return false;
    }

    let country_code = &trimmed[1..open_index];
    let area_code = &trimmed[open_index + 1..close_index];
    let remainder = &trimmed[close_index + 1..];

    let Some(remainder) = remainder.strip_prefix(' ') else {
        return false;
    };
    let Some((prefix, line_number)) = remainder.split_once('-') else {
        return false;
    };

    (1..=3).contains(&country_code.len())
        && (2..=4).contains(&area_code.len())
        && (3..=4).contains(&prefix.len())
        && (3..=4).contains(&line_number.len())
        && country_code.chars().all(|ch| ch.is_ascii_digit())
        && area_code.chars().all(|ch| ch.is_ascii_digit())
        && prefix.chars().all(|ch| ch.is_ascii_digit())
        && line_number.chars().all(|ch| ch.is_ascii_digit())
}

fn prefill_auth_field(server_value: Option<String>, local_value: Option<String>) -> String {
    server_value
        .filter(|value| !value.trim().is_empty())
        .or(local_value.filter(|value| !value.trim().is_empty()))
        .unwrap_or_default()
}

#[cfg(target_arch = "wasm32")]
const PENDING_ONBOARDING_DRAFT_KEY: &str = "stloads.pending-onboarding-draft";

#[cfg(target_arch = "wasm32")]
fn save_pending_onboarding_draft(draft: &PendingOnboardingDraft) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(serialized) = serde_json::to_string(draft) {
                let _ = storage.set_item(PENDING_ONBOARDING_DRAFT_KEY, &serialized);
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn save_pending_onboarding_draft(_draft: &PendingOnboardingDraft) {}

#[cfg(target_arch = "wasm32")]
fn load_pending_onboarding_draft() -> Option<PendingOnboardingDraft> {
    let window = web_sys::window()?;
    let storage = window.local_storage().ok()??;
    let serialized = storage.get_item(PENDING_ONBOARDING_DRAFT_KEY).ok()??;
    serde_json::from_str(&serialized).ok()
}

#[cfg(not(target_arch = "wasm32"))]
fn load_pending_onboarding_draft() -> Option<PendingOnboardingDraft> {
    None
}

#[cfg(target_arch = "wasm32")]
fn clear_pending_onboarding_draft() {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.remove_item(PENDING_ONBOARDING_DRAFT_KEY);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn clear_pending_onboarding_draft() {}
