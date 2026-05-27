use leptos::{ev, prelude::*, task::spawn_local};
use leptos_router::{
    components::A,
    hooks::{use_navigate, use_query_map},
};

use crate::{api, session::use_auth};
use shared::MfaVerifyRequest;

use crate::pages::auth_helpers::*;
#[component]
pub fn MfaPage() -> impl IntoView {
    let navigate = use_navigate();
    let query = use_query_map();
    let auth = use_auth();
    let email = RwSignal::new(String::new());
    let challenge_id = RwSignal::new(String::new());
    let code = RwSignal::new(String::new());
    let feedback = RwSignal::new(None::<String>);
    let is_submitting = RwSignal::new(false);

    Effect::new(move |_| {
        query.with(|params| {
            if email.get_untracked().is_empty()
                && let Some(value) = params.get("email")
            {
                email.set(value.clone());
            }
            if challenge_id.get_untracked().is_empty()
                && let Some(value) = params.get("challenge")
            {
                challenge_id.set(value.clone());
            }
        });
    });

    let on_submit = move |ev: ev::SubmitEvent| {
        let navigate = navigate.clone();
        let auth = auth;
        ev.prevent_default();
        feedback.set(None);

        let payload = MfaVerifyRequest {
            email: email.get().trim().to_string(),
            challenge_id: challenge_id.get().trim().to_string(),
            code: code.get().trim().to_string(),
        };

        if payload.email.is_empty() || payload.challenge_id.is_empty() || payload.code.is_empty() {
            feedback.set(Some("Email, challenge, and MFA code are required.".into()));
            return;
        }

        is_submitting.set(true);
        spawn_local(async move {
            match api::verify_mfa(&payload).await {
                Ok(response) => {
                    auth.notice.set(Some(response.message.clone()));
                    feedback.set(Some(response.message.clone()));
                    if response.success {
                        if let Some(session) = response.session.clone() {
                            auth.session.set(session);
                            auth.session_ready.set(true);
                        }
                        navigate(&response.next_step, Default::default());
                    }
                }
                Err(error) => {
                    auth.notice.set(Some(error.clone()));
                    feedback.set(Some(error));
                }
            }
            is_submitting.set(false);
        });
    };

    view! {
        <AuthArticle
            title=Signal::derive(|| "MFA Verification".to_string())
            subtitle=Signal::derive(|| "Privileged accounts require a second verification step before a session starts.".to_string())
        >
            <LocalNotice message=feedback />
            <form on:submit=on_submit class="auth-form">
                <TextField label="Email" value=email input_type="email" placeholder="name@example.com" />
                <TextAreaField label="Challenge" value=challenge_id placeholder="Challenge id from login" />
                <TextField label="MFA code" value=code input_type="text" placeholder="Enter the 6-digit code or recovery code" />
                <div class="auth-actions">
                    <nav class="auth-links">
                        <A href="/auth/login" attr:class="auth-link">"Back to login"</A>
                    </nav>
                    <button type="submit" style=button_style("#111827") disabled=move || is_submitting.get()>
                        {move || if is_submitting.get() { "Verifying..." } else { "Verify MFA" }}
                    </button>
                </div>
            </form>
        </AuthArticle>
    }
}
