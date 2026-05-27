use serde::{Deserialize, Serialize, de::DeserializeOwned};
use shared::{
    AcceptLegalAgreementRequest, AcceptLegalAgreementResponse, AdminAccessElevationDecisionRequest,
    AdminAccessReviewDecisionRequest, AdminAccessReviewMutationResponse, AdminAccessReviewScreen,
    AdminAuditExportResponse, AdminAuditSearchFilters, AdminAuditSearchScreen,
    AdminCheckIdentityDomainDnsRequest, AdminCreateAccessElevationRequest,
    AdminCreateSupportCaseRequest, AdminCreateSupportNoteRequest, AdminCreateSupportNoteResponse,
    AdminCreateUserRequest, AdminCreateUserResponse, AdminDeleteUserResponse,
    AdminIdentityMutationResponse, AdminIdentityScreen, AdminLoadListScreen,
    AdminOnboardingReviewScreen, AdminReviewLoadRequest, AdminReviewLoadResponse,
    AdminRolePermissionScreen, AdminStartAccessReviewRequest, AdminSupportCaseFeedbackRequest,
    AdminSupportCaseMutationResponse, AdminSupportCaseScreen, AdminSupportSearchScreen,
    AdminSupportTimelineScreen, AdminUpdateRolePermissionsRequest,
    AdminUpdateRolePermissionsResponse, AdminUpdateSupportCaseRequest,
    AdminUpdateUserProfileRequest, AdminUpdateUserProfileResponse, AdminUpdateUserRequest,
    AdminUpdateUserResponse, AdminUpsertIdentityDomainRequest, AdminUpsertIdentityProviderRequest,
    AdminUserDirectoryScreen, AdminUserProfileScreen, AdminVerifyIdentityDomainRequest,
    ApiResponse, AuthOnboardingScreen, AuthSessionState, BookLoadLegRequest, BookLoadLegResponse,
    BulkLoadImportCommitRequest, BulkLoadImportPreviewRequest, BulkLoadImportResponse,
    CarrierMatchScreen, ChangePasswordRequest, ChangePasswordResponse, ChatSendMessageRequest,
    ChatSendMessageResponse, ChatWorkspaceScreen, CityUpsertRequest, ConversationReadResponse,
    CountryUpsertRequest, CreateLoadRequest, CreateLoadResponse,
    CustomerConfigurationRuleUpsertRequest, DeleteKycDocumentResponse, DispatchDeskFollowUpRequest,
    DispatchDeskFollowUpResponse, DispatchDeskScreen, DocumentRequirementRuleUpsertRequest,
    EscrowFundRequest, EscrowHoldRequest, EscrowLifecycleResponse, EscrowReleaseRequest,
    ExecutionLegActionRequest, ExecutionLegActionResponse, ExecutionLegScreen,
    ExecutionLocationPingRequest, ExecutionLocationPingResponse, FacilityAppointmentRequest,
    FacilityAppointmentResponse, ForgotPasswordRequest, ForgotPasswordResponse,
    GenerateFreightDocumentsRequest, GenerateFreightDocumentsResponse,
    GovernedCatalogUpsertRequest, LegalAgreementScreen, LoadBoardFilters, LoadBoardScreen,
    LoadBuilderScreen, LoadLifecycleActionRequest, LoadLifecycleActionResponse, LoadProfileScreen,
    LocationUpsertRequest, LoginRequest, LoginResponse, LogoutResponse, MasterDataDeleteRequest,
    MasterDataExportResponse, MasterDataImportRequest, MasterDataMutationResponse,
    MasterDataRollbackRequest, MasterDataScreen, MfaVerifyRequest, MfaVerifyResponse,
    OfferCounterRequest, OfferCounterResponse, OfferReviewRequest, OfferReviewResponse,
    PortalRoleCountsResponse, RateCalculationRequest, RateCalculationResponse,
    RateConfirmationResponse, RealtimeTopic, RegisterRequest, RegisterResponse, ResendOtpRequest,
    ResendOtpResponse, ResetPasswordRequest, ResetPasswordResponse,
    ResolveDispatchExceptionRequest, ResolveDispatchExceptionResponse, ResolveSyncErrorRequest,
    ResolveSyncErrorResponse, ReviewOnboardingRequest, ReviewOnboardingResponse, SelfProfileScreen,
    SimpleCatalogUpsertRequest, StloadsOperationsScreen, StloadsReconciliationScreen,
    StripeWebhookRequest, StripeWebhookResponse, SubmitOnboardingRequest, SubmitOnboardingResponse,
    TmsCloseRequest, TmsHandoffPayload, TmsHandoffResponse, TmsRequeueRequest,
    TmsStatusWebhookRequest, TmsWebhookResponse, TmsWithdrawRequest, UpdateCarrierCapacityRequest,
    UpdateCarrierCapacityResponse, UpdateSelfProfileRequest, UpdateSelfProfileResponse,
    UpsertCarrierNetworkRequest, UpsertCarrierNetworkResponse, UpsertKycDocumentRequest,
    UpsertKycDocumentResponse, UpsertLoadDocumentRequest, UpsertLoadDocumentResponse,
    VerifyKycDocumentRequest, VerifyKycDocumentResponse, VerifyLoadDocumentRequest,
    VerifyLoadDocumentResponse, VerifyOtpRequest, VerifyOtpResponse,
};

