pub mod actions;
pub mod auth_state;
pub mod execution;
pub mod master_data;
pub mod realtime;
pub mod screens;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequiredDocumentChecklistItem {
    pub key: String,
    pub label: String,
    pub requirement_scope: String,
    pub lifecycle_state: String,
    pub is_required: bool,
    pub is_satisfied: bool,
    pub status_label: String,
    pub status_tone: String,
    pub blocking_message: Option<String>,
}

pub use actions::{
    AdminReviewLoadRequest, AdminReviewLoadResponse, ApiPostLoadRequest, ApiPostLoadResponse,
    BookLoadLegRequest, BookLoadLegResponse, BulkLoadImportCommitRequest,
    BulkLoadImportPreviewRequest, BulkLoadImportResponse, BulkLoadImportRowResult,
    ChatSendMessageRequest, ChatSendMessageResponse, ConversationReadResponse,
    CreateLoadLegRequest, CreateLoadRequest, CreateLoadResponse, DispatchDeskFollowUpRequest,
    DispatchDeskFollowUpResponse, EscrowFundRequest, EscrowHoldRequest, EscrowLifecycleResponse,
    EscrowReleaseRequest, FacilityAppointmentRequest, FacilityAppointmentResponse,
    GenerateFreightDocumentsRequest, GenerateFreightDocumentsResponse,
    GeneratedFreightDocumentItem, LoadLifecycleActionRequest, LoadLifecycleActionResponse,
    OfferCounterRequest, OfferCounterResponse, OfferReviewDecision, OfferReviewRequest,
    OfferReviewResponse, RateAccessorialLine, RateCalculationRequest, RateCalculationResponse,
    RateConfirmationResponse, ResolveDispatchExceptionRequest, ResolveDispatchExceptionResponse,
    ResolveSyncErrorRequest, ResolveSyncErrorResponse, StripeWebhookRequest, StripeWebhookResponse,
    TmsBulkStatusWebhookRequest, TmsBulkStatusWebhookResponse, TmsCloseRequest,
    TmsExternalRefRequest, TmsHandoffPayload, TmsHandoffResponse, TmsRequeueRequest,
    TmsStatusWebhookRequest, TmsWebhookResponse, TmsWithdrawRequest, UpsertLoadDocumentRequest,
    UpsertLoadDocumentResponse, VerifyLoadDocumentRequest, VerifyLoadDocumentResponse,
};
pub use auth_state::{
    AcceptLegalAgreementRequest, AcceptLegalAgreementResponse, AdminAccessElevationDecisionRequest,
    AdminAccessElevationRequestRow, AdminAccessReviewDecisionRequest, AdminAccessReviewItemRow,
    AdminAccessReviewMutationResponse, AdminAccessReviewRow, AdminAccessReviewScreen,
    AdminAuditEventRow, AdminAuditExportResponse, AdminAuditSearchFilters, AdminAuditSearchScreen,
    AdminBreakGlassRequest, AdminBreakGlassResponse, AdminCheckIdentityDomainDnsRequest,
    AdminCreateAccessElevationRequest, AdminCreateSupportCaseRequest,
    AdminCreateSupportNoteRequest, AdminCreateSupportNoteResponse, AdminCreateUserRequest,
    AdminCreateUserResponse, AdminDeleteUserResponse, AdminIdentityDomainRow,
    AdminIdentityMutationResponse, AdminIdentityProviderRow, AdminIdentityScimEventRow,
    AdminIdentityScreen, AdminOnboardingReviewScreen, AdminOnboardingReviewUser,
    AdminRolePermissionOption, AdminRolePermissionRow, AdminRolePermissionScreen,
    AdminStartAccessReviewRequest, AdminSupportCaseFeedbackRequest,
    AdminSupportCaseMutationResponse, AdminSupportCaseRow, AdminSupportCaseScreen,
    AdminSupportSearchFact, AdminSupportSearchResult, AdminSupportSearchScreen,
    AdminSupportTimelineEntry, AdminSupportTimelineRequest, AdminSupportTimelineScreen,
    AdminUpdateRolePermissionsRequest, AdminUpdateRolePermissionsResponse,
    AdminUpdateSupportCaseRequest, AdminUpdateUserProfileRequest, AdminUpdateUserProfileResponse,
    AdminUpdateUserRequest, AdminUpdateUserResponse, AdminUpsertIdentityDomainRequest,
    AdminUpsertIdentityProviderRequest, AdminUserDirectoryRoleOption, AdminUserDirectoryScreen,
    AdminUserDirectoryStatusOption, AdminUserDirectoryUser, AdminUserHistoryItem,
    AdminUserProfileFact, AdminUserProfileScreen, AdminVerifyIdentityDomainRequest,
    AuthOnboardingDraft, AuthOnboardingScreen, AuthSessionState, AuthSessionUser,
    CarrierCapacityProfile, ChangePasswordRequest, ChangePasswordResponse,
    DeleteKycDocumentResponse, EnterpriseSsoDiscoveryRequest, EnterpriseSsoDiscoveryResponse,
    EnterpriseSsoLoginResponse, EnterpriseSsoOidcCallbackRequest, ForgotPasswordRequest,
    ForgotPasswordResponse, KycDocumentItem, LegalAgreementAcceptanceItem, LegalAgreementScreen,
    LegalAgreementTemplateItem, LoginRequest, LoginResponse, LogoutResponse,
    MfaRecoveryCodesResponse, MfaVerifyRequest, MfaVerifyResponse, OtpPurpose,
    PortalRoleCountsResponse, RegisterRequest, RegisterResponse, ResendOtpRequest,
    ResendOtpResponse, ResetPasswordRequest, ResetPasswordResponse, ReviewOnboardingRequest,
    ReviewOnboardingResponse, ScimDeprovisionRequest, ScimDeprovisionResponse,
    ScimUpsertUserRequest, ScimUpsertUserResponse, SelfProfileDraft, SelfProfileFact,
    SelfProfileScreen, SubmitOnboardingRequest, SubmitOnboardingResponse,
    UpdateCarrierCapacityRequest, UpdateCarrierCapacityResponse, UpdateSelfProfileRequest,
    UpdateSelfProfileResponse, UpsertKycDocumentRequest, UpsertKycDocumentResponse,
    VerifyKycDocumentRequest, VerifyKycDocumentResponse, VerifyOtpRequest, VerifyOtpResponse,
};
pub use execution::{
    ExecutionActionItem, ExecutionCloseoutApprovalRequest, ExecutionCustomerTrackingLinkRequest,
    ExecutionCustomerTrackingLinkResponse, ExecutionCustomerTrackingRevokeRequest,
    ExecutionCustomerTrackingScreen, ExecutionDocumentItem, ExecutionDocumentTypeOption,
    ExecutionFinanceExceptionDecisionRequest, ExecutionFinanceExceptionRequest,
    ExecutionLegActionRequest, ExecutionLegActionResponse, ExecutionLegScreen,
    ExecutionLocationPingRequest, ExecutionLocationPingResponse, ExecutionNoteItem,
    ExecutionOfflineSubmissionRequest, ExecutionOfflineSubmissionResponse,
    ExecutionRoutePlanRequest, ExecutionStatusItem, ExecutionTelematicsConnectionRequest,
    ExecutionTelematicsPingRequest, ExecutionTimelineItem, ExecutionTrackingConsentRequest,
    ExecutionTrackingConsentResponse, ExecutionTrackingPointItem, ExecutionUploadDocumentResponse,
    ExecutionWorkflowMutationResponse,
};
pub use master_data::{
    CityUpsertRequest, CountryUpsertRequest, CustomerConfigurationRuleUpsertRequest,
    DocumentRequirementRuleUpsertRequest, GovernedCatalogUpsertRequest, LocationUpsertRequest,
    MasterDataCityOption, MasterDataDeleteRequest, MasterDataExportResponse,
    MasterDataImportRequest, MasterDataMutationResponse, MasterDataOption,
    MasterDataRollbackRequest, MasterDataRow, MasterDataScreen, MasterDataSection,
    MasterDataSummaryCard, SimpleCatalogUpsertRequest,
};
pub use realtime::{RealtimeEvent, RealtimeEventKind, RealtimeTopic};
pub use screens::{
    AdminLoadListScreen, AdminLoadRow, AdminLoadTab, CarrierMatchRow, CarrierMatchScreen,
    CarrierNetworkOption, CarrierNetworkRow, CarrierNetworkScreen, ChatConversationItem,
    ChatMessageItem, ChatOfferItem, ChatWorkspaceScreen, DispatchDeskLink, DispatchDeskRow,
    DispatchDeskScreen, ErrorBreakdownRow, HandoffRow, LoadBoardFilterOption, LoadBoardFilters,
    LoadBoardMetric, LoadBoardRow, LoadBoardSavedFilter, LoadBoardScreen, LoadBoardTab,
    LoadBuilderDraft, LoadBuilderLegDraft, LoadBuilderOption, LoadBuilderScreen, LoadDocumentRow,
    LoadHandoffSummary, LoadHistoryRow, LoadLifecycleAction, LoadProfileField, LoadProfileLegRow,
    LoadProfileScreen, MismatchCard, ReconciliationLogRow, StatusCard, StloadsOperationsScreen,
    StloadsReconciliationScreen, SyncIssueRow, SyncIssueSummary, UpsertCarrierNetworkRequest,
    UpsertCarrierNetworkResponse, sample_chat_workspace_screen, sample_load_board_screen,
    sample_stloads_operations_screen, sample_stloads_reconciliation_screen,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub status: String,
    pub data: T,
}

impl<T> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            status: "ok".to_string(),
            data,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub page: u64,
    pub per_page: u64,
    pub total: u64,
}
