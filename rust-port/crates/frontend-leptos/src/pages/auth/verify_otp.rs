use leptos::{ev, prelude::*, task::spawn_local};
use leptos_router::{
    components::A,
    hooks::{use_navigate, use_query_map},
};

use crate::{api, session::use_auth};
use shared::{OtpPurpose, VerifyOtpRequest};

use crate::pages::auth_helpers::*;
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
            if email.get_untracked().is_empty()
                && let Some(prefill) = params.get("email")
            {
                email.set(prefill.clone());
            }
            if reset_token.get_untracked().is_empty()
                && let Some(token) = params.get("token")
            {
                reset_token.set(token.clone());
            }
            if let Some(value) = params.get("purpose")
                && let Some(next_purpose) = otp_purpose_from_query(&value)
            {
                purpose.set(next_purpose);
            }
        });
    });

    let on_submit = move |ev: ev::SubmitEvent| {
        let navigate = navigate.clone();
        let auth = auth;
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
        let auth = auth;
        let email_value = email.get().trim().to_string();
        if email_value.is_empty() {
            feedback.set(Some(
                "Enter the email address first so we know where to resend the OTP.".into(),
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
                    let message = response.message;
                    if let Some(dev_otp) = response.dev_otp {
                        let _ = dev_otp;
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