#[cfg(target_arch = "wasm32")]
use crate::runtime_config;
#[derive(Debug, Clone, Deserialize)]
pub struct AdminOverview {
    pub screen_routes: Vec<String>,
    pub operational_views: usize,
    pub user_total: usize,
    pub shipper_total: usize,
    pub carrier_total: usize,
    pub broker_total: usize,
    pub freight_forwarder_total: usize,
    pub admin_total: usize,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaymentsOverview {
    pub contract: serde_json::Value,
    pub escrow_statuses: usize,
    pub webhook_events: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IntegrationPortalScreen {
    pub organization_id: u64,
    pub api_version: String,
    pub docs: Vec<IntegrationDocLink>,
    pub sandbox: IntegrationSandboxSummary,
    pub api_keys: Vec<IntegrationApiKeyRow>,
    pub webhook_endpoints: Vec<IntegrationWebhookEndpointRow>,
    pub recent_deliveries: Vec<IntegrationWebhookDeliveryRow>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IntegrationDocLink {
    pub label: String,
    pub href: String,
    pub description: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IntegrationSandboxSummary {
    pub base_url: String,
    pub production_safety: String,
    pub reset_policy: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IntegrationApiKeyRow {
    pub id: u64,
    pub client_name: String,
    pub key_prefix: String,
    pub scopes: Vec<String>,
    pub status: String,
    pub rate_limit_per_minute: i32,
    pub require_request_signature: bool,
    pub last_used_at: Option<String>,
    pub expires_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IntegrationWebhookEndpointRow {
    pub id: u64,
    pub endpoint_name: String,
    pub target_url: String,
    pub event_types: Vec<String>,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IntegrationWebhookDeliveryRow {
    pub id: u64,
    pub endpoint_id: Option<u64>,
    pub event_type: String,
    pub event_id: String,
    pub delivery_status: String,
    pub attempt_count: i32,
    pub next_retry_at: Option<String>,
    pub last_attempt_at: Option<String>,
    pub response_status_code: Option<i32>,
    pub response_latency_ms: Option<i32>,
    pub response_body_excerpt: Option<String>,
    pub dead_letter_reason: Option<String>,
    pub replay_of_delivery_id: Option<u64>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreatePartnerApiKeyRequest {
    pub client_name: String,
    pub scopes: Vec<String>,
    pub rate_limit_per_minute: Option<i32>,
    pub require_request_signature: Option<bool>,
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreatePartnerApiKeyResponse {
    pub success: bool,
    pub api_key: Option<String>,
    pub key_prefix: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpsertWebhookEndpointRequest {
    pub endpoint_name: String,
    pub target_url: String,
    pub event_types: Vec<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IntegrationMutationResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EdiIntegrationScreen {
    pub supported_transactions: Vec<EdiTransactionMappingRow>,
    pub partner_profiles: Vec<EdiPartnerProfileRow>,
    pub message_logs: Vec<EdiMessageLogRow>,
    pub replay_policy: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EdiTransactionMappingRow {
    pub id: u64,
    pub transaction_code: String,
    pub direction: String,
    pub stloads_model: String,
    pub mapping_version: String,
    pub status: String,
    pub required_fields: Vec<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EdiPartnerProfileRow {
    pub id: u64,
    pub partner_name: String,
    pub isa_id: Option<String>,
    pub gs_id: Option<String>,
    pub transport_type: String,
    pub status: String,
    pub supported_transactions: Vec<String>,
    pub validation_mode: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EdiMessageLogRow {
    pub id: u64,
    pub partner_profile_id: Option<u64>,
    pub transaction_code: String,
    pub direction: String,
    pub control_number: Option<String>,
    pub business_key: Option<String>,
    pub message_status: String,
    pub ack_status: String,
    pub retry_count: i32,
    pub next_retry_at: Option<String>,
    pub error_summary: Option<String>,
    pub replay_of_message_id: Option<u64>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpsertEdiPartnerProfileRequest {
    pub partner_name: String,
    pub isa_id: Option<String>,
    pub gs_id: Option<String>,
    pub transport_type: Option<String>,
    pub status: Option<String>,
    pub supported_transactions: Vec<String>,
    pub validation_mode: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ValidateEdiMessageRequest {
    pub transaction_code: String,
    pub direction: String,
    pub control_number: Option<String>,
    pub business_key: Option<String>,
    pub payload_excerpt: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EdiValidationResponse {
    pub success: bool,
    pub message_id: Option<u64>,
    pub missing_fields: Vec<String>,
    pub ack_status: String,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EdiReplayResponse {
    pub success: bool,
    pub message_id: u64,
    pub replay_message_id: Option<u64>,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SandboxGovernanceScreen {
    pub environments: Vec<SandboxEnvironmentRow>,
    pub reset_jobs: Vec<SandboxResetJobRow>,
    pub policy_notes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SandboxEnvironmentRow {
    pub id: u64,
    pub environment_key: String,
    pub display_name: String,
    pub base_url: String,
    pub data_classification: String,
    pub pii_allowed: bool,
    pub production_payment_blocked: bool,
    pub production_tms_push_blocked: bool,
    pub production_notification_blocked: bool,
    pub seeded_dataset_version: String,
    pub reset_status: String,
    pub last_reset_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SandboxResetJobRow {
    pub id: u64,
    pub sandbox_environment_id: u64,
    pub job_status: String,
    pub reset_reason: Option<String>,
    pub result_summary: Option<String>,
    pub created_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct QueueSandboxResetRequest {
    pub sandbox_environment_id: u64,
    pub reset_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SandboxResetResponse {
    pub success: bool,
    pub reset_job_id: Option<u64>,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiLifecycleScreen {
    pub policies: Vec<ApiLifecyclePolicyRow>,
    pub examples: Vec<ApiPartnerExampleRow>,
    pub upgrade_paths: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiLifecyclePolicyRow {
    pub api_version: String,
    pub release_status: String,
    pub released_on: String,
    pub sunset_on: Option<String>,
    pub minimum_notice_days: i32,
    pub emergency_breaking_change_policy: String,
    pub changelog_url: String,
    pub postman_collection_url: Option<String>,
    pub sdk_strategy: String,
    pub compatibility_test_status: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiPartnerExampleRow {
    pub api_version: String,
    pub example_key: String,
    pub surface: String,
    pub method: String,
    pub path: String,
    pub sandbox_runnable: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NotificationCenterScreen {
    pub unread_count: u64,
    pub notifications: Vec<NotificationEventRow>,
    pub preferences: Vec<NotificationPreferenceRow>,
    pub provider_decisions: Vec<NotificationProviderDecisionRow>,
    pub coverage_rules: Vec<NotificationCoverageRuleRow>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NotificationEventRow {
    pub id: u64,
    pub event_key: String,
    pub category: String,
    pub priority: String,
    pub subject: String,
    pub body: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<u64>,
    pub action_href: Option<String>,
    pub channels: Vec<String>,
    pub delivery_status: String,
    pub read_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NotificationPreferenceRow {
    pub id: u64,
    pub event_key: String,
    pub email_enabled: bool,
    pub in_app_enabled: bool,
    pub sms_enabled: bool,
    pub push_enabled: bool,
    pub quiet_hours_start: Option<String>,
    pub quiet_hours_end: Option<String>,
    pub timezone: String,
    pub escalation_minutes: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NotificationProviderDecisionRow {
    pub channel: String,
    pub provider_name: String,
    pub decision_status: String,
    pub opt_in_required: bool,
    pub opt_out_required: bool,
    pub quiet_hours_required: bool,
    pub emergency_exception_allowed: bool,
    pub provider_audit_logs_required: bool,
    pub compliance_notes: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NotificationCoverageRuleRow {
    pub event_key: String,
    pub category: String,
    pub default_priority: String,
    pub default_channels: Vec<String>,
    pub responsible_party: String,
    pub entity_type: Option<String>,
    pub escalation_minutes: Option<i32>,
    pub active: bool,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpsertNotificationPreferenceRequest {
    pub event_key: String,
    pub email_enabled: bool,
    pub in_app_enabled: bool,
    pub sms_enabled: bool,
    pub push_enabled: bool,
    pub quiet_hours_start: Option<String>,
    pub quiet_hours_end: Option<String>,
    pub timezone: Option<String>,
    pub escalation_minutes: Option<i32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarkNotificationReadRequest {
    pub notification_id: Option<u64>,
    pub mark_all: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NotificationMutationResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CommunicationGovernanceScreen {
    pub sender_identities: Vec<SenderIdentityRow>,
    pub delivery_events: Vec<DeliveryEventRow>,
    pub suppression_entries: Vec<SuppressionEntryRow>,
    pub template_governance: Vec<MessageTemplateGovernanceRow>,
    pub monitoring_rules: Vec<MessageMonitoringRuleRow>,
    pub branding_policies: Vec<TenantBrandingPolicyRow>,
    pub brand_assets: Vec<TenantBrandAssetRow>,
    pub custom_domains: Vec<TenantCustomDomainRow>,
    pub branded_template_rules: Vec<TenantBrandedTemplateRuleRow>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SenderIdentityRow {
    pub id: u64,
    pub environment_key: String,
    pub sender_domain: String,
    pub from_email: String,
    pub from_name: String,
    pub spf_status: String,
    pub dkim_status: String,
    pub dmarc_status: String,
    pub identity_status: String,
    pub verified_at: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeliveryEventRow {
    pub id: u64,
    pub channel: String,
    pub event_type: String,
    pub provider_message_id: Option<String>,
    pub recipient: String,
    pub reason: Option<String>,
    pub occurred_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SuppressionEntryRow {
    pub id: u64,
    pub channel: String,
    pub recipient: String,
    pub suppression_reason: String,
    pub status: String,
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MessageTemplateGovernanceRow {
    pub id: u64,
    pub template_key: String,
    pub channel: String,
    pub locale: String,
    pub version: i32,
    pub owner_team: String,
    pub approval_status: String,
    pub high_risk: bool,
    pub test_send_required: bool,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MessageMonitoringRuleRow {
    pub rule_key: String,
    pub event_key: Option<String>,
    pub template_key: Option<String>,
    pub category: String,
    pub priority: String,
    pub required_sender_identity: bool,
    pub fallback_channel: String,
    pub escalation_minutes: i32,
    pub active: bool,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TenantBrandingPolicyRow {
    pub organization_id: u64,
    pub portal_branding_enabled: bool,
    pub document_branding_enabled: bool,
    pub email_branding_enabled: bool,
    pub custom_domain_enabled: bool,
    pub white_label_status: String,
    pub unsupported_message: String,
    pub fallback_brand_name: String,
    pub cache_version: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TenantBrandAssetRow {
    pub id: u64,
    pub asset_type: String,
    pub asset_url: String,
    pub mime_type: String,
    pub file_size_bytes: i64,
    pub review_status: String,
    pub cache_key: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TenantCustomDomainRow {
    pub id: u64,
    pub domain: String,
    pub purpose: String,
    pub verification_status: String,
    pub dns_txt_name: String,
    pub dns_txt_value: String,
    pub tls_status: String,
    pub rollback_status: String,
    pub last_checked_at: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TenantBrandedTemplateRuleRow {
    pub id: u64,
    pub template_key: String,
    pub template_surface: String,
    pub branding_status: String,
    pub fallback_allowed: bool,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MessageTemplateTestSendRequest {
    pub template_key: String,
    pub channel: String,
    pub recipient: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EscrowStatusDescriptorLite {
    pub label: String,
    pub legacy_label: String,
    pub description: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StripeWebhookEventDescriptorLite {
    pub legacy_label: String,
    pub updates: Vec<String>,
    pub notes: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FinanceApprovalQueueItem {
    pub approval_id: i64,
    pub approval_type: String,
    pub leg_id: i64,
    pub load_id: Option<i64>,
    pub amount_cents: i64,
    pub currency: String,
    pub status: String,
    pub required_approval_count: i32,
    pub approval_count: i32,
    pub requested_by_user_id: Option<i64>,
    pub first_approved_by_user_id: Option<i64>,
    pub second_approved_by_user_id: Option<i64>,
    pub reason: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FinanceApprovalQueueResponse {
    pub approvals: Vec<FinanceApprovalQueueItem>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FinanceApprovalActionRequest {
    pub note: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FinanceApprovalActionResponse {
    pub success: bool,
    pub leg_id: i64,
    pub approval_id: Option<i64>,
    pub status: String,
    pub required_approval_count: i32,
    pub approval_count: i32,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InvoiceSettlementQueueItem {
    pub invoice_id: i64,
    pub invoice_number: String,
    pub settlement_id: i64,
    pub settlement_number: String,
    pub load_id: i64,
    pub leg_id: i64,
    pub customer_user_id: Option<i64>,
    pub carrier_user_id: Option<i64>,
    pub currency: String,
    pub invoice_total_amount_cents: i64,
    pub invoice_adjustment_amount_cents: i64,
    pub invoice_status: String,
    pub settlement_gross_amount_cents: i64,
    pub settlement_platform_fee_cents: i64,
    pub settlement_adjustment_amount_cents: i64,
    pub settlement_net_amount_cents: i64,
    pub settlement_status: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InvoiceSettlementQueueResponse {
    pub rows: Vec<InvoiceSettlementQueueItem>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PaymentAdjustmentRequest {
    pub amount_cents: i64,
    pub direction: String,
    pub adjustment_reference: Option<String>,
    pub note: Option<String>,
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlatformBillingRow {
    pub billing_account_id: i64,
    pub organization_id: Option<i64>,
    pub customer_user_id: Option<i64>,
    pub plan_name: Option<String>,
    pub billing_status: String,
    pub payment_method_status: String,
    pub latest_invoice_id: Option<i64>,
    pub latest_invoice_number: Option<String>,
    pub latest_invoice_status: Option<String>,
    pub open_invoice_cents: i64,
    pub past_due_invoice_cents: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlatformBillingScreenResponse {
    pub rows: Vec<PlatformBillingRow>,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FinanceMutationResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ShipperCreditRow {
    pub credit_account_id: i64,
    pub organization_id: Option<i64>,
    pub customer_user_id: Option<i64>,
    pub credit_status: String,
    pub credit_limit_cents: i64,
    pub open_ar_cents: i64,
    pub overdue_ar_cents: i64,
    pub payment_terms_days: i32,
    pub credit_hold: bool,
    pub override_required: bool,
    pub internal_risk_note: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ShipperCreditScreenResponse {
    pub rows: Vec<ShipperCreditRow>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreditOverrideRequest {
    pub credit_account_id: i64,
    pub reason: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PayoutReviewRow {
    pub review_id: i64,
    pub carrier_user_id: i64,
    pub stripe_connect_account_id: Option<String>,
    pub change_type: String,
    pub risk_status: String,
    pub cooling_off_until: Option<String>,
    pub notification_sent_at: Option<String>,
    pub review_note: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PayoutReviewQueueResponse {
    pub rows: Vec<PayoutReviewRow>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PayoutReviewDecisionRequest {
    pub decision: String,
    pub note: Option<String>,
}

#[cfg(target_arch = "wasm32")]
const AUTH_TOKEN_STORAGE_KEY: &str = "stloads_rust_auth_token";

pub async fn fetch_auth_session() -> Result<AuthSessionState, String> {
    get_api("/auth/session").await
}

pub async fn login(payload: &LoginRequest) -> Result<LoginResponse, String> {
    let response: LoginResponse = post_api("/auth/login", payload).await?;
    if response.success
        && let Some(token) = response.token.as_deref()
    {
        store_auth_token(token);
    }
    Ok(response)
}

pub async fn logout() -> Result<LogoutResponse, String> {
    let response = post_api::<LogoutResponse, _>("/auth/logout", &serde_json::json!({})).await?;
    if response.success {
        clear_auth_token();
    }
    Ok(response)
}

pub async fn verify_mfa(payload: &MfaVerifyRequest) -> Result<MfaVerifyResponse, String> {
    let response: MfaVerifyResponse = post_api("/auth/mfa/verify", payload).await?;
    if response.success
        && let Some(token) = response.token.as_deref()
    {
        store_auth_token(token);
    }
    Ok(response)
}

pub async fn register(payload: &RegisterRequest) -> Result<RegisterResponse, String> {
    post_api("/auth/register", payload).await
}

pub async fn fetch_portal_role_counts() -> Result<PortalRoleCountsResponse, String> {
    get_api("/auth/public-role-counts").await
}

pub async fn verify_otp(payload: &VerifyOtpRequest) -> Result<VerifyOtpResponse, String> {
    let response: VerifyOtpResponse = post_api("/auth/verify-otp", payload).await?;
    if response.success
        && let Some(token) = response.token.as_deref()
    {
        store_auth_token(token);
    }
    Ok(response)
}

pub async fn resend_otp(payload: &ResendOtpRequest) -> Result<ResendOtpResponse, String> {
    post_api("/auth/otp/resend", payload).await
}

pub async fn forgot_password(
    payload: &ForgotPasswordRequest,
) -> Result<ForgotPasswordResponse, String> {
    post_api("/auth/forgot-password", payload).await
}

pub async fn reset_password(
    payload: &ResetPasswordRequest,
) -> Result<ResetPasswordResponse, String> {
    post_api("/auth/reset-password", payload).await
}

pub async fn fetch_onboarding_screen() -> Result<AuthOnboardingScreen, String> {
    get_api("/auth/onboarding-screen").await
}

pub async fn fetch_legal_agreement_screen() -> Result<LegalAgreementScreen, String> {
    get_api("/auth/legal-agreements").await
}

pub async fn accept_legal_agreement(
    payload: &AcceptLegalAgreementRequest,
) -> Result<AcceptLegalAgreementResponse, String> {
    post_api("/auth/legal-agreements/accept", payload).await
}

pub async fn submit_onboarding(
    payload: &SubmitOnboardingRequest,
) -> Result<SubmitOnboardingResponse, String> {
    post_api("/auth/onboarding", payload).await
}

pub async fn fetch_self_profile_screen() -> Result<SelfProfileScreen, String> {
    get_api("/auth/profile-screen").await
}

pub async fn update_self_profile(
    payload: &UpdateSelfProfileRequest,
) -> Result<UpdateSelfProfileResponse, String> {
    post_api("/auth/profile", payload).await
}

pub async fn update_carrier_capacity(
    payload: &UpdateCarrierCapacityRequest,
) -> Result<UpdateCarrierCapacityResponse, String> {
    post_api("/auth/carrier-capacity", payload).await
}

pub async fn change_password(
    payload: &ChangePasswordRequest,
) -> Result<ChangePasswordResponse, String> {
    post_api("/auth/change-password", payload).await
}

pub async fn fetch_notification_center() -> Result<NotificationCenterScreen, String> {
    get_api("/auth/notifications").await
}

pub async fn mark_notification_read(
    payload: &MarkNotificationReadRequest,
) -> Result<NotificationMutationResponse, String> {
    post_api("/auth/notifications/read", payload).await
}

pub async fn upsert_notification_preference(
    payload: &UpsertNotificationPreferenceRequest,
) -> Result<NotificationMutationResponse, String> {
    post_api("/auth/notification-preferences", payload).await
}

pub async fn fetch_communication_governance() -> Result<CommunicationGovernanceScreen, String> {
    get_api("/auth/communication-governance").await
}

pub async fn record_message_template_test_send(
    payload: &MessageTemplateTestSendRequest,
) -> Result<NotificationMutationResponse, String> {
    post_api("/auth/message-templates/test-send", payload).await
}

pub async fn update_profile_kyc_document(
    document_id: u64,
    payload: &UpsertKycDocumentRequest,
) -> Result<UpsertKycDocumentResponse, String> {
    let path = format!("/auth/profile/documents/{}", document_id);
    post_api(&path, payload).await
}

pub async fn verify_profile_kyc_document(
    document_id: u64,
    payload: &VerifyKycDocumentRequest,
) -> Result<VerifyKycDocumentResponse, String> {
    let path = format!("/auth/profile/documents/{}/verify-blockchain", document_id);
    post_api(&path, payload).await
}

pub async fn delete_profile_kyc_document(
    document_id: u64,
) -> Result<DeleteKycDocumentResponse, String> {
    let path = format!("/auth/profile/documents/{}/delete", document_id);
    post_api(&path, &serde_json::json!({})).await
}

pub async fn fetch_admin_overview() -> Result<AdminOverview, String> {
    get_api("/admin").await
}

pub async fn fetch_master_data_screen() -> Result<MasterDataScreen, String> {
    get_api("/master-data/screen").await
}

pub async fn export_governed_master_data() -> Result<MasterDataExportResponse, String> {
    get_api("/master-data/governed/export").await
}

pub async fn import_governed_master_data(
    payload: &MasterDataImportRequest,
) -> Result<MasterDataMutationResponse, String> {
    post_api("/master-data/governed/import", payload).await
}

pub async fn rollback_governed_change(
    payload: &MasterDataRollbackRequest,
) -> Result<MasterDataMutationResponse, String> {
    post_api("/master-data/governed/rollback", payload).await
}

pub async fn upsert_country(
    payload: &CountryUpsertRequest,
) -> Result<MasterDataMutationResponse, String> {
    post_api("/master-data/countries", payload).await
}

pub async fn upsert_city(
    payload: &CityUpsertRequest,
) -> Result<MasterDataMutationResponse, String> {
    post_api("/master-data/cities", payload).await
}

pub async fn upsert_load_type(
    payload: &SimpleCatalogUpsertRequest,
) -> Result<MasterDataMutationResponse, String> {
    post_api("/master-data/load-types", payload).await
}

pub async fn upsert_equipment(
    payload: &SimpleCatalogUpsertRequest,
) -> Result<MasterDataMutationResponse, String> {
    post_api("/master-data/equipments", payload).await
}

pub async fn upsert_commodity_type(
    payload: &SimpleCatalogUpsertRequest,
) -> Result<MasterDataMutationResponse, String> {
    post_api("/master-data/commodity-types", payload).await
}

pub async fn upsert_location(
    payload: &LocationUpsertRequest,
) -> Result<MasterDataMutationResponse, String> {
    post_api("/master-data/locations", payload).await
}

pub async fn upsert_service_level(
    payload: &GovernedCatalogUpsertRequest,
) -> Result<MasterDataMutationResponse, String> {
    post_api("/master-data/service-levels", payload).await
}

pub async fn upsert_rejection_reason(
    payload: &GovernedCatalogUpsertRequest,
) -> Result<MasterDataMutationResponse, String> {
    post_api("/master-data/rejection-reasons", payload).await
}

pub async fn upsert_exception_reason(
    payload: &GovernedCatalogUpsertRequest,
) -> Result<MasterDataMutationResponse, String> {
    post_api("/master-data/exception-reasons", payload).await
}

pub async fn upsert_trailer_type(
    payload: &GovernedCatalogUpsertRequest,
) -> Result<MasterDataMutationResponse, String> {
    post_api("/master-data/trailer-types", payload).await
}

pub async fn upsert_hazmat_class(
    payload: &GovernedCatalogUpsertRequest,
) -> Result<MasterDataMutationResponse, String> {
    post_api("/master-data/hazmat-classes", payload).await
}

pub async fn upsert_accessorial(
    payload: &GovernedCatalogUpsertRequest,
) -> Result<MasterDataMutationResponse, String> {
    post_api("/master-data/accessorials", payload).await
}

pub async fn upsert_document_requirement(
    payload: &DocumentRequirementRuleUpsertRequest,
) -> Result<MasterDataMutationResponse, String> {
    post_api("/master-data/document-requirements", payload).await
}

pub async fn upsert_customer_configuration(
    payload: &CustomerConfigurationRuleUpsertRequest,
) -> Result<MasterDataMutationResponse, String> {
    post_api("/master-data/customer-configurations", payload).await
}

pub async fn delete_country(
    payload: &MasterDataDeleteRequest,
) -> Result<MasterDataMutationResponse, String> {
    post_api("/master-data/countries/delete", payload).await
}

pub async fn delete_city(
    payload: &MasterDataDeleteRequest,
) -> Result<MasterDataMutationResponse, String> {
    post_api("/master-data/cities/delete", payload).await
}

pub async fn delete_load_type(
    payload: &MasterDataDeleteRequest,
) -> Result<MasterDataMutationResponse, String> {
    post_api("/master-data/load-types/delete", payload).await
}

pub async fn delete_equipment(
    payload: &MasterDataDeleteRequest,
) -> Result<MasterDataMutationResponse, String> {
    post_api("/master-data/equipments/delete", payload).await
}

pub async fn delete_commodity_type(
    payload: &MasterDataDeleteRequest,
) -> Result<MasterDataMutationResponse, String> {
    post_api("/master-data/commodity-types/delete", payload).await
}

pub async fn delete_location(
    payload: &MasterDataDeleteRequest,
) -> Result<MasterDataMutationResponse, String> {
    post_api("/master-data/locations/delete", payload).await
}

pub async fn delete_service_level(
    payload: &MasterDataDeleteRequest,
) -> Result<MasterDataMutationResponse, String> {
    post_api("/master-data/service-levels/delete", payload).await
}

pub async fn delete_rejection_reason(
    payload: &MasterDataDeleteRequest,
) -> Result<MasterDataMutationResponse, String> {
    post_api("/master-data/rejection-reasons/delete", payload).await
}

pub async fn delete_exception_reason(
    payload: &MasterDataDeleteRequest,
) -> Result<MasterDataMutationResponse, String> {
    post_api("/master-data/exception-reasons/delete", payload).await
}

pub async fn delete_trailer_type(
    payload: &MasterDataDeleteRequest,
) -> Result<MasterDataMutationResponse, String> {
    post_api("/master-data/trailer-types/delete", payload).await
}

pub async fn delete_hazmat_class(
    payload: &MasterDataDeleteRequest,
) -> Result<MasterDataMutationResponse, String> {
    post_api("/master-data/hazmat-classes/delete", payload).await
}

pub async fn delete_accessorial(
    payload: &MasterDataDeleteRequest,
) -> Result<MasterDataMutationResponse, String> {
    post_api("/master-data/accessorials/delete", payload).await
}

pub async fn delete_document_requirement(
    payload: &MasterDataDeleteRequest,
) -> Result<MasterDataMutationResponse, String> {
    post_api("/master-data/document-requirements/delete", payload).await
}

pub async fn delete_customer_configuration(
    payload: &MasterDataDeleteRequest,
) -> Result<MasterDataMutationResponse, String> {
    post_api("/master-data/customer-configurations/delete", payload).await
}

pub async fn fetch_load_builder_screen(load_id: Option<u64>) -> Result<LoadBuilderScreen, String> {
    let path = match load_id {
        Some(load_id) => format!("/dispatch/loads/{}/builder", load_id),
        None => "/dispatch/load-builder".to_string(),
    };
    get_api(&path).await
}

pub async fn create_load(payload: &CreateLoadRequest) -> Result<CreateLoadResponse, String> {
    post_api("/dispatch/loads", payload).await
}

pub async fn update_load(
    load_id: u64,
    payload: &CreateLoadRequest,
) -> Result<CreateLoadResponse, String> {
    let path = format!("/dispatch/loads/{}/update", load_id);
    post_api(&path, payload).await
}

pub async fn preview_bulk_load_import(
    payload: &BulkLoadImportPreviewRequest,
) -> Result<BulkLoadImportResponse, String> {
    post_api("/dispatch/loads/import/preview", payload).await
}

pub async fn commit_bulk_load_import(
    payload: &BulkLoadImportCommitRequest,
) -> Result<BulkLoadImportResponse, String> {
    post_api("/dispatch/loads/import/commit", payload).await
}

pub async fn fetch_load_board_screen(
    tab: &str,
    filters: &LoadBoardFilters,
) -> Result<LoadBoardScreen, String> {
    let mut params = vec![
        format!("tab={}", tab),
        format!("page={}", filters.page.max(1)),
        format!("per_page={}", filters.per_page.clamp(1, 100)),
    ];
    let mut push_param = |key: &str, value: Option<String>| {
        if let Some(value) = value
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
        {
            params.push(format!("{}={}", key, urlencoding::encode(&value)));
        }
    };
    push_param("origin", filters.origin.clone());
    push_param("destination", filters.destination.clone());
    push_param("pickup_date", filters.pickup_date.clone());
    push_param("delivery_date", filters.delivery_date.clone());
    push_param("customer", filters.customer.clone());
    push_param("status", filters.status.clone());
    push_param("compliance", filters.compliance.clone());
    push_param("visibility", filters.visibility.clone());
    if let Some(value) = filters.radius_miles {
        params.push(format!("radius_miles={}", value));
    }
    if let Some(value) = filters.equipment_id {
        params.push(format!("equipment_id={}", value));
    }
    if let Some(value) = filters.commodity_type_id {
        params.push(format!("commodity_type_id={}", value));
    }
    if let Some(value) = filters.min_rate {
        params.push(format!("min_rate={}", value));
    }
    if let Some(value) = filters.max_rate {
        params.push(format!("max_rate={}", value));
    }
    let path = format!("/dispatch/load-board?{}", params.join("&"));
    get_api(&path).await
}

pub async fn fetch_dispatch_desk_screen(desk_key: &str) -> Result<DispatchDeskScreen, String> {
    let path = format!("/dispatch/desk/{}", desk_key);
    get_api(&path).await
}

pub async fn fetch_carrier_network_screen() -> Result<shared::CarrierNetworkScreen, String> {
    get_api("/dispatch/carrier-network").await
}

pub async fn fetch_carrier_matches(leg_id: u64) -> Result<CarrierMatchScreen, String> {
    let path = format!("/dispatch/load-board/{}/carrier-matches", leg_id);
    get_api(&path).await
}

pub async fn upsert_carrier_network(
    payload: &UpsertCarrierNetworkRequest,
) -> Result<UpsertCarrierNetworkResponse, String> {
    post_api("/dispatch/carrier-network", payload).await
}

pub async fn add_dispatch_desk_follow_up(
    leg_id: u64,
    payload: &DispatchDeskFollowUpRequest,
) -> Result<DispatchDeskFollowUpResponse, String> {
    let path = format!("/dispatch/desk/legs/{}/follow-up", leg_id);
    post_api(&path, payload).await
}

pub async fn resolve_dispatch_exception(
    leg_id: u64,
    payload: &ResolveDispatchExceptionRequest,
) -> Result<ResolveDispatchExceptionResponse, String> {
    let path = format!("/dispatch/desk/legs/{}/exceptions/resolve", leg_id);
    post_api(&path, payload).await
}

pub async fn fetch_load_profile_screen(load_id: u64) -> Result<LoadProfileScreen, String> {
    let path = format!("/dispatch/loads/{}", load_id);
    get_api(&path).await
}

pub async fn run_load_lifecycle_action(
    load_id: u64,
    payload: &LoadLifecycleActionRequest,
) -> Result<LoadLifecycleActionResponse, String> {
    let path = format!("/dispatch/loads/{}/lifecycle", load_id);
    post_api(&path, payload).await
}

pub async fn calculate_load_rate(
    load_id: u64,
    payload: &RateCalculationRequest,
) -> Result<RateCalculationResponse, String> {
    let path = format!("/dispatch/loads/{}/rating/calculate", load_id);
    post_api(&path, payload).await
}

pub async fn schedule_facility_appointment(
    load_id: u64,
    payload: &FacilityAppointmentRequest,
) -> Result<FacilityAppointmentResponse, String> {
    let path = format!("/dispatch/loads/{}/appointments", load_id);
    post_api(&path, payload).await
}

pub async fn create_load_document(
    load_id: u64,
    payload: &UpsertLoadDocumentRequest,
) -> Result<UpsertLoadDocumentResponse, String> {
    let path = format!("/dispatch/loads/{}/documents", load_id);
    post_api(&path, payload).await
}

pub async fn update_load_document(
    document_id: u64,
    payload: &UpsertLoadDocumentRequest,
) -> Result<UpsertLoadDocumentResponse, String> {
    let path = format!("/dispatch/documents/{}", document_id);
    post_api(&path, payload).await
}

pub async fn verify_load_document(
    document_id: u64,
    payload: &VerifyLoadDocumentRequest,
) -> Result<VerifyLoadDocumentResponse, String> {
    let path = format!("/dispatch/documents/{}/verify-blockchain", document_id);
    post_api(&path, payload).await
}

pub async fn generate_standard_freight_documents(
    load_id: u64,
    payload: &GenerateFreightDocumentsRequest,
) -> Result<GenerateFreightDocumentsResponse, String> {
    let path = format!("/dispatch/loads/{}/documents/generate-standard", load_id);
    post_api(&path, payload).await
}

pub async fn book_load_leg(
    leg_id: u64,
    payload: &BookLoadLegRequest,
) -> Result<BookLoadLegResponse, String> {
    let path = format!("/dispatch/load-board/{}/book", leg_id);
    post_api(&path, payload).await
}

pub async fn fetch_chat_workspace_screen(
    conversation_id: Option<u64>,
) -> Result<ChatWorkspaceScreen, String> {
    let path = match conversation_id {
        Some(id) => format!("/marketplace/chat-workspace?conversation_id={}", id),
        None => "/marketplace/chat-workspace".to_string(),
    };
    get_api(&path).await
}

pub async fn review_offer(
    offer_id: u64,
    payload: &OfferReviewRequest,
) -> Result<OfferReviewResponse, String> {
    let path = format!("/marketplace/offers/{}/review", offer_id);
    post_api(&path, payload).await
}

pub async fn review_tender(
    offer_id: u64,
    payload: &OfferReviewRequest,
) -> Result<OfferReviewResponse, String> {
    let path = format!("/marketplace/tenders/{}/decision", offer_id);
    post_api(&path, payload).await
}

pub async fn counter_offer(
    offer_id: u64,
    payload: &OfferCounterRequest,
) -> Result<OfferCounterResponse, String> {
    let path = format!("/marketplace/offers/{}/counter", offer_id);
    post_api(&path, payload).await
}

pub async fn generate_rate_confirmation(offer_id: u64) -> Result<RateConfirmationResponse, String> {
    let path = format!("/marketplace/offers/{}/rate-confirmation", offer_id);
    post_api(&path, &serde_json::json!({})).await
}

pub async fn send_message(
    conversation_id: u64,
    payload: &ChatSendMessageRequest,
) -> Result<ChatSendMessageResponse, String> {
    let path = format!("/marketplace/conversations/{}/messages", conversation_id);
    post_api(&path, payload).await
}

pub async fn mark_conversation_read(
    conversation_id: u64,
) -> Result<ConversationReadResponse, String> {
    let path = format!("/marketplace/conversations/{}/read", conversation_id);
    post_api(&path, &serde_json::json!({})).await
}

pub async fn fetch_stloads_operations_screen(
    status_filter: Option<&str>,
) -> Result<StloadsOperationsScreen, String> {
    let path = match status_filter {
        Some(filter) if !filter.trim().is_empty() => {
            format!("/admin/stloads/operations?status={}", filter)
        }
        _ => "/admin/stloads/operations".to_string(),
    };
    get_api(&path).await
}

pub async fn fetch_stloads_reconciliation_screen(
    action_filter: Option<&str>,
) -> Result<StloadsReconciliationScreen, String> {
    let path = match action_filter {
        Some(filter) if !filter.trim().is_empty() => {
            format!("/admin/stloads/reconciliation?action={}", filter)
        }
        _ => "/admin/stloads/reconciliation".to_string(),
    };
    get_api(&path).await
}

pub async fn fetch_integration_portal() -> Result<IntegrationPortalScreen, String> {
    get_api("/admin/integrations/portal").await
}

pub async fn create_partner_api_key(
    payload: &CreatePartnerApiKeyRequest,
) -> Result<CreatePartnerApiKeyResponse, String> {
    post_api("/admin/integrations/api-keys", payload).await
}

pub async fn upsert_webhook_endpoint(
    payload: &UpsertWebhookEndpointRequest,
) -> Result<IntegrationMutationResponse, String> {
    post_api("/admin/integrations/webhook-endpoints", payload).await
}

pub async fn fetch_edi_integration_screen() -> Result<EdiIntegrationScreen, String> {
    get_api("/admin/integrations/edi").await
}

pub async fn upsert_edi_partner_profile(
    payload: &UpsertEdiPartnerProfileRequest,
) -> Result<IntegrationMutationResponse, String> {
    post_api("/admin/integrations/edi/partners", payload).await
}

pub async fn validate_edi_message(
    payload: &ValidateEdiMessageRequest,
) -> Result<EdiValidationResponse, String> {
    post_api("/admin/integrations/edi/messages/validate", payload).await
}

pub async fn replay_edi_message(message_id: u64) -> Result<EdiReplayResponse, String> {
    let path = format!("/admin/integrations/edi/messages/{}/replay", message_id);
    post_api(&path, &serde_json::json!({})).await
}

pub async fn fetch_sandbox_governance_screen() -> Result<SandboxGovernanceScreen, String> {
    get_api("/admin/integrations/sandbox").await
}

pub async fn queue_sandbox_reset(
    payload: &QueueSandboxResetRequest,
) -> Result<SandboxResetResponse, String> {
    post_api("/admin/integrations/sandbox/reset", payload).await
}

pub async fn fetch_api_lifecycle_screen() -> Result<ApiLifecycleScreen, String> {
    get_api("/admin/integrations/api-lifecycle").await
}

pub async fn resolve_sync_error(
    sync_error_id: u64,
    payload: &ResolveSyncErrorRequest,
) -> Result<ResolveSyncErrorResponse, String> {
    let path = format!("/admin/stloads/sync-errors/{}/resolve", sync_error_id);
    post_api(&path, payload).await
}

pub async fn fetch_admin_onboarding_reviews() -> Result<AdminOnboardingReviewScreen, String> {
    get_api("/admin/onboarding-reviews").await
}

pub async fn fetch_admin_user_directory() -> Result<AdminUserDirectoryScreen, String> {
    get_api("/admin/users").await
}

pub async fn fetch_admin_support_search(
    query: &str,
    target_organization_id: Option<u64>,
) -> Result<AdminSupportSearchScreen, String> {
    let encoded_query = urlencoding::encode(query);
    let path = match target_organization_id {
        Some(organization_id) => format!(
            "/admin/support/search?q={}&target_organization_id={}",
            encoded_query, organization_id
        ),
        None => format!("/admin/support/search?q={}", encoded_query),
    };
    get_api(&path).await
}

pub async fn fetch_admin_audit_search(
    filters: &AdminAuditSearchFilters,
) -> Result<AdminAuditSearchScreen, String> {
    get_api(&admin_audit_path("/admin/audit", filters)).await
}

pub async fn export_admin_audit_search(
    filters: &AdminAuditSearchFilters,
) -> Result<AdminAuditExportResponse, String> {
    get_api(&admin_audit_path("/admin/audit/export", filters)).await
}

fn admin_audit_path(base: &str, filters: &AdminAuditSearchFilters) -> String {
    let mut params = Vec::new();
    if let Some(value) = filters.target_organization_id {
        params.push(format!("target_organization_id={value}"));
    }
    if let Some(value) = filters.actor_user_id {
        params.push(format!("actor_user_id={value}"));
    }
    push_optional_query(&mut params, "q", filters.q.as_deref());
    push_optional_query(&mut params, "entity_type", filters.entity_type.as_deref());
    push_optional_query(&mut params, "entity_id", filters.entity_id.as_deref());
    push_optional_query(&mut params, "action", filters.action.as_deref());
    push_optional_query(&mut params, "request_id", filters.request_id.as_deref());
    push_optional_query(&mut params, "date_from", filters.date_from.as_deref());
    push_optional_query(&mut params, "date_to", filters.date_to.as_deref());
    if params.is_empty() {
        base.into()
    } else {
        format!("{}?{}", base, params.join("&"))
    }
}

fn push_optional_query(params: &mut Vec<String>, key: &str, value: Option<&str>) {
    if let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) {
        params.push(format!("{}={}", key, urlencoding::encode(value)));
    }
}

pub async fn fetch_admin_support_timeline(
    entity_type: &str,
    entity_id: Option<&str>,
    target_organization_id: Option<u64>,
) -> Result<AdminSupportTimelineScreen, String> {
    let mut path = format!(
        "/admin/support/timeline?entity_type={}",
        urlencoding::encode(entity_type)
    );
    if let Some(entity_id) = entity_id {
        path.push_str(&format!("&entity_id={}", urlencoding::encode(entity_id)));
    }
    if let Some(organization_id) = target_organization_id {
        path.push_str(&format!("&target_organization_id={}", organization_id));
    }
    get_api(&path).await
}

pub async fn create_admin_support_note(
    payload: &AdminCreateSupportNoteRequest,
) -> Result<AdminCreateSupportNoteResponse, String> {
    post_api("/admin/support/notes", payload).await
}

pub async fn fetch_admin_support_cases(
    target_organization_id: Option<u64>,
) -> Result<AdminSupportCaseScreen, String> {
    let path = match target_organization_id {
        Some(organization_id) => {
            format!("/admin/support/cases?target_organization_id={organization_id}")
        }
        None => "/admin/support/cases".into(),
    };
    get_api(&path).await
}

pub async fn create_admin_support_case(
    payload: &AdminCreateSupportCaseRequest,
) -> Result<AdminSupportCaseMutationResponse, String> {
    post_api("/admin/support/cases", payload).await
}

pub async fn update_admin_support_case(
    case_id: u64,
    payload: &AdminUpdateSupportCaseRequest,
) -> Result<AdminSupportCaseMutationResponse, String> {
    post_api(&format!("/admin/support/cases/{case_id}"), payload).await
}

pub async fn record_admin_support_case_feedback(
    case_id: u64,
    payload: &AdminSupportCaseFeedbackRequest,
) -> Result<AdminSupportCaseMutationResponse, String> {
    post_api(&format!("/admin/support/cases/{case_id}/feedback"), payload).await
}

pub async fn fetch_admin_identity_screen(
    target_organization_id: Option<u64>,
) -> Result<AdminIdentityScreen, String> {
    let path = match target_organization_id {
        Some(organization_id) => {
            format!("/admin/identity?target_organization_id={}", organization_id)
        }
        None => "/admin/identity".into(),
    };
    get_api(&path).await
}

pub async fn upsert_admin_identity_domain(
    payload: &AdminUpsertIdentityDomainRequest,
) -> Result<AdminIdentityMutationResponse, String> {
    post_api("/admin/identity/domains", payload).await
}

pub async fn verify_admin_identity_domain(
    payload: &AdminVerifyIdentityDomainRequest,
) -> Result<AdminIdentityMutationResponse, String> {
    post_api("/admin/identity/domains/verify", payload).await
}

pub async fn check_admin_identity_domain_dns(
    payload: &AdminCheckIdentityDomainDnsRequest,
) -> Result<AdminIdentityMutationResponse, String> {
    post_api("/admin/identity/domains/verify-dns", payload).await
}

pub async fn upsert_admin_identity_provider(
    payload: &AdminUpsertIdentityProviderRequest,
) -> Result<AdminIdentityMutationResponse, String> {
    post_api("/admin/identity/providers", payload).await
}

pub async fn fetch_admin_access_reviews(
    target_organization_id: Option<u64>,
) -> Result<AdminAccessReviewScreen, String> {
    let path = match target_organization_id {
        Some(organization_id) => {
            format!(
                "/admin/access-reviews?target_organization_id={}",
                organization_id
            )
        }
        None => "/admin/access-reviews".into(),
    };
    get_api(&path).await
}

pub async fn start_admin_access_review(
    payload: &AdminStartAccessReviewRequest,
) -> Result<AdminAccessReviewMutationResponse, String> {
    post_api("/admin/access-reviews/start", payload).await
}

pub async fn decide_admin_access_review_item(
    item_id: u64,
    payload: &AdminAccessReviewDecisionRequest,
) -> Result<AdminAccessReviewMutationResponse, String> {
    let path = format!("/admin/access-reviews/items/{}/decision", item_id);
    post_api(&path, payload).await
}

pub async fn create_admin_access_elevation_request(
    payload: &AdminCreateAccessElevationRequest,
) -> Result<AdminAccessReviewMutationResponse, String> {
    post_api("/admin/access-reviews/elevation-requests", payload).await
}

pub async fn decide_admin_access_elevation_request(
    request_id: u64,
    payload: &AdminAccessElevationDecisionRequest,
) -> Result<AdminAccessReviewMutationResponse, String> {
    let path = format!(
        "/admin/access-reviews/elevation-requests/{}/decision",
        request_id
    );
    post_api(&path, payload).await
}

pub async fn fetch_admin_load_list_screen(tab: &str) -> Result<AdminLoadListScreen, String> {
    let path = format!("/admin/loads?tab={}", tab);
    get_api(&path).await
}

pub async fn review_admin_load(
    load_id: u64,
    payload: &AdminReviewLoadRequest,
) -> Result<AdminReviewLoadResponse, String> {
    let path = format!("/admin/loads/{}/review", load_id);
    post_api(&path, payload).await
}

pub async fn fetch_admin_user_profile(user_id: u64) -> Result<AdminUserProfileScreen, String> {
    let path = format!("/admin/users/{}/profile", user_id);
    get_api(&path).await
}

pub async fn create_admin_user(
    payload: &AdminCreateUserRequest,
) -> Result<AdminCreateUserResponse, String> {
    post_api("/admin/users", payload).await
}

pub async fn fetch_admin_role_permissions() -> Result<AdminRolePermissionScreen, String> {
    get_api("/admin/roles/permissions").await
}

pub async fn review_onboarding_user(
    user_id: u64,
    payload: &ReviewOnboardingRequest,
) -> Result<ReviewOnboardingResponse, String> {
    let path = format!("/admin/users/{}/review", user_id);
    post_api(&path, payload).await
}

pub async fn update_admin_user_account(
    user_id: u64,
    payload: &AdminUpdateUserRequest,
) -> Result<AdminUpdateUserResponse, String> {
    let path = format!("/admin/users/{}/account", user_id);
    post_api(&path, payload).await
}

pub async fn update_admin_user_profile(
    user_id: u64,
    payload: &AdminUpdateUserProfileRequest,
) -> Result<AdminUpdateUserProfileResponse, String> {
    let path = format!("/admin/users/{}/profile", user_id);
    post_api(&path, payload).await
}

pub async fn delete_admin_user(user_id: u64) -> Result<AdminDeleteUserResponse, String> {
    let path = format!("/admin/users/{}/delete", user_id);
    post_api(&path, &serde_json::json!({})).await
}

pub async fn update_admin_role_permissions(
    role_key: &str,
    payload: &AdminUpdateRolePermissionsRequest,
) -> Result<AdminUpdateRolePermissionsResponse, String> {
    let path = format!("/admin/roles/{}/permissions", role_key);
    post_api(&path, payload).await
}
pub async fn fetch_payments_overview() -> Result<PaymentsOverview, String> {
    get_api("/payments").await
}

pub async fn fetch_escrow_status_catalog() -> Result<Vec<EscrowStatusDescriptorLite>, String> {
    get_api("/payments/escrow-statuses").await
}

pub async fn fetch_stripe_webhook_event_catalog()
-> Result<Vec<StripeWebhookEventDescriptorLite>, String> {
    get_api("/payments/webhook-events").await
}

pub async fn fund_escrow(
    leg_id: u64,
    payload: &EscrowFundRequest,
) -> Result<EscrowLifecycleResponse, String> {
    let path = format!("/payments/legs/{}/fund", leg_id);
    post_api(&path, payload).await
}

pub async fn hold_escrow(
    leg_id: u64,
    payload: &EscrowHoldRequest,
) -> Result<EscrowLifecycleResponse, String> {
    let path = format!("/payments/legs/{}/hold", leg_id);
    post_api(&path, payload).await
}

pub async fn release_escrow(
    leg_id: u64,
    payload: &EscrowReleaseRequest,
) -> Result<EscrowLifecycleResponse, String> {
    let path = format!("/payments/legs/{}/release", leg_id);
    post_api(&path, payload).await
}

pub async fn fetch_release_approval_queue() -> Result<FinanceApprovalQueueResponse, String> {
    get_api("/payments/finance-approvals").await
}

pub async fn approve_release_escrow(
    leg_id: u64,
    payload: &FinanceApprovalActionRequest,
) -> Result<FinanceApprovalActionResponse, String> {
    let path = format!("/payments/legs/{}/release-approval", leg_id);
    post_api(&path, payload).await
}

pub async fn approve_hold_escrow(
    leg_id: u64,
    payload: &FinanceApprovalActionRequest,
) -> Result<FinanceApprovalActionResponse, String> {
    let path = format!("/payments/legs/{}/hold-approval", leg_id);
    post_api(&path, payload).await
}

pub async fn fetch_invoice_settlement_queue() -> Result<InvoiceSettlementQueueResponse, String> {
    get_api("/payments/invoice-settlements").await
}

pub async fn record_payment_adjustment(
    leg_id: u64,
    payload: &PaymentAdjustmentRequest,
) -> Result<EscrowLifecycleResponse, String> {
    let path = format!("/payments/legs/{}/adjustment", leg_id);
    post_api(&path, payload).await
}

pub async fn fetch_platform_billing_accounts() -> Result<PlatformBillingScreenResponse, String> {
    get_api("/payments/platform-billing/accounts").await
}

pub async fn generate_platform_billing_invoices() -> Result<FinanceMutationResponse, String> {
    post_api(
        "/payments/platform-billing/generate-invoices",
        &serde_json::json!({}),
    )
    .await
}

pub async fn mark_platform_invoice_paid(
    invoice_id: i64,
) -> Result<FinanceMutationResponse, String> {
    let path = format!(
        "/payments/platform-billing/invoices/{}/mark-paid",
        invoice_id
    );
    post_api(&path, &serde_json::json!({})).await
}

pub async fn fetch_shipper_credit_accounts() -> Result<ShipperCreditScreenResponse, String> {
    get_api("/payments/shipper-credit/accounts").await
}

pub async fn approve_credit_override(
    payload: &CreditOverrideRequest,
) -> Result<FinanceMutationResponse, String> {
    post_api("/payments/shipper-credit/override", payload).await
}

pub async fn fetch_payout_change_reviews() -> Result<PayoutReviewQueueResponse, String> {
    get_api("/payments/payout-change-reviews").await
}

pub async fn decide_payout_change_review(
    review_id: i64,
    payload: &PayoutReviewDecisionRequest,
) -> Result<FinanceMutationResponse, String> {
    let path = format!("/payments/payout-change-reviews/{}/decision", review_id);
    post_api(&path, payload).await
}

pub async fn trigger_stripe_webhook(
    payload: &StripeWebhookRequest,
) -> Result<StripeWebhookResponse, String> {
    post_api("/payments/webhooks/stripe", payload).await
}

pub async fn push_tms_handoff(payload: &TmsHandoffPayload) -> Result<TmsHandoffResponse, String> {
    post_api("/tms/push", payload).await
}

pub async fn queue_tms_handoff(payload: &TmsHandoffPayload) -> Result<TmsHandoffResponse, String> {
    post_api("/tms/queue", payload).await
}

pub async fn requeue_tms_handoff(
    payload: &TmsRequeueRequest,
) -> Result<TmsHandoffResponse, String> {
    post_api("/tms/requeue", payload).await
}

pub async fn withdraw_tms_handoff(
    payload: &TmsWithdrawRequest,
) -> Result<TmsHandoffResponse, String> {
    post_api("/tms/withdraw", payload).await
}

pub async fn close_tms_handoff(payload: &TmsCloseRequest) -> Result<TmsHandoffResponse, String> {
    post_api("/tms/close", payload).await
}

pub async fn run_dispatch_desk_handoff_action(
    handoff_id: u64,
    action_key: &str,
) -> Result<TmsHandoffResponse, String> {
    match action_key {
        "requeue" => {
            requeue_tms_handoff(&TmsRequeueRequest {
                handoff_id: handoff_id as i64,
                pushed_by: Some("rust_dispatch_desk".into()),
                source_module: Some("dispatch_desk".into()),
            })
            .await
        }
        "withdraw" => {
            withdraw_tms_handoff(&TmsWithdrawRequest {
                handoff_id: handoff_id as i64,
                reason: Some("Rust dispatch desk action".into()),
                pushed_by: Some("rust_dispatch_desk".into()),
                source_module: Some("dispatch_desk".into()),
            })
            .await
        }
        "close" => {
            close_tms_handoff(&TmsCloseRequest {
                handoff_id: handoff_id as i64,
                reason: Some("Rust dispatch desk action".into()),
                pushed_by: Some("rust_dispatch_desk".into()),
                source_module: Some("dispatch_desk".into()),
            })
            .await
        }
        _ => Err(format!(
            "Unsupported dispatch desk action '{}'.",
            action_key
        )),
    }
}

pub async fn apply_tms_status_webhook(
    payload: &TmsStatusWebhookRequest,
) -> Result<TmsWebhookResponse, String> {
    post_api("/tms/webhook/status", payload).await
}

pub fn realtime_ws_url(conversation_id: Option<u64>, topics: &[RealtimeTopic]) -> Option<String> {
    let token = auth_token()?;
    let base = configured_api_base();
    if base.is_empty() {
        return None;
    }

    let websocket_base = if let Some(stripped) = base.strip_prefix("https://") {
        format!("wss://{}", stripped)
    } else if let Some(stripped) = base.strip_prefix("http://") {
        format!("ws://{}", stripped)
    } else {
        format!("ws://{}", base)
    };

    let mut url = format!(
        "{}/realtime/ws?token={}",
        websocket_base.trim_end_matches('/'),
        token
    );

    if let Some(conversation_id) = conversation_id {
        url.push_str(&format!("&conversation_id={}", conversation_id));
    }

    if !topics.is_empty() {
        let topic_query = topics
            .iter()
            .map(|topic| topic.as_key())
            .collect::<Vec<_>>()
            .join(",");
        url.push_str(&format!("&topics={}", topic_query));
    }

    Some(url)
}

pub fn auth_token() -> Option<String> {
    read_auth_token()
}

fn store_auth_token(token: &str) {
    write_auth_token(Some(token));
}

pub fn clear_auth_token() {
    write_auth_token(None);
}

async fn get_api<T>(path: &str) -> Result<T, String>
where
    T: DeserializeOwned,
{
    let url = api_url(path);
    fetch_get::<T>(&url).await
}

async fn post_api<T, B>(path: &str, body: &B) -> Result<T, String>
where
    T: DeserializeOwned,
    B: Serialize + ?Sized,
{
    let url = api_url(path);
    fetch_post::<T, B>(&url, body).await
}

pub fn api_href(path: &str) -> String {
    let normalized_path = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{}", path)
    };

    #[cfg(target_arch = "wasm32")]
    {
        if let Some(base) = runtime_config::backend_api_base_url() {
            return format!("{}{}", base, normalized_path);
        }

        if let Some(base) = option_env!("BACKEND_API_BASE_URL") {
            let trimmed = base.trim().trim_end_matches('/');
            if !trimmed.is_empty() {
                return format!("{}{}", trimmed, normalized_path);
            }
        }

        // Use a dedicated proxied API prefix in the browser so JSON requests do not
        // collide with same-path SPA routes such as /admin/users or /admin/onboarding-reviews.
        return format!("/api/stloads{}", normalized_path);
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let base = configured_api_base();
        if base.is_empty() {
            normalized_path
        } else {
            format!("{}{}", base, normalized_path)
        }
    }
}

fn api_url(path: &str) -> String {
    api_href(path)
}

#[cfg(target_arch = "wasm32")]
fn configured_api_base() -> String {
    if let Some(base) = runtime_config::backend_api_base_url() {
        return base;
    }

    if let Some(base) = option_env!("BACKEND_API_BASE_URL") {
        let trimmed = base.trim().trim_end_matches('/');
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    web_sys::window()
        .and_then(|window| window.location().origin().ok())
        .unwrap_or_default()
        .trim_end_matches('/')
        .to_string()
}

#[cfg(not(target_arch = "wasm32"))]
fn configured_api_base() -> String {
    std::env::var("BACKEND_API_BASE_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:3001".to_string())
        .trim_end_matches('/')
        .to_string()
}

#[cfg(target_arch = "wasm32")]
fn read_auth_token() -> Option<String> {
    web_sys::window()
        .and_then(|window| window.local_storage().ok())
        .flatten()
        .and_then(|storage| storage.get_item(AUTH_TOKEN_STORAGE_KEY).ok())
        .flatten()
}

#[cfg(not(target_arch = "wasm32"))]
fn read_auth_token() -> Option<String> {
    std::env::var("STLOADS_AUTH_TOKEN").ok()
}

#[cfg(target_arch = "wasm32")]
fn write_auth_token(token: Option<&str>) {
    if let Some(storage) = web_sys::window()
        .and_then(|window| window.local_storage().ok())
        .flatten()
    {
        match token {
            Some(token) => {
                let _ = storage.set_item(AUTH_TOKEN_STORAGE_KEY, token);
            }
            None => {
                let _ = storage.remove_item(AUTH_TOKEN_STORAGE_KEY);
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn write_auth_token(_token: Option<&str>) {}

#[cfg(target_arch = "wasm32")]
async fn fetch_get<T>(url: &str) -> Result<T, String>
where
    T: DeserializeOwned,
{
    let mut request = gloo_net::http::Request::get(url).header("Accept", "application/json");
    if let Some(token) = read_auth_token() {
        request = request.header("Authorization", &format!("Bearer {}", token));
    }

    let response = request
        .send()
        .await
        .map_err(|error| format!("GET {} failed: {}", url, error))?;

    if !response.ok() {
        let status = response.status();
        let detail = response.text().await.unwrap_or_default();
        return Err(format!("GET {} returned {} {}", url, status, detail));
    }

    let envelope = response
        .json::<ApiResponse<T>>()
        .await
        .map_err(|error| format!("Failed to decode GET {}: {}", url, error))?;

    Ok(envelope.data)
}

#[cfg(not(target_arch = "wasm32"))]
async fn fetch_get<T>(url: &str) -> Result<T, String>
where
    T: DeserializeOwned,
{
    let mut request = reqwest::Client::new()
        .get(url)
        .header(reqwest::header::ACCEPT, "application/json");
    if let Some(token) = read_auth_token() {
        request = request.bearer_auth(token);
    }

    let response = request
        .send()
        .await
        .map_err(|error| format!("GET {} failed: {}", url, error))?;

    if !response.status().is_success() {
        let status = response.status();
        let detail = response.text().await.unwrap_or_default();
        return Err(format!("GET {} returned {} {}", url, status, detail));
    }

    let envelope = response
        .json::<ApiResponse<T>>()
        .await
        .map_err(|error| format!("Failed to decode GET {}: {}", url, error))?;

    Ok(envelope.data)
}

#[cfg(target_arch = "wasm32")]
async fn fetch_post<T, B>(url: &str, body: &B) -> Result<T, String>
where
    T: DeserializeOwned,
    B: Serialize + ?Sized,
{
    let mut request = gloo_net::http::Request::post(url);
    if let Some(token) = read_auth_token() {
        request = request.header("Authorization", &format!("Bearer {}", token));
    }

    let response = request
        .json(body)
        .map_err(|error| format!("Failed to serialize POST {} payload: {}", url, error))?
        .send()
        .await
        .map_err(|error| format!("POST {} failed: {}", url, error))?;

    if !response.ok() {
        let status = response.status();
        let detail = response.text().await.unwrap_or_default();
        return Err(format!("POST {} returned {} {}", url, status, detail));
    }

    let envelope = response
        .json::<ApiResponse<T>>()
        .await
        .map_err(|error| format!("Failed to decode POST {}: {}", url, error))?;

    Ok(envelope.data)
}

#[cfg(not(target_arch = "wasm32"))]
async fn fetch_post<T, B>(url: &str, body: &B) -> Result<T, String>
where
    T: DeserializeOwned,
    B: Serialize + ?Sized,
{
    let mut request = reqwest::Client::new().post(url).json(body);
    if let Some(token) = read_auth_token() {
        request = request.bearer_auth(token);
    }

    let response = request
        .send()
        .await
        .map_err(|error| format!("POST {} failed: {}", url, error))?;

    if !response.status().is_success() {
        let status = response.status();
        let detail = response.text().await.unwrap_or_default();
        return Err(format!("POST {} returned {} {}", url, status, detail));
    }

    let envelope = response
        .json::<ApiResponse<T>>()
        .await
        .map_err(|error| format!("Failed to decode POST {}: {}", url, error))?;

    Ok(envelope.data)
}

pub async fn fetch_execution_leg_screen(leg_id: u64) -> Result<ExecutionLegScreen, String> {
    let path = format!("/execution/legs/{}", leg_id);
    get_api(&path).await
}

pub async fn fetch_customer_tracking_screen(
    share_token: &str,
) -> Result<shared::ExecutionCustomerTrackingScreen, String> {
    let path = format!("/execution/customer-tracking/{}", share_token);
    get_api(&path).await
}

pub async fn run_execution_leg_action(
    leg_id: u64,
    payload: &ExecutionLegActionRequest,
) -> Result<ExecutionLegActionResponse, String> {
    let path = format!("/execution/legs/{}/actions", leg_id);
    post_api(&path, payload).await
}

pub async fn capture_execution_tracking_consent(
    leg_id: u64,
    payload: &shared::ExecutionTrackingConsentRequest,
) -> Result<shared::ExecutionTrackingConsentResponse, String> {
    let path = format!("/execution/legs/{}/tracking-consent", leg_id);
    post_api(&path, payload).await
}

pub async fn replay_execution_offline_submission(
    leg_id: u64,
    payload: &shared::ExecutionOfflineSubmissionRequest,
) -> Result<shared::ExecutionOfflineSubmissionResponse, String> {
    let path = format!("/execution/legs/{}/offline-submissions", leg_id);
    post_api(&path, payload).await
}

pub async fn review_execution_closeout(
    leg_id: u64,
    payload: &shared::ExecutionCloseoutApprovalRequest,
) -> Result<shared::ExecutionWorkflowMutationResponse, String> {
    let path = format!("/execution/legs/{}/closeout", leg_id);
    post_api(&path, payload).await
}

pub async fn create_execution_finance_exception(
    leg_id: u64,
    payload: &shared::ExecutionFinanceExceptionRequest,
) -> Result<shared::ExecutionWorkflowMutationResponse, String> {
    let path = format!("/execution/legs/{}/finance-exceptions", leg_id);
    post_api(&path, payload).await
}

pub async fn decide_execution_finance_exception(
    leg_id: u64,
    payload: &shared::ExecutionFinanceExceptionDecisionRequest,
) -> Result<shared::ExecutionWorkflowMutationResponse, String> {
    let path = format!("/execution/legs/{}/finance-exceptions/decision", leg_id);
    post_api(&path, payload).await
}

pub async fn create_customer_tracking_link(
    leg_id: u64,
    payload: &shared::ExecutionCustomerTrackingLinkRequest,
) -> Result<shared::ExecutionCustomerTrackingLinkResponse, String> {
    let path = format!("/execution/legs/{}/customer-tracking", leg_id);
    post_api(&path, payload).await
}

pub async fn revoke_customer_tracking_links(
    leg_id: u64,
    payload: &shared::ExecutionCustomerTrackingRevokeRequest,
) -> Result<shared::ExecutionCustomerTrackingLinkResponse, String> {
    let path = format!("/execution/legs/{}/customer-tracking/revoke", leg_id);
    post_api(&path, payload).await
}

pub async fn upsert_execution_telematics(
    leg_id: u64,
    payload: &shared::ExecutionTelematicsConnectionRequest,
) -> Result<shared::ExecutionWorkflowMutationResponse, String> {
    let path = format!("/execution/legs/{}/telematics", leg_id);
    post_api(&path, payload).await
}

pub async fn upsert_execution_route_plan(
    leg_id: u64,
    payload: &shared::ExecutionRoutePlanRequest,
) -> Result<shared::ExecutionWorkflowMutationResponse, String> {
    let path = format!("/execution/legs/{}/route-plan", leg_id);
    post_api(&path, payload).await
}

pub async fn send_execution_location_ping(
    leg_id: u64,
    payload: &ExecutionLocationPingRequest,
) -> Result<ExecutionLocationPingResponse, String> {
    let path = format!("/execution/legs/{}/location", leg_id);
    post_api(&path, payload).await
}
