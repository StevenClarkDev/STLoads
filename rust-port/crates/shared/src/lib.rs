pub mod actions;
pub mod auth_state;
pub mod master_data;
pub mod realtime;
pub mod screens;

use serde::{Deserialize, Serialize};

pub use actions::{
    BookLoadLegRequest, BookLoadLegResponse, ChatSendMessageRequest, ChatSendMessageResponse,
    ConversationReadResponse, EscrowFundRequest, EscrowHoldRequest, EscrowLifecycleResponse,
    EscrowReleaseRequest, OfferReviewDecision, OfferReviewRequest, OfferReviewResponse,
    ResolveSyncErrorRequest, ResolveSyncErrorResponse, StripeWebhookRequest, StripeWebhookResponse,
    TmsBulkStatusWebhookRequest, TmsBulkStatusWebhookResponse, TmsCloseRequest,
    TmsExternalRefRequest, TmsHandoffPayload, TmsHandoffResponse, TmsRequeueRequest,
    TmsStatusWebhookRequest, TmsWebhookResponse, TmsWithdrawRequest,
};
pub use auth_state::{
    AuthSessionState, AuthSessionUser, LoginRequest, LoginResponse, LogoutResponse,
};
pub use master_data::{
    LocationUpsertRequest, MasterDataCityOption, MasterDataMutationResponse, MasterDataOption,
    MasterDataRow, MasterDataScreen, MasterDataSection, MasterDataSummaryCard,
    SimpleCatalogUpsertRequest,
};
pub use realtime::{RealtimeEvent, RealtimeEventKind, RealtimeTopic};
pub use screens::{
    ChatConversationItem, ChatMessageItem, ChatOfferItem, ChatWorkspaceScreen, ErrorBreakdownRow,
    HandoffRow, LoadBoardMetric, LoadBoardRow, LoadBoardScreen, LoadBoardTab, MismatchCard,
    ReconciliationLogRow, StatusCard, StloadsOperationsScreen, StloadsReconciliationScreen,
    SyncIssueRow, SyncIssueSummary, sample_chat_workspace_screen, sample_load_board_screen,
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
