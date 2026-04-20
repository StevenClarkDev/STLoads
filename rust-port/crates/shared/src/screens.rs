use crate::Pagination;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusCard {
    pub key: String,
    pub label: String,
    pub value: u64,
    pub tone: String,
    pub note: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncIssueSummary {
    pub total: u64,
    pub critical: u64,
    pub error: u64,
    pub warning: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncIssueRow {
    pub id: u64,
    pub severity: String,
    pub error_class: String,
    pub title: String,
    pub handoff_ref: Option<String>,
    pub created_at_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffRow {
    pub handoff_id: u64,
    pub handoff_ref: String,
    pub tms_load_id: String,
    pub route_label: String,
    pub freight_mode: String,
    pub equipment_type: String,
    pub rate_label: String,
    pub status_key: String,
    pub status_label: String,
    pub status_tone: String,
    pub load_number: Option<String>,
    pub retry_count: u64,
    pub pushed_at_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StloadsOperationsScreen {
    pub title: String,
    pub active_filter: Option<String>,
    pub sync_issue_summary: SyncIssueSummary,
    pub status_cards: Vec<StatusCard>,
    pub recent_sync_issues: Vec<SyncIssueRow>,
    pub handoffs: Vec<HandoffRow>,
    pub notes: Vec<String>,
    pub pagination: Pagination,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MismatchCard {
    pub label: String,
    pub value: u64,
    pub tone: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorBreakdownRow {
    pub error_class: String,
    pub severity: String,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationLogRow {
    pub id: u64,
    pub action: String,
    pub action_tone: String,
    pub handoff_ref: Option<String>,
    pub tms_transition: Option<String>,
    pub stloads_transition: Option<String>,
    pub detail: String,
    pub triggered_by: String,
    pub created_at_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StloadsReconciliationScreen {
    pub title: String,
    pub mismatch_cards: Vec<MismatchCard>,
    pub action_filters: Vec<String>,
    pub active_action: Option<String>,
    pub error_breakdown: Vec<ErrorBreakdownRow>,
    pub logs: Vec<ReconciliationLogRow>,
    pub callouts: Vec<String>,
    pub pagination: Pagination,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBoardTab {
    pub key: String,
    pub label: String,
    pub count: u64,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBoardMetric {
    pub label: String,
    pub value: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBoardRow {
    pub load_id: u64,
    pub leg_id: u64,
    pub leg_code: String,
    pub origin_label: String,
    pub destination_label: String,
    pub pickup_date_label: String,
    pub delivery_date_label: String,
    pub status_label: String,
    pub status_tone: String,
    pub stloads_label: Option<String>,
    pub stloads_tone: Option<String>,
    pub stloads_alert: Option<String>,
    pub remarks_label: Option<String>,
    pub carrier_label: Option<String>,
    pub booked_carrier_id: Option<u64>,
    pub bid_status_label: String,
    pub amount_label: String,
    pub payment_label: String,
    pub recommended_score: Option<u8>,
    pub primary_action_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBoardScreen {
    pub title: String,
    pub role_label: String,
    pub primary_action_label: Option<String>,
    pub primary_action_href: Option<String>,
    pub tabs: Vec<LoadBoardTab>,
    pub metrics: Vec<LoadBoardMetric>,
    pub rows: Vec<LoadBoardRow>,
    pub recommendation_notes: Vec<String>,
    pub pagination: Pagination,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchDeskLink {
    pub key: String,
    pub label: String,
    pub href: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchDeskRow {
    pub load_id: u64,
    pub leg_id: u64,
    pub handoff_id: Option<u64>,
    pub load_number: Option<String>,
    pub title: String,
    pub equipment_label: Option<String>,
    pub weight_label: Option<String>,
    pub carrier_label: Option<String>,
    pub payment_label: Option<String>,
    pub leg_status_label: String,
    pub leg_status_tone: String,
    pub stloads_label: Option<String>,
    pub stloads_tone: Option<String>,
    pub focus_label: String,
    pub focus_tone: String,
    pub focus_note: Option<String>,
    pub archive_guidance_label: Option<String>,
    pub archive_guidance_tone: Option<String>,
    pub archive_guidance_note: Option<String>,
    pub latest_activity_note: Option<String>,
    pub load_href: Option<String>,
    pub primary_action_key: Option<String>,
    pub primary_action_label: Option<String>,
    pub primary_action_enabled: bool,
    pub finance_action_key: Option<String>,
    pub finance_action_label: Option<String>,
    pub finance_action_enabled: bool,
    pub secondary_action_label: Option<String>,
    pub secondary_action_href: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchDeskScreen {
    pub desk_key: String,
    pub title: String,
    pub subtitle: String,
    pub desks: Vec<DispatchDeskLink>,
    pub quick_links: Vec<DispatchDeskLink>,
    pub status_cards: Vec<StatusCard>,
    pub rows: Vec<DispatchDeskRow>,
    pub notes: Vec<String>,
    pub pagination: Pagination,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminLoadTab {
    pub key: String,
    pub label: String,
    pub count: u64,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminLoadRow {
    pub load_id: u64,
    pub leg_id: u64,
    pub status_code: i16,
    pub leg_code: String,
    pub owner_label: String,
    pub carrier_label: Option<String>,
    pub origin_label: String,
    pub destination_label: String,
    pub pickup_date_label: String,
    pub delivery_date_label: String,
    pub status_label: String,
    pub status_tone: String,
    pub bid_status_label: String,
    pub amount_label: String,
    pub payment_label: Option<String>,
    pub finance_action_key: Option<String>,
    pub finance_action_label: Option<String>,
    pub finance_action_enabled: bool,
    pub finance_note: Option<String>,
    pub can_review: bool,
    pub primary_action_label: Option<String>,
    pub load_href: String,
    pub track_href: Option<String>,
    pub payments_href: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminLoadListScreen {
    pub title: String,
    pub subtitle: String,
    pub active_tab: String,
    pub tabs: Vec<AdminLoadTab>,
    pub rows: Vec<AdminLoadRow>,
    pub notes: Vec<String>,
    pub pagination: Pagination,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBuilderOption {
    pub id: u64,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBuilderLegDraft {
    pub pickup_location_address: String,
    pub pickup_city: Option<String>,
    pub pickup_country: Option<String>,
    pub pickup_place_id: Option<String>,
    pub pickup_latitude: Option<f64>,
    pub pickup_longitude: Option<f64>,
    pub delivery_location_address: String,
    pub delivery_city: Option<String>,
    pub delivery_country: Option<String>,
    pub delivery_place_id: Option<String>,
    pub delivery_latitude: Option<f64>,
    pub delivery_longitude: Option<f64>,
    pub pickup_date: String,
    pub delivery_date: String,
    pub bid_status: String,
    pub price: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBuilderDraft {
    pub load_id: u64,
    pub load_number: Option<String>,
    pub title: String,
    pub load_type_id: Option<u64>,
    pub equipment_id: Option<u64>,
    pub commodity_type_id: Option<u64>,
    pub weight_unit: Option<String>,
    pub weight: Option<f64>,
    pub special_instructions: Option<String>,
    pub is_hazardous: bool,
    pub is_temperature_controlled: bool,
    pub legs: Vec<LoadBuilderLegDraft>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBuilderScreen {
    pub title: String,
    pub subtitle: String,
    pub mode: String,
    pub submit_label: String,
    pub load_id: Option<u64>,
    pub draft: Option<LoadBuilderDraft>,
    pub load_type_options: Vec<LoadBuilderOption>,
    pub equipment_options: Vec<LoadBuilderOption>,
    pub commodity_type_options: Vec<LoadBuilderOption>,
    pub location_options: Vec<LoadBuilderOption>,
    pub weight_units: Vec<String>,
    pub bid_status_options: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadProfileField {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadProfileLegRow {
    pub leg_id: u64,
    pub status_code: i16,
    pub leg_code: String,
    pub route_label: String,
    pub pickup_date_label: String,
    pub delivery_date_label: String,
    pub status_label: String,
    pub status_tone: String,
    pub bid_status_label: String,
    pub amount_label: String,
    pub carrier_label: Option<String>,
    pub payment_label: Option<String>,
    pub finance_action_key: Option<String>,
    pub finance_action_label: Option<String>,
    pub finance_action_enabled: bool,
    pub payments_href: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadDocumentRow {
    pub id: u64,
    pub document_name: String,
    pub document_type_key: String,
    pub document_type_label: String,
    pub file_label: String,
    pub source_path: String,
    pub download_path: Option<String>,
    pub original_name: Option<String>,
    pub mime_type: Option<String>,
    pub file_size_bytes: Option<u64>,
    pub uploaded_by_label: Option<String>,
    pub can_view_file: bool,
    pub blockchain_label: Option<String>,
    pub blockchain_tone: Option<String>,
    pub blockchain_hash_preview: Option<String>,
    pub can_edit: bool,
    pub can_verify_blockchain: bool,
    pub uploaded_at_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadHistoryRow {
    pub id: u64,
    pub status_label: String,
    pub status_tone: String,
    pub remarks_label: String,
    pub actor_label: String,
    pub created_at_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadHandoffSummary {
    pub handoff_id: u64,
    pub status_label: String,
    pub status_tone: String,
    pub tms_load_id: String,
    pub board_rate_label: String,
    pub tms_status_label: Option<String>,
    pub tms_status_at_label: Option<String>,
    pub published_at_label: Option<String>,
    pub pushed_by_label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadProfileScreen {
    pub title: String,
    pub subtitle: String,
    pub load_id: u64,
    pub load_number: Option<String>,
    pub can_manage_documents: bool,
    pub info_fields: Vec<LoadProfileField>,
    pub legs: Vec<LoadProfileLegRow>,
    pub documents: Vec<LoadDocumentRow>,
    pub history: Vec<LoadHistoryRow>,
    pub stloads_handoff: Option<LoadHandoffSummary>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatConversationItem {
    pub id: u64,
    pub participant_user_id: u64,
    pub participant_name: String,
    pub participant_initials: String,
    pub load_leg_code: String,
    pub last_message_preview: String,
    pub last_seen_label: String,
    pub unread_count: u64,
    pub presence_label: Option<String>,
    pub presence_tone: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessageItem {
    pub id: u64,
    pub author_user_id: u64,
    pub author_name: String,
    pub sent_at_label: String,
    pub body: String,
    pub direction: String,
    pub receipt_label: Option<String>,
    pub receipt_tone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatOfferItem {
    pub offer_id: u64,
    pub amount_label: String,
    pub status_label: String,
    pub status_tone: String,
    pub created_at_label: String,
    pub can_accept: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatWorkspaceScreen {
    pub title: String,
    pub active_conversation_id: Option<u64>,
    pub active_participant: String,
    pub active_participant_user_id: Option<u64>,
    pub active_participant_presence_label: Option<String>,
    pub active_participant_presence_tone: Option<String>,
    pub active_participant_last_read_label: Option<String>,
    pub active_load_leg: String,
    pub composer_user_id: Option<u64>,
    pub smart_offer_label: String,
    pub smart_offer_tone: String,
    pub conversations: Vec<ChatConversationItem>,
    pub messages: Vec<ChatMessageItem>,
    pub offers: Vec<ChatOfferItem>,
    pub notes: Vec<String>,
}

pub fn sample_stloads_operations_screen() -> StloadsOperationsScreen {
    StloadsOperationsScreen {
        title: "STLOADS Operations".into(),
        active_filter: Some("published".into()),
        sync_issue_summary: SyncIssueSummary {
            total: 6,
            critical: 1,
            error: 3,
            warning: 2,
        },
        status_cards: vec![
            StatusCard { key: "queued".into(), label: "Queued".into(), value: 14, tone: "warning".into(), note: Some("Awaiting the next push cycle.".into()), is_active: false },
            StatusCard { key: "push_in_progress".into(), label: "In Progress".into(), value: 5, tone: "info".into(), note: Some("Currently publishing into STLOADS.".into()), is_active: false },
            StatusCard { key: "published".into(), label: "Published".into(), value: 62, tone: "success".into(), note: Some("Live on the board and visible to carriers.".into()), is_active: true },
            StatusCard { key: "push_failed".into(), label: "Failed".into(), value: 3, tone: "danger".into(), note: Some("Needs operator attention before retry.".into()), is_active: false },
            StatusCard { key: "requeue_required".into(), label: "Requeue".into(), value: 2, tone: "primary".into(), note: Some("Payload drift detected after initial publish.".into()), is_active: false },
            StatusCard { key: "withdrawn".into(), label: "Withdrawn".into(), value: 9, tone: "secondary".into(), note: Some("Removed from board after cancellation or completion.".into()), is_active: false },
            StatusCard { key: "closed".into(), label: "Closed".into(), value: 118, tone: "dark".into(), note: Some("Archived and ready for long-term audit only.".into()), is_active: false },
        ],
        recent_sync_issues: vec![
            SyncIssueRow { id: 8421, severity: "critical".into(), error_class: "duplicate_publish".into(), title: "TMS resent the same handoff while the previous board post remained live.".into(), handoff_ref: Some("#8421".into()), created_at_label: "5 minutes ago".into() },
            SyncIssueRow { id: 8407, severity: "error".into(), error_class: "delivered_still_open".into(), title: "Delivery completed upstream, but STLOADS still shows the lane as published.".into(), handoff_ref: Some("#8407".into()), created_at_label: "18 minutes ago".into() },
            SyncIssueRow { id: 8394, severity: "warning".into(), error_class: "rate_mismatch".into(), title: "Rate update arrived from TMS after the original publish payload.".into(), handoff_ref: Some("#8394".into()), created_at_label: "42 minutes ago".into() },
        ],
        handoffs: vec![
            HandoffRow { handoff_id: 8421, handoff_ref: "#8421".into(), tms_load_id: "TMS-110204".into(), route_label: "Dallas, TX -> Joliet, IL".into(), freight_mode: "FTL".into(), equipment_type: "Dry Van".into(), rate_label: "$2,450".into(), status_key: "published".into(), status_label: "Published".into(), status_tone: "success".into(), load_number: Some("LD-24017".into()), retry_count: 0, pushed_at_label: "Apr 5, 09:10".into() },
            HandoffRow { handoff_id: 8407, handoff_ref: "#8407".into(), tms_load_id: "TMS-110166".into(), route_label: "Houston, TX -> Memphis, TN".into(), freight_mode: "Reefer".into(), equipment_type: "53 ft Reefer".into(), rate_label: "$3,180".into(), status_key: "push_failed".into(), status_label: "Push Failed".into(), status_tone: "danger".into(), load_number: None, retry_count: 2, pushed_at_label: "Apr 5, 08:31".into() },
            HandoffRow { handoff_id: 8394, handoff_ref: "#8394".into(), tms_load_id: "TMS-110152".into(), route_label: "Ontario, CA -> Reno, NV".into(), freight_mode: "LTL".into(), equipment_type: "Straight Truck".into(), rate_label: "$1,260".into(), status_key: "requeue_required".into(), status_label: "Requeue Required".into(), status_tone: "primary".into(), load_number: Some("LD-24012".into()), retry_count: 1, pushed_at_label: "Apr 5, 07:58".into() },
            HandoffRow { handoff_id: 8368, handoff_ref: "#8368".into(), tms_load_id: "TMS-110090".into(), route_label: "Savannah, GA -> Newark, NJ".into(), freight_mode: "Drayage".into(), equipment_type: "Container Chassis".into(), rate_label: "$1,980".into(), status_key: "closed".into(), status_label: "Closed".into(), status_tone: "dark".into(), load_number: Some("LD-23977".into()), retry_count: 0, pushed_at_label: "Apr 4, 18:44".into() },
        ],
        notes: vec![
            "The unresolved issue banner is the top operational alert, just like the Blade screen.".into(),
            "Status filtering stays card-driven so ops can jump straight from counts into the affected handoffs.".into(),
        ],
        pagination: Pagination { page: 1, per_page: 25, total: 213 },
    }
}

pub fn sample_stloads_reconciliation_screen() -> StloadsReconciliationScreen {
    StloadsReconciliationScreen {
        title: "STLOADS Reconciliation".into(),
        mismatch_cards: vec![
            MismatchCard { label: "Published".into(), value: 62, tone: "success".into(), note: "All currently active board postings.".into() },
            MismatchCard { label: "TMS Cancelled".into(), value: 4, tone: "danger".into(), note: "Cancelled upstream but still visible on STLOADS.".into() },
            MismatchCard { label: "TMS Delivered".into(), value: 7, tone: "warning".into(), note: "Delivery closed in TMS while the board record remains live.".into() },
            MismatchCard { label: "TMS Invoiced/Settled".into(), value: 3, tone: "info".into(), note: "Finance completed upstream before STLOADS archived the posting.".into() },
            MismatchCard { label: "No TMS Status".into(), value: 6, tone: "secondary".into(), note: "Published records that never received a webhook update.".into() },
            MismatchCard { label: "Stale 30d+".into(), value: 5, tone: "dark".into(), note: "No webhook activity for more than thirty days.".into() },
        ],
        action_filters: vec!["all".into(), "status_update".into(), "auto_withdraw".into(), "auto_close".into(), "auto_archive".into(), "rate_update".into(), "mismatch_detected".into(), "force_sync".into()],
        active_action: Some("all".into()),
        error_breakdown: vec![
            ErrorBreakdownRow { error_class: "duplicate_publish".into(), severity: "critical".into(), count: 1 },
            ErrorBreakdownRow { error_class: "delivered_still_open".into(), severity: "error".into(), count: 2 },
            ErrorBreakdownRow { error_class: "withdraw_mismatch".into(), severity: "warning".into(), count: 3 },
        ],
        logs: vec![
            ReconciliationLogRow { id: 1904, action: "status_update".into(), action_tone: "info".into(), handoff_ref: Some("#8421 (TMS-110204)".into()), tms_transition: Some("in_transit -> delivered".into()), stloads_transition: Some("published -> published".into()), detail: "Webhook advanced TMS delivery state but local posting still waits for close logic.".into(), triggered_by: "webhook".into(), created_at_label: "Apr 5, 09:16".into() },
            ReconciliationLogRow { id: 1902, action: "rate_update".into(), action_tone: "primary".into(), handoff_ref: Some("#8394 (TMS-110152)".into()), tms_transition: None, stloads_transition: Some("published -> requeue_required".into()), detail: "Rate increased from $1,140 to $1,260 and downstream leg pricing needs refresh.".into(), triggered_by: "webhook".into(), created_at_label: "Apr 5, 08:48".into() },
            ReconciliationLogRow { id: 1897, action: "mismatch_detected".into(), action_tone: "danger".into(), handoff_ref: Some("#8407 (TMS-110166)".into()), tms_transition: Some("delivered -> delivered".into()), stloads_transition: Some("published -> published".into()), detail: "Scheduled scan found a delivered load still live on the public board.".into(), triggered_by: "cron".into(), created_at_label: "Apr 5, 08:15".into() },
            ReconciliationLogRow { id: 1892, action: "auto_withdraw".into(), action_tone: "warning".into(), handoff_ref: Some("#8365 (TMS-110081)".into()), tms_transition: Some("cancelled -> cancelled".into()), stloads_transition: Some("published -> withdrawn".into()), detail: "Cancellation webhook auto-withdrew the board listing.".into(), triggered_by: "webhook".into(), created_at_label: "Apr 5, 07:56".into() },
        ],
        callouts: vec![
            "The reconciliation screen is exception-oriented: counts first, then error breakdown, then audit log.".into(),
            "Action filters stay explicit because operators use them like saved lenses during cleanup sessions.".into(),
        ],
        pagination: Pagination { page: 1, per_page: 30, total: 128 },
    }
}

pub fn sample_load_board_screen() -> LoadBoardScreen {
    LoadBoardScreen {
        title: "Manage Loads".into(),
        role_label: "Carrier Workspace".into(),
        primary_action_label: Some("Set Preferences".into()),
        primary_action_href: Some("/loads/preferences".into()),
        tabs: vec![
            LoadBoardTab { key: "all".into(), label: "All Loads".into(), count: 128, is_active: true },
            LoadBoardTab { key: "recommended".into(), label: "Recommended".into(), count: 18, is_active: false },
            LoadBoardTab { key: "booked".into(), label: "Booked".into(), count: 11, is_active: false },
        ],
        metrics: vec![
            LoadBoardMetric { label: "Open Board".into(), value: "74 legs".into(), note: "Visible to carriers and not yet booked.".into() },
            LoadBoardMetric { label: "Recommended Matches".into(), value: "18 legs".into(), note: "Carrier preference scoring pulled these to the top.".into() },
            LoadBoardMetric { label: "Funding Watch".into(), value: "6 legs".into(), note: "Booked legs awaiting escrow funding or release.".into() },
        ],
        rows: vec![
            LoadBoardRow { load_id: 24017, leg_id: 240171, leg_code: "LD-24017-1".into(), origin_label: "Dallas, TX".into(), destination_label: "Joliet, IL".into(), pickup_date_label: "Apr 8, 2026".into(), delivery_date_label: "Apr 10, 2026".into(), status_label: "Booked".into(), status_tone: "primary".into(), stloads_label: Some("Published".into()), stloads_tone: Some("success".into()), stloads_alert: None, remarks_label: None, carrier_label: Some("Atlas Freight".into()), booked_carrier_id: Some(412), bid_status_label: "Fixed".into(), amount_label: "$2,450".into(), payment_label: "Escrow pending".into(), recommended_score: Some(96), primary_action_label: "Fund escrow".into() },
            LoadBoardRow { load_id: 24012, leg_id: 240122, leg_code: "LD-24012-2".into(), origin_label: "Ontario, CA".into(), destination_label: "Reno, NV".into(), pickup_date_label: "Apr 7, 2026".into(), delivery_date_label: "Apr 7, 2026".into(), status_label: "Published".into(), status_tone: "success".into(), stloads_label: Some("Requeue Required".into()), stloads_tone: Some("danger".into()), stloads_alert: Some("Rate drift after TMS update".into()), remarks_label: None, carrier_label: None, booked_carrier_id: None, bid_status_label: "Open".into(), amount_label: "$1,260".into(), payment_label: "Not funded".into(), recommended_score: Some(88), primary_action_label: "View offers".into() },
            LoadBoardRow { load_id: 23998, leg_id: 239981, leg_code: "LD-23998-1".into(), origin_label: "Houston, TX".into(), destination_label: "Memphis, TN".into(), pickup_date_label: "Apr 6, 2026".into(), delivery_date_label: "Apr 7, 2026".into(), status_label: "In Transit".into(), status_tone: "info".into(), stloads_label: Some("Published".into()), stloads_tone: Some("success".into()), stloads_alert: Some("Delivered upstream but still live on board".into()), remarks_label: Some("POD expected by end of shift.".into()), carrier_label: Some("Roadwise Logistics".into()), booked_carrier_id: Some(513), bid_status_label: "Fixed".into(), amount_label: "$3,180".into(), payment_label: "Funded".into(), recommended_score: None, primary_action_label: "Track leg".into() },
        ],
        recommendation_notes: vec![
            "Recommended loads are scored from preferred equipment, geography, and availability-day overlap.".into(),
            "The final Rust port should swap this static screen contract for DB-backed filtering and booking actions.".into(),
        ],
        pagination: Pagination { page: 1, per_page: 20, total: 128 },
    }
}

pub fn sample_chat_workspace_screen() -> ChatWorkspaceScreen {
    ChatWorkspaceScreen {
        title: "Private Chat".into(),
        active_conversation_id: Some(128),
        active_participant: "Atlas Freight".into(),
        active_participant_user_id: Some(412),
        active_participant_presence_label: Some("Online now".into()),
        active_participant_presence_tone: Some("success".into()),
        active_participant_last_read_label: Some("Read your latest update".into()),
        active_load_leg: "LD-24017-1".into(),
        composer_user_id: Some(71),
        smart_offer_label: "Pending - awaiting shipper".into(),
        smart_offer_tone: "warning".into(),
        conversations: vec![
            ChatConversationItem { id: 128, participant_user_id: 412, participant_name: "Atlas Freight".into(), participant_initials: "AF".into(), load_leg_code: "LD-24017-1".into(), last_message_preview: "We can hold the rate if pickup stays on Tuesday morning.".into(), last_seen_label: "2 min ago".into(), unread_count: 1, presence_label: Some("Online".into()), presence_tone: Some("success".into()), is_active: true },
            ChatConversationItem { id: 126, participant_user_id: 513, participant_name: "Roadwise Logistics".into(), participant_initials: "RL".into(), load_leg_code: "LD-23998-1".into(), last_message_preview: "POD will be uploaded after delivery.".into(), last_seen_label: "19 min ago".into(), unread_count: 0, presence_label: None, presence_tone: None, is_active: false },
            ChatConversationItem { id: 121, participant_user_id: 622, participant_name: "Blue Harbor Carrier".into(), participant_initials: "BH".into(), load_leg_code: "LD-23984-2".into(), last_message_preview: "Can you confirm appointment reference before we submit?".into(), last_seen_label: "1 h ago".into(), unread_count: 3, presence_label: None, presence_tone: None, is_active: false },
        ],
        messages: vec![
            ChatMessageItem { id: 5401, author_user_id: 412, author_name: "Maria Chen".into(), sent_at_label: "09:12".into(), body: "Pickup is locked for Tuesday at 08:00 local. Can your driver make that window?".into(), direction: "incoming".into(), receipt_label: None, receipt_tone: None },
            ChatMessageItem { id: 5402, author_user_id: 71, author_name: "You".into(), sent_at_label: "09:14".into(), body: "Yes. We can cover it, but we need the rate held at $2,450 including the fuel stop.".into(), direction: "outgoing".into(), receipt_label: Some("Read".into()), receipt_tone: Some("success".into()) },
            ChatMessageItem { id: 5403, author_user_id: 412, author_name: "Maria Chen".into(), sent_at_label: "09:16".into(), body: "That works. I pushed your offer to the top of the review queue.".into(), direction: "incoming".into(), receipt_label: None, receipt_tone: None },
        ],
        offers: vec![
            ChatOfferItem { offer_id: 901, amount_label: "$2,450".into(), status_label: "Pending".into(), status_tone: "warning".into(), created_at_label: "Submitted Apr 5, 09:10".into(), can_accept: true },
            ChatOfferItem { offer_id: 876, amount_label: "$2,390".into(), status_label: "Declined".into(), status_tone: "danger".into(), created_at_label: "Submitted Apr 4, 17:42".into(), can_accept: false },
            ChatOfferItem { offer_id: 852, amount_label: "$2,520".into(), status_label: "Approved".into(), status_tone: "success".into(), created_at_label: "Submitted Apr 3, 14:20".into(), can_accept: false },
        ],
        notes: vec![
            "The workspace keeps conversation list, active thread, and offer history in a single route like the Blade view.".into(),
            "Read receipts and presence are now part of the shared screen contract for the Rust cutover.".into(),
        ],
    }
}
