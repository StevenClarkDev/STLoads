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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OtpPurpose {
    Registration,
    PasswordReset,
}

impl OtpPurpose {
    pub const fn as_key(self) -> &'static str {
        match self {
            Self::Registration => "registration",
            Self::PasswordReset => "password_reset",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub name: String,
    pub email: String,
    pub password: String,
    pub password_confirmation: String,
    pub role_key: String,
    pub phone_no: Option<String>,
    pub address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub success: bool,
    pub email: String,
    pub role_key: String,
    pub next_step: String,
    pub message: String,
    pub otp_expires_at: Option<String>,
    pub dev_otp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyOtpRequest {
    pub email: String,
    pub otp: String,
    pub purpose: OtpPurpose,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyOtpResponse {
    pub success: bool,
    pub email: String,
    pub purpose: OtpPurpose,
    pub next_step: String,
    pub message: String,
    pub token: Option<String>,
    pub session: Option<AuthSessionState>,
    pub reset_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResendOtpRequest {
    pub email: String,
    pub purpose: OtpPurpose,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResendOtpResponse {
    pub success: bool,
    pub email: String,
    pub purpose: OtpPurpose,
    pub message: String,
    pub otp_expires_at: Option<String>,
    pub dev_otp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgotPasswordResponse {
    pub success: bool,
    pub email: String,
    pub next_step: String,
    pub message: String,
    pub otp_expires_at: Option<String>,
    pub dev_otp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResetPasswordRequest {
    pub email: String,
    pub reset_token: String,
    pub password: String,
    pub password_confirmation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResetPasswordResponse {
    pub success: bool,
    pub email: String,
    pub next_step: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthOnboardingDraft {
    pub company_name: Option<String>,
    pub company_address: Option<String>,
    pub dot_number: Option<String>,
    pub mc_number: Option<String>,
    pub equipment_types: Option<String>,
    pub business_entity_id: Option<String>,
    pub facility_address: Option<String>,
    pub fulfillment_contact_info: Option<String>,
    pub fmcsa_broker_license_no: Option<String>,
    pub mc_authority_number: Option<String>,
    pub freight_forwarder_license: Option<String>,
    pub customs_license: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthOnboardingScreen {
    pub title: String,
    pub subtitle: String,
    pub role_key: String,
    pub role_label: String,
    pub status_label: String,
    pub can_submit: bool,
    pub requires_otp: bool,
    pub draft: AuthOnboardingDraft,
    pub documents: Vec<KycDocumentItem>,
    pub required_fields: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitOnboardingRequest {
    pub company_name: String,
    pub company_address: String,
    pub dot_number: Option<String>,
    pub mc_number: Option<String>,
    pub equipment_types: Option<String>,
    pub business_entity_id: Option<String>,
    pub facility_address: Option<String>,
    pub fulfillment_contact_info: Option<String>,
    pub fmcsa_broker_license_no: Option<String>,
    pub mc_authority_number: Option<String>,
    pub freight_forwarder_license: Option<String>,
    pub customs_license: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitOnboardingResponse {
    pub success: bool,
    pub session: Option<AuthSessionState>,
    pub next_step: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KycDocumentItem {
    pub id: u64,
    pub document_name: String,
    pub document_type: String,
    pub file_label: String,
    pub original_name: Option<String>,
    pub mime_type: Option<String>,
    pub file_size_bytes: Option<u64>,
    pub uploaded_at_label: String,
    pub download_path: Option<String>,
    pub can_view_file: bool,
    pub blockchain_label: Option<String>,
    pub blockchain_tone: Option<String>,
    pub blockchain_hash_preview: Option<String>,
    pub blockchain_hash: Option<String>,
    pub can_edit: bool,
    pub can_verify_blockchain: bool,
    pub can_delete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminOnboardingReviewUser {
    pub user_id: u64,
    pub name: String,
    pub email: String,
    pub role_label: String,
    pub status_label: String,
    pub company_name: Option<String>,
    pub company_address: Option<String>,
    pub submitted_at_label: String,
    pub document_count: u64,
    pub documents: Vec<KycDocumentItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminOnboardingReviewScreen {
    pub title: String,
    pub summary: String,
    pub users: Vec<AdminOnboardingReviewUser>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewOnboardingRequest {
    pub decision: String,
    pub remarks: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewOnboardingResponse {
    pub success: bool,
    pub user_id: u64,
    pub status_label: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminUserDirectoryRoleOption {
    pub key: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminUserDirectoryStatusOption {
    pub key: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminUserDirectoryUser {
    pub user_id: u64,
    pub name: String,
    pub email: String,
    pub role_key: String,
    pub role_label: String,
    pub status_key: String,
    pub status_label: String,
    pub company_name: Option<String>,
    pub phone_no: Option<String>,
    pub joined_at_label: String,
    pub document_count: u64,
    pub latest_review_note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminUserDirectoryScreen {
    pub title: String,
    pub summary: String,
    pub role_options: Vec<AdminUserDirectoryRoleOption>,
    pub status_options: Vec<AdminUserDirectoryStatusOption>,
    pub users: Vec<AdminUserDirectoryUser>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminUserProfileFact {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminUserHistoryItem {
    pub status_label: String,
    pub remarks: Option<String>,
    pub created_at_label: String,
    pub admin_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminUserProfileScreen {
    pub user_id: u64,
    pub name: String,
    pub email: String,
    pub role_key: String,
    pub role_label: String,
    pub status_key: String,
    pub status_label: String,
    pub phone_no: Option<String>,
    pub address: Option<String>,
    pub dob_label: Option<String>,
    pub gender: Option<String>,
    pub joined_at_label: String,
    pub company_name: Option<String>,
    pub company_address: Option<String>,
    pub image_path: Option<String>,
    pub personal_facts: Vec<AdminUserProfileFact>,
    pub company_facts: Vec<AdminUserProfileFact>,
    pub documents: Vec<KycDocumentItem>,
    pub history: Vec<AdminUserHistoryItem>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminRolePermissionOption {
    pub key: String,
    pub label: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminRolePermissionRow {
    pub role_key: String,
    pub role_label: String,
    pub assigned_permission_keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminRolePermissionScreen {
    pub title: String,
    pub summary: String,
    pub permissions: Vec<AdminRolePermissionOption>,
    pub roles: Vec<AdminRolePermissionRow>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminUpdateUserRequest {
    pub role_key: String,
    pub status_key: String,
    pub remarks: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminUpdateUserResponse {
    pub success: bool,
    pub user_id: u64,
    pub role_label: Option<String>,
    pub status_label: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminCreateUserRequest {
    pub name: String,
    pub email: String,
    pub password: String,
    pub password_confirmation: String,
    pub role_key: String,
    pub status_key: String,
    pub phone_no: Option<String>,
    pub address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminCreateUserResponse {
    pub success: bool,
    pub user_id: Option<u64>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminUpdateUserProfileRequest {
    pub name: String,
    pub email: String,
    pub password: Option<String>,
    pub password_confirmation: Option<String>,
    pub phone_no: Option<String>,
    pub address: Option<String>,
    pub remarks: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminUpdateUserProfileResponse {
    pub success: bool,
    pub user_id: u64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminDeleteUserResponse {
    pub success: bool,
    pub user_id: u64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfProfileDraft {
    pub name: String,
    pub email: String,
    pub phone_no: Option<String>,
    pub address: Option<String>,
    pub company_name: Option<String>,
    pub dot_number: Option<String>,
    pub mc_number: Option<String>,
    pub mc_cbsa_usdot_no: Option<String>,
    pub ucr_hcc_no: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfProfileFact {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfProfileScreen {
    pub title: String,
    pub role_key: String,
    pub role_label: String,
    pub status_label: String,
    pub draft: SelfProfileDraft,
    pub personal_facts: Vec<SelfProfileFact>,
    pub company_facts: Vec<SelfProfileFact>,
    pub documents: Vec<KycDocumentItem>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSelfProfileRequest {
    pub name: String,
    pub email: String,
    pub phone_no: Option<String>,
    pub address: Option<String>,
    pub company_name: Option<String>,
    pub dot_number: Option<String>,
    pub mc_number: Option<String>,
    pub mc_cbsa_usdot_no: Option<String>,
    pub ucr_hcc_no: Option<String>,
    pub password: Option<String>,
    pub password_confirmation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSelfProfileResponse {
    pub success: bool,
    pub message: String,
    pub session: Option<AuthSessionState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub password: String,
    pub password_confirmation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangePasswordResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertKycDocumentRequest {
    pub document_name: String,
    pub document_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertKycDocumentResponse {
    pub success: bool,
    pub document_id: Option<u64>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyKycDocumentRequest {
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyKycDocumentResponse {
    pub success: bool,
    pub document_id: u64,
    pub hash: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteKycDocumentResponse {
    pub success: bool,
    pub document_id: u64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminUpdateRolePermissionsRequest {
    pub permission_keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminUpdateRolePermissionsResponse {
    pub success: bool,
    pub role_key: String,
    pub assigned_permission_keys: Vec<String>,
    pub message: String,
}
