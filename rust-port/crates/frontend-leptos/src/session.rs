use leptos::{prelude::*, task::spawn_local};
use shared::{AuthSessionState, LoginRequest, LoginResponse, LogoutResponse};

use crate::api;

#[derive(Clone, Copy)]
pub struct AuthContext {
    pub session: RwSignal<AuthSessionState>,
    pub session_ready: RwSignal<bool>,
    pub session_loading: RwSignal<bool>,
    pub notice: RwSignal<Option<String>>,
}

#[component]
pub fn AuthProvider(children: Children) -> impl IntoView {
    let auth = AuthContext {
        session: RwSignal::new(unauthenticated_session(
            "Loading Rust session state for this app shell.",
        )),
        session_ready: RwSignal::new(false),
        session_loading: RwSignal::new(false),
        notice: RwSignal::new(None),
    };

    provide_context(auth.clone());

    Effect::new(move |_| {
        if auth.session_ready.get() || auth.session_loading.get() {
            return;
        }

        let auth = auth.clone();
        spawn_local(async move {
            let _ = refresh_session(auth).await;
        });
    });

    view! { {children()} }
}

pub fn use_auth() -> AuthContext {
    use_context::<AuthContext>().expect("AuthContext should be provided at the app root")
}

pub async fn refresh_session(auth: AuthContext) -> Result<AuthSessionState, String> {
    auth.session_loading.set(true);

    let result = api::fetch_auth_session().await;
    auth.session_loading.set(false);
    auth.session_ready.set(true);

    match result {
        Ok(session) => {
            auth.session.set(session.clone());
            if session.authenticated {
                auth.notice.set(None);
            }
            Ok(session)
        }
        Err(error) => {
            if error.contains("returned 401") {
                api::clear_auth_token();
                auth.session.set(unauthenticated_session(
                    "The Rust session expired. Sign in again.",
                ));
            } else {
                auth.session.set(unauthenticated_session(
                    "Unable to load Rust session state from the backend.",
                ));
            }
            auth.notice.set(Some(error.clone()));
            Err(error)
        }
    }
}

pub async fn sign_in(auth: AuthContext, payload: LoginRequest) -> Result<LoginResponse, String> {
    auth.session_loading.set(true);
    let result = api::login(&payload).await;
    auth.session_loading.set(false);
    auth.session_ready.set(true);

    match result {
        Ok(response) => {
            auth.session.set(response.session.clone());
            auth.notice.set(Some(response.message.clone()));
            Ok(response)
        }
        Err(error) => {
            auth.notice.set(Some(error.clone()));
            Err(error)
        }
    }
}

pub async fn sign_out(auth: AuthContext) -> Result<LogoutResponse, String> {
    auth.session_loading.set(true);
    let result = api::logout().await;
    auth.session_loading.set(false);
    auth.session_ready.set(true);

    match result {
        Ok(response) => {
            if response.success {
                auth.session.set(unauthenticated_session(
                    "Signed out from the Rust session layer.",
                ));
            }
            auth.notice.set(Some(response.message.clone()));
            Ok(response)
        }
        Err(error) => {
            auth.notice.set(Some(error.clone()));
            Err(error)
        }
    }
}

pub fn invalidate_session(auth: &AuthContext, note: &str) {
    api::clear_auth_token();
    auth.session.set(unauthenticated_session(note));
    auth.session_ready.set(true);
    auth.notice.set(Some(note.to_string()));
}

pub fn clear_notice(auth: &AuthContext) {
    auth.notice.set(None);
}

pub fn has_permission(auth: &AuthContext, permission_key: &str) -> bool {
    auth.session
        .get()
        .permissions
        .iter()
        .any(|permission| permission == permission_key)
}

fn unauthenticated_session(note: &str) -> AuthSessionState {
    AuthSessionState {
        authenticated: false,
        user: None,
        permissions: Vec::new(),
        notes: vec![note.to_string()],
    }
}
