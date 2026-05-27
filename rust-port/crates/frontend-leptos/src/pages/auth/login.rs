use leptos::{ev, prelude::*, task::spawn_local};
use leptos_router::{
    components::A,
    hooks::{use_location, use_navigate, use_query_map},
};

use crate::session::{self, use_auth};
use shared::LoginRequest;

use crate::pages::auth_helpers::*;
#[component]
pub fn LoginPage() -> impl IntoView {
    let navigate = use_navigate();
    let location = use_location();
    let query = use_query_map();
    let auth = use_auth();
    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let is_submitting = RwSignal::new(false);
    let is_admin_portal = Memo::new(move |_| {
        let admin_from_query = query.with(|params| {
            params
                .get("portal")
                .map(|value| value == "admin")
                .unwrap_or(false)
        });
        admin_from_query || location.pathname.get() == "/admin-login"
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
            let login_email = email_value.clone();
            let result = session::sign_in(
                auth,
                LoginRequest {
                    email: email_value,
                    password: password_value,
                },
            )
            .await;

            if let Ok(response) = result {
                if response.mfa_required {
                    if let Some(challenge_id) = response.mfa_challenge_id {
                        navigate(
                            &build_mfa_path(&login_email, &challenge_id),
                            Default::default(),
                        );
                    }
                    is_submitting.set(false);
                    return;
                }
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
