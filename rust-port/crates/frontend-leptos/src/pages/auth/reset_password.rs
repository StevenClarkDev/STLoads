use leptos::{ev, prelude::*, task::spawn_local};
use leptos_router::{
    components::A,
    hooks::{use_navigate, use_query_map},
};

use crate::{api, session::use_auth};
use shared::ResetPasswordRequest;

use crate::pages::auth_helpers::*;
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
            if email.get_untracked().is_empty()
                && let Some(prefill) = params.get("email")
            {
                email.set(prefill.clone());
            }
            if reset_token.get_untracked().is_empty()
                && let Some(prefill) = params.get("token")
            {
                reset_token.set(prefill.clone());
            }
        });
    });

    let on_submit = move |ev: ev::SubmitEvent| {
        let navigate = navigate.clone();
        let auth = auth;
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
                "Complete your password reset after OTP verification by setting a fresh password here.".to_string()
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
