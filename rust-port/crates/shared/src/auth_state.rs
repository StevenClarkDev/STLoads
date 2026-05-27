use serde::{Deserialize, Serialize};

use crate::RequiredDocumentChecklistItem;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSessionUser {
    pub id: u64,
    pub name: String,
    pub email: String,
    pub organization_id: Option<u64>,
    pub organization_role_key: Option<String>,
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
    pub mfa_required: bool,
    pub mfa_challenge_id: Option<String>,
    pub mfa_expires_at: Option<String>,
    pub next_step: Option<String>,
    pub dev_mfa_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseSsoDiscoveryRequest {
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseSsoDiscoveryResponse {
    pub email_domain: Option<String>,
    pub organization_id: Option<u64>,
    pub organization_name: Option<String>,
    pub sso_required: bool,
    pub password_allowed: bool,
    pub provider_type: Option<String>,
    pub provider_display_name: Option<String>,
    pub sso_url: Option<String>,
    pub jit_enabled: bool,
    pub default_role_key: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseSsoOidcCallbackRequest {
    pub email: String,
    pub id_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseSsoLoginResponse {
    pub success: bool,
    pub token: Option<String>,
    pub session: Option<AuthSessionState>,
    pub created_user: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimDeprovisionRequest {
    pub external_id: Option<String>,
    pub email: Option<String>,
    pub user_id: Option<u64>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimDeprovisionResponse {
    pub success: bool,
    pub organization_id: Option<u64>,
    pub user_id: Option<u64>,
    pub revoked_sessions: u64,
    pub membership_rows_changed: u64,
    pub user_rows_changed: u64,
    pub event_id: Option<u64>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimUpsertUserRequest {
    pub external_id: String,
    pub email: String,
    pub name: String,
    pub role_key: Option<String>,
    pub active: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimUpsertUserResponse {
    pub success: bool,
    pub organization_id: Option<u64>,
    pub user_id: Option<u64>,
    pub created: bool,
    pub reactivated: bool,
    pub revoked_sessions: u64,
    pub event_id: Option<u64>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminIdentityScreen {
    pub title: String,
    pub target_organization_id: u64,
    pub domains: Vec<AdminIdentityDomainRow>,
    pub providers: Vec<AdminIdentityProviderRow>,
    pub scim_events: Vec<AdminIdentityScimEventRow>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminIdentityDomainRow {
    pub id: u64,
    pub domain: String,
    pub verification_status: String,
    pub verification_token: String,
    pub login_routing_enabled: bool,
    pub verified_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminIdentityProviderRow {
    pub id: u64,
    pub provider_type: String,
    pub status: String,
    pub display_name: String,
    pub issuer: Option<String>,
    pub sso_url: Option<String>,
    pub jwks_url: Option<String>,
    pub metadata_url: Option<String>,
    pub client_id: Option<String>,
    pub jit_enabled: bool,
    pub default_role_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminIdentityScimEventRow {
    pub id: u64,
    pub action: String,
    pub outcome: String,
    pub external_id: Option<String>,
    pub user_id: Option<u64>,
    pub reason: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminUpsertIdentityDomainRequest {
    pub target_organization_id: Option<u64>,
    pub domain: String,
    pub login_routing_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminVerifyIdentityDomainRequest {
    pub target_organization_id: Option<u64>,
    pub domain: String,
    pub verification_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminCheckIdentityDomainDnsRequest {
    pub target_organization_id: Option<u64>,
    pub domain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminUpsertIdentityProviderRequest {
    pub target_organization_id: Option<u64>,
    pub provider_id: Option<u64>,
    pub provider_type: String,
    pub status: String,
    pub display_name: String,
    pub issuer: Option<String>,
    pub sso_url: Option<String>,
    pub jwks_url: Option<String>,
    pub metadata_url: Option<String>,
    pub client_id: Option<String>,
    pub jit_enabled: bool,
    pub default_role_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminIdentityMutationResponse {
    pub success: bool,
    pub message: String,
    pub screen: AdminIdentityScreen,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogoutResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaVerifyRequest {
    pub email: String,
    pub challenge_id: String,
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaVerifyResponse {
    pub success: bool,
    pub email: String,
    pub token: Option<String>,
    pub session: Option<AuthSessionState>,
    pub recovery_codes: Vec<String>,
    pub message: String,
    pub next_step: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaRecoveryCodesResponse {
    pub success: bool,
    pub recovery_codes: Vec<String>,
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
pub struct PortalRoleCountsResponse {
    pub shipper_total: u64,
    pub carrier_total: u64,
    pub broker_total: u64,
    pub freight_forwarder_total: u64,
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
    pub required_documents: Vec<RequiredDocumentChecklistItem>,
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
pub struct LegalAgreementTemplateItem {
    pub id: u64,
    pub agreement_key: String,
    pub version: String,
    pub title: String,
    pub document_uri: Option<String>,
    pub required_role_key: Option<String>,
    pub requires_user_acceptance: bool,
    pub requires_organization_acceptance: bool,
    pub effective_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalAgreementAcceptanceItem {
    pub id: u64,
    pub agreement_key: String,
    pub version: String,
    pub signer_name: String,
    pub signer_email: String,
    pub accepted_at: String,
    pub audit_event_id: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalAgreementScreen {
    pub title: String,
    pub missing_required: Vec<LegalAgreementTemplateItem>,
    pub acceptance_proofs: Vec<LegalAgreementAcceptanceItem>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptLegalAgreementRequest {
    pub agreement_key: String,
    pub accept_for_organization: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptLegalAgreementResponse {
    pub success: bool,
    pub message: String,
    pub screen: LegalAgreementScreen,
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
    pub current_version: u32,
    pub version_count: u64,
    pub version_history_label: String,
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
pub struct AdminBreakGlassRequest {
    pub target_organization_id: u64,
    pub reason: String,
    pub ticket_ref: String,
    pub duration_minutes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminBreakGlassResponse {
    pub success: bool,
    pub session_id: Option<String>,
    pub target_organization_id: u64,
    pub expires_at: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminSupportSearchScreen {
    pub title: String,
    pub query: String,
    pub target_organization_id: Option<u64>,
    pub result_count: u64,
    pub results: Vec<AdminSupportSearchResult>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminSupportSearchResult {
    pub category: String,
    pub id: u64,
    pub label: String,
    pub detail: String,
    pub organization_id: Option<u64>,
    pub href: Option<String>,
    pub facts: Vec<AdminSupportSearchFact>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminSupportSearchFact {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AdminAuditSearchFilters {
    pub q: Option<String>,
    pub target_organization_id: Option<u64>,
    pub actor_user_id: Option<u64>,
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
    pub action: Option<String>,
    pub request_id: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminAuditSearchScreen {
    pub title: String,
    pub target_organization_id: u64,
    pub filters: AdminAuditSearchFilters,
    pub result_count: u64,
    pub rows: Vec<AdminAuditEventRow>,
    pub export_path: String,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminAuditEventRow {
    pub id: u64,
    pub actor_user_id: Option<u64>,
    pub actor_label: String,
    pub organization_id: Option<u64>,
    pub target_organization_id: Option<u64>,
    pub entity_type: String,
    pub entity_id: Option<String>,
    pub action: String,
    pub reason: Option<String>,
    pub ticket_ref: Option<String>,
    pub request_id: Option<String>,
    pub source: String,
    pub metadata_preview: Option<String>,
    pub before_after_label: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminAuditExportResponse {
    pub filename: String,
    pub content_type: String,
    pub row_count: u64,
    pub csv: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminSupportTimelineRequest {
    pub target_organization_id: Option<u64>,
    pub entity_type: String,
    pub entity_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminCreateSupportNoteRequest {
    pub target_organization_id: Option<u64>,
    pub entity_type: String,
    pub entity_id: Option<String>,
    pub visibility: String,
    pub ticket_ref: Option<String>,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminSupportTimelineScreen {
    pub title: String,
    pub target_organization_id: u64,
    pub entity_type: String,
    pub entity_id: Option<String>,
    pub entries: Vec<AdminSupportTimelineEntry>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminSupportTimelineEntry {
    pub source: String,
    pub action: String,
    pub actor_user_id: Option<u64>,
    pub visibility: String,
    pub ticket_ref: Option<String>,
    pub summary: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminCreateSupportNoteResponse {
    pub success: bool,
    pub note_id: Option<u64>,
    pub message: String,
    pub timeline: AdminSupportTimelineScreen,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminSupportCaseRow {
    pub id: u64,
    pub case_number: String,
    pub organization_id: u64,
    pub title: String,
    pub severity: String,
    pub status: String,
    pub channel: String,
    pub category: String,
    pub owner_team: String,
    pub breach_state: String,
    pub first_response_due_at: String,
    pub next_update_due_at: String,
    pub resolution_due_at: String,
    pub related_entity_type: Option<String>,
    pub related_entity_id: Option<String>,
    pub customer_impact: String,
    pub resolution_reason: Option<String>,
    pub feedback_score: Option<i32>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminSupportCaseScreen {
    pub title: String,
    pub target_organization_id: u64,
    pub open_count: u64,
    pub breach_risk_count: u64,
    pub breached_count: u64,
    pub rows: Vec<AdminSupportCaseRow>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminCreateSupportCaseRequest {
    pub target_organization_id: Option<u64>,
    pub reporter_user_id: Option<u64>,
    pub affected_user_id: Option<u64>,
    pub related_entity_type: Option<String>,
    pub related_entity_id: Option<String>,
    pub channel: String,
    pub severity: String,
    pub category: String,
    pub owner_team: String,
    pub title: String,
    pub description: String,
    pub customer_impact: String,
    pub customer_update: Option<String>,
    pub internal_note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminUpdateSupportCaseRequest {
    pub status: Option<String>,
    pub severity: Option<String>,
    pub owner_team: Option<String>,
    pub owner_user_id: Option<u64>,
    pub escalation_owner_user_id: Option<u64>,
    pub customer_update: Option<String>,
    pub internal_note: Option<String>,
    pub resolution_reason: Option<String>,
    pub root_cause_category: Option<String>,
    pub follow_up_action: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminSupportCaseFeedbackRequest {
    pub feedback_score: i32,
    pub feedback_comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminSupportCaseMutationResponse {
    pub success: bool,
    pub message: String,
    pub case_row: Option<AdminSupportCaseRow>,
    pub screen: AdminSupportCaseScreen,
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
pub struct CarrierCapacityProfile {
    pub equipment_types: Vec<String>,
    pub lane_preferences: Vec<String>,
    pub operating_regions: Vec<String>,
    pub preferred_commodities: Vec<String>,
    pub service_levels: Vec<String>,
    pub certifications: Vec<String>,
    pub availability_status: String,
    pub available_power_units: u32,
    pub insurance_limit_usd: f64,
    pub capacity_notes: Option<String>,
    pub readiness_label: String,
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
    pub carrier_capacity: Option<CarrierCapacityProfile>,
    pub documents: Vec<KycDocumentItem>,
    pub required_documents: Vec<RequiredDocumentChecklistItem>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCarrierCapacityRequest {
    pub equipment_types: Vec<String>,
    pub lane_preferences: Vec<String>,
    pub operating_regions: Vec<String>,
    pub preferred_commodities: Vec<String>,
    pub service_levels: Vec<String>,
    pub certifications: Vec<String>,
    pub availability_status: String,
    pub available_power_units: u32,
    pub insurance_limit_usd: f64,
    pub capacity_notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCarrierCapacityResponse {
    pub success: bool,
    pub message: String,
    pub capacity: Option<CarrierCapacityProfile>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminAccessReviewRow {
    pub id: u64,
    pub title: String,
    pub status: String,
    pub review_type: String,
    pub due_at_label: Option<String>,
    pub completed_at_label: Option<String>,
    pub created_at_label: String,
    pub pending_count: u64,
    pub approved_count: u64,
    pub exception_count: u64,
    pub revoke_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminAccessReviewItemRow {
    pub id: u64,
    pub review_id: u64,
    pub user_id: u64,
    pub user_name: String,
    pub user_email: String,
    pub role_key: String,
    pub role_label: String,
    pub account_status_label: String,
    pub membership_status: Option<String>,
    pub last_activity_label: Option<String>,
    pub risk_flags: Vec<String>,
    pub decision: String,
    pub decision_reason: Option<String>,
    pub decided_at_label: Option<String>,
    pub revoked_at_label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminAccessReviewScreen {
    pub title: String,
    pub summary: String,
    pub target_organization_id: u64,
    pub reviews: Vec<AdminAccessReviewRow>,
    pub items: Vec<AdminAccessReviewItemRow>,
    pub elevation_requests: Vec<AdminAccessElevationRequestRow>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminAccessElevationRequestRow {
    pub id: u64,
    pub requester_user_id: u64,
    pub requester_email: String,
    pub target_user_id: u64,
    pub target_email: String,
    pub current_role_key: Option<String>,
    pub requested_role_key: String,
    pub status: String,
    pub business_justification: String,
    pub decision_reason: Option<String>,
    pub decided_at_label: Option<String>,
    pub expires_at_label: Option<String>,
    pub created_at_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminStartAccessReviewRequest {
    pub target_organization_id: Option<u64>,
    pub title: String,
    pub due_days: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminAccessReviewDecisionRequest {
    pub target_organization_id: Option<u64>,
    pub decision: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminCreateAccessElevationRequest {
    pub target_organization_id: Option<u64>,
    pub target_user_id: u64,
    pub requested_role_key: String,
    pub business_justification: String,
    pub expires_in_days: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminAccessElevationDecisionRequest {
    pub target_organization_id: Option<u64>,
    pub decision: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminAccessReviewMutationResponse {
    pub success: bool,
    pub message: String,
    pub screen: AdminAccessReviewScreen,
}
