pub mod actions;
pub mod auth_state;
pub mod execution;
pub mod master_data;
pub mod realtime;
pub mod screens;

use serde::{Deserialize, Serialize};

pub use actions::{
    AdminReviewLoadRequest, AdminReviewLoadResponse, BookLoadLegRequest, BookLoadLegResponse,
    ChatSendMessageRequest, ChatSendMessageResponse, ConversationReadResponse,
    CreateLoadLegRequest, CreateLoadRequest, CreateLoadResponse, DispatchDeskFollowUpRequest,
    DispatchDeskFollowUpResponse, EscrowFundRequest, EscrowHoldRequest, EscrowLifecycleResponse,
    EscrowReleaseRequest, OfferReviewDecision, OfferReviewRequest, OfferReviewResponse,
    ResolveSyncErrorRequest, ResolveSyncErrorResponse, StripeWebhookRequest, StripeWebhookResponse,
    TmsBulkStatusWebhookRequest, TmsBulkStatusWebhookResponse, TmsCloseRequest,
    TmsExternalRefRequest, TmsHandoffPayload, TmsHandoffResponse, TmsRequeueRequest,
    TmsStatusWebhookRequest, TmsWebhookResponse, TmsWithdrawRequest, UpsertLoadDocumentRequest,
    UpsertLoadDocumentResponse, VerifyLoadDocumentRequest, VerifyLoadDocumentResponse,
};
pub use auth_state::{
    AdminCreateUserRequest, AdminCreateUserResponse, AdminDeleteUserResponse,
    AdminOnboardingReviewScreen, AdminOnboardingReviewUser, AdminRolePermissionOption,
    AdminRolePermissionRow, AdminRolePermissionScreen, AdminUpdateRolePermissionsRequest,
    AdminUpdateRolePermissionsResponse, AdminUpdateUserProfileRequest,
    AdminUpdateUserProfileResponse, AdminUpdateUserRequest, AdminUpdateUserResponse,
    AdminUserDirectoryRoleOption, AdminUserDirectoryScreen, AdminUserDirectoryStatusOption,
    AdminUserDirectoryUser, AdminUserHistoryItem, AdminUserProfileFact, AdminUserProfileScreen,
    AuthOnboardingDraft, AuthOnboardingScreen, AuthSessionState, AuthSessionUser,
    ChangePasswordRequest, ChangePasswordResponse, DeleteKycDocumentResponse,
    ForgotPasswordRequest, ForgotPasswordResponse, KycDocumentItem, LoginRequest, LoginResponse,
    LogoutResponse, OtpPurpose, RegisterRequest, RegisterResponse, ResendOtpRequest,
    ResendOtpResponse, ResetPasswordRequest, ResetPasswordResponse, ReviewOnboardingRequest,
    ReviewOnboardingResponse, SelfProfileDraft, SelfProfileFact, SelfProfileScreen,
    SubmitOnboardingRequest, SubmitOnboardingResponse, UpdateSelfProfileRequest,
    UpdateSelfProfileResponse, UpsertKycDocumentRequest, UpsertKycDocumentResponse,
    VerifyKycDocumentRequest, VerifyKycDocumentResponse, VerifyOtpRequest, VerifyOtpResponse,
};
pub use execution::{
    ExecutionActionItem, ExecutionDocumentItem, ExecutionDocumentTypeOption,
    ExecutionLegActionRequest, ExecutionLegActionResponse, ExecutionLegScreen,
    ExecutionLocationPingRequest, ExecutionLocationPingResponse, ExecutionNoteItem,
    ExecutionTimelineItem, ExecutionTrackingPointItem, ExecutionUploadDocumentResponse,
};
pub use master_data::{
    CityUpsertRequest, CountryUpsertRequest, LocationUpsertRequest, MasterDataCityOption,
    MasterDataDeleteRequest, MasterDataMutationResponse, MasterDataOption, MasterDataRow,
    MasterDataScreen, MasterDataSection, MasterDataSummaryCard, SimpleCatalogUpsertRequest,
};
pub use realtime::{RealtimeEvent, RealtimeEventKind, RealtimeTopic};
pub use screens::{
    AdminLoadListScreen, AdminLoadRow, AdminLoadTab, ChatConversationItem, ChatMessageItem,
    ChatOfferItem, ChatWorkspaceScreen, DispatchDeskLink, DispatchDeskRow, DispatchDeskScreen,
    ErrorBreakdownRow, HandoffRow, LoadBoardMetric, LoadBoardRow, LoadBoardScreen, LoadBoardTab,
    LoadBuilderDraft, LoadBuilderLegDraft, LoadBuilderOption, LoadBuilderScreen, LoadDocumentRow,
    LoadHandoffSummary, LoadHistoryRow, LoadProfileField, LoadProfileLegRow, LoadProfileScreen,
    MismatchCard, ReconciliationLogRow, StatusCard, StloadsOperationsScreen,
    StloadsReconciliationScreen, SyncIssueRow, SyncIssueSummary, sample_chat_workspace_screen,
    sample_load_board_screen, sample_stloads_operations_screen,
    sample_stloads_reconciliation_screen,
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
