use leptos::{ev, prelude::*, task::spawn_local};
use leptos_router::{
    components::A,
    hooks::{use_navigate, use_query_map},
};

use crate::{api, session::use_auth};
use shared::{ForgotPasswordRequest, OtpPurpose};

use crate::pages::auth_helpers::*;
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
            if email.get_untracked().is_empty()
                && let Some(prefill) = params.get("email")
            {
                email.set(prefill.clone());
            }
        });
    });

    let on_submit = move |ev: ev::SubmitEvent| {
        let navigate = navigate.clone();
        let auth = auth;
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
                    let message = response.message;
                    if let Some(dev_otp) = response.dev_otp {
                        let _ = dev_otp;
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
                "Use OTP verification first, then set a new password.".to_string()
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
