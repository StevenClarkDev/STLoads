use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookLoadLegRequest {
    pub booked_amount: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookLoadLegResponse {
    pub success: bool,
    pub leg_id: i64,
    pub status_label: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLoadLegRequest {
    pub pickup_location_id: Option<u64>,
    pub pickup_location_address: Option<String>,
    pub pickup_city: Option<String>,
    pub pickup_country: Option<String>,
    pub pickup_place_id: Option<String>,
    pub pickup_latitude: Option<f64>,
    pub pickup_longitude: Option<f64>,
    pub delivery_location_id: Option<u64>,
    pub delivery_location_address: Option<String>,
    pub delivery_city: Option<String>,
    pub delivery_country: Option<String>,
    pub delivery_place_id: Option<String>,
    pub delivery_latitude: Option<f64>,
    pub delivery_longitude: Option<f64>,
    pub pickup_date: String,
    pub delivery_date: String,
    pub bid_status: String,
    pub price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLoadRequest {
    pub title: String,
    pub load_type_id: u64,
    pub equipment_id: u64,
    pub commodity_type_id: u64,
    pub weight_unit: String,
    pub weight: f64,
    pub special_instructions: Option<String>,
    pub is_hazardous: bool,
    pub is_temperature_controlled: bool,
    pub legs: Vec<CreateLoadLegRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLoadResponse {
    pub success: bool,
    pub load_id: Option<i64>,
    pub load_number: Option<String>,
    pub leg_count: u64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertLoadDocumentRequest {
    pub document_name: String,
    pub document_type: String,
    pub file_path: String,
    pub original_name: Option<String>,
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertLoadDocumentResponse {
    pub success: bool,
    pub load_id: i64,
    pub document_id: Option<i64>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyLoadDocumentRequest {
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyLoadDocumentResponse {
    pub success: bool,
    pub load_id: i64,
    pub document_id: i64,
    pub hash: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchDeskFollowUpRequest {
    pub desk_key: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchDeskFollowUpResponse {
    pub success: bool,
    pub leg_id: i64,
    pub load_id: i64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminReviewLoadRequest {
    pub decision: String,
    pub remarks: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminReviewLoadResponse {
    pub success: bool,
    pub load_id: i64,
    pub status_label: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OfferReviewDecision {
    Accept,
    Decline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfferReviewRequest {
    pub decision: OfferReviewDecision,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfferReviewResponse {
    pub success: bool,
    pub offer_id: i64,
    pub leg_id: i64,
    pub status_label: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSendMessageRequest {
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSendMessageResponse {
    pub success: bool,
    pub conversation_id: i64,
    pub message_id: i64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationReadResponse {
    pub success: bool,
    pub conversation_id: i64,
    pub last_read_message_id: Option<i64>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveSyncErrorRequest {
    pub resolution_note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveSyncErrorResponse {
    pub success: bool,
    pub sync_error_id: i64,
    pub handoff_id: Option<i64>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscrowFundRequest {
    pub amount_cents: Option<i64>,
    pub currency: Option<String>,
    pub platform_fee_cents: Option<i64>,
    pub payment_intent_id: Option<String>,
    pub charge_id: Option<String>,
    pub transfer_group: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscrowHoldRequest {
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscrowReleaseRequest {
    pub transfer_id: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscrowLifecycleResponse {
    pub success: bool,
    pub leg_id: i64,
    pub escrow_id: Option<i64>,
    pub payment_intent_id: Option<String>,
    pub client_secret: Option<String>,
    pub transfer_id: Option<String>,
    pub status_label: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StripeWebhookRequest {
    pub event_type: String,
    pub leg_id: Option<i64>,
    pub payment_intent_id: Option<String>,
    pub charge_id: Option<String>,
    pub transfer_id: Option<String>,
    pub transfer_group: Option<String>,
    pub amount_cents: Option<i64>,
    pub currency: Option<String>,
    pub platform_fee_cents: Option<i64>,
    pub stripe_account_id: Option<String>,
    pub payouts_enabled: Option<bool>,
    pub kyc_status: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StripeWebhookResponse {
    pub acknowledged: bool,
    pub event_type: String,
    pub leg_id: Option<i64>,
    pub user_id: Option<i64>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmsExternalRefRequest {
    pub ref_type: String,
    pub ref_value: String,
    pub ref_source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmsHandoffPayload {
    pub tms_load_id: String,
    pub tenant_id: String,
    pub external_handoff_id: Option<String>,
    pub party_type: String,
    pub freight_mode: String,
    pub equipment_type: String,
    pub commodity_description: Option<String>,
    pub weight: f64,
    pub weight_unit: String,
    pub piece_count: Option<i32>,
    pub is_hazardous: Option<bool>,
    pub temperature_data: Option<Value>,
    pub container_data: Option<Value>,
    pub securement_data: Option<Value>,
    pub pickup_city: String,
    pub pickup_state: Option<String>,
    pub pickup_zip: Option<String>,
    pub pickup_country: String,
    pub pickup_address: String,
    pub pickup_window_start: String,
    pub pickup_window_end: Option<String>,
    pub pickup_instructions: Option<String>,
    pub pickup_appointment_ref: Option<String>,
    pub dropoff_city: String,
    pub dropoff_state: Option<String>,
    pub dropoff_zip: Option<String>,
    pub dropoff_country: String,
    pub dropoff_address: String,
    pub dropoff_window_start: String,
    pub dropoff_window_end: Option<String>,
    pub dropoff_instructions: Option<String>,
    pub dropoff_appointment_ref: Option<String>,
    pub board_rate: f64,
    pub rate_currency: Option<String>,
    pub accessorial_flags: Option<Value>,
    pub bid_type: String,
    pub quote_status: Option<String>,
    pub tender_posture: Option<String>,
    pub compliance_passed: Option<bool>,
    pub compliance_summary: Option<Value>,
    pub required_documents_status: Option<Value>,
    pub readiness: Option<String>,
    pub pushed_by: Option<String>,
    pub push_reason: Option<String>,
    pub source_module: Option<String>,
    pub payload_version: Option<String>,
    pub external_refs: Option<Vec<TmsExternalRefRequest>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmsHandoffResponse {
    pub success: bool,
    pub handoff_id: Option<i64>,
    pub load_id: Option<i64>,
    pub load_number: Option<String>,
    pub status_label: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmsRequeueRequest {
    pub handoff_id: i64,
    pub pushed_by: Option<String>,
    pub source_module: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmsWithdrawRequest {
    pub handoff_id: i64,
    pub reason: Option<String>,
    pub pushed_by: Option<String>,
    pub source_module: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmsCloseRequest {
    pub handoff_id: i64,
    pub reason: Option<String>,
    pub pushed_by: Option<String>,
    pub source_module: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmsStatusWebhookRequest {
    pub tms_load_id: String,
    pub tenant_id: String,
    pub tms_status: String,
    pub status_at: Option<String>,
    pub source_module: Option<String>,
    pub pushed_by: Option<String>,
    pub detail: Option<String>,
    pub rate_update: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmsBulkStatusWebhookRequest {
    pub updates: Vec<TmsStatusWebhookRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmsWebhookResponse {
    pub success: bool,
    pub handoff_id: Option<i64>,
    pub action_label: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmsBulkStatusWebhookResponse {
    pub processed: usize,
    pub updated: usize,
    pub missing: usize,
    pub failed: usize,
    pub messages: Vec<String>,
}
