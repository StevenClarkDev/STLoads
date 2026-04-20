use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RealtimeTopic {
    Conversation,
    LoadBoard,
    ExecutionTracking,
    AdminDashboard,
    AdminTmsOperations,
    AdminTmsReconciliation,
    AdminPayments,
}

impl RealtimeTopic {
    pub const fn as_key(self) -> &'static str {
        match self {
            Self::Conversation => "conversation",
            Self::LoadBoard => "load_board",
            Self::ExecutionTracking => "execution_tracking",
            Self::AdminDashboard => "admin_dashboard",
            Self::AdminTmsOperations => "admin_tms_operations",
            Self::AdminTmsReconciliation => "admin_tms_reconciliation",
            Self::AdminPayments => "admin_payments",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RealtimeEventKind {
    SessionInvalidated,
    LoadLegBooked,
    LegExecutionUpdated,
    LegLocationUpdated,
    OfferReviewed,
    MessageSent,
    ConversationRead,
    ConversationPresenceChanged,
    AdminDashboardUpdated,
    TmsOperationsUpdated,
    TmsReconciliationUpdated,
    PaymentsOperationsUpdated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeEvent {
    pub kind: RealtimeEventKind,
    pub leg_id: Option<u64>,
    pub conversation_id: Option<u64>,
    pub offer_id: Option<u64>,
    pub message_id: Option<u64>,
    pub actor_user_id: Option<u64>,
    pub subject_user_id: Option<u64>,
    pub presence_state: Option<String>,
    pub last_read_message_id: Option<u64>,
    pub summary: String,
}
