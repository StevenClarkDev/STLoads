use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSessionUser {
    pub id: u64,
    pub name: String,
    pub email: String,
    pub role_key: String,
    pub role_label: String,
    pub account_status_label: String,
    pub dashboard_href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSessionState {
    pub authenticated: bool,
    pub user: Option<AuthSessionUser>,
    pub permissions: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub success: bool,
    pub token: Option<String>,
    pub session: AuthSessionState,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogoutResponse {
    pub success: bool,
    pub message: String,
}
