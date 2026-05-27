use serde::{Deserialize, Serialize};

use crate::RequiredDocumentChecklistItem;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionActionItem {
    pub key: String,
    pub label: String,
    pub description: String,
    pub is_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTimelineItem {
    pub id: u64,
    pub event_type_key: String,
    pub event_type_label: String,
    pub created_at_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionNoteItem {
    pub id: u64,
    pub actor_label: String,
    pub status_label: String,
    pub remarks_label: String,
    pub created_at_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTrackingPointItem {
    pub id: u64,
    pub lat: f64,
    pub lng: f64,
    pub recorded_at_label: String,
    pub is_latest: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionDocumentTypeOption {
    pub key: String,
    pub label: String,
    pub description: String,
    pub mobile_capture_hint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionDocumentItem {
    pub id: u64,
    pub document_type_key: String,
    pub document_type_label: String,
    pub file_label: String,
    pub source_path: String,
    pub download_path: Option<String>,
    pub uploaded_by_label: Option<String>,
    pub can_view_file: bool,
    pub current_version: u32,
    pub version_count: u64,
    pub version_history_label: String,
    pub created_at_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStatusItem {
    pub key: String,
    pub label: String,
    pub status_label: String,
    pub status_tone: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionLegScreen {
    pub title: String,
    pub subtitle: String,
    pub leg_id: u64,
    pub load_id: u64,
    pub load_number: Option<String>,
    pub leg_code: String,
    pub route_label: String,
    pub status_label: String,
    pub status_tone: String,
    pub carrier_label: Option<String>,
    pub operator_mode_label: String,
    pub latest_location_label: Option<String>,
    pub latest_coordinate_label: Option<String>,
    pub latest_map_url: Option<String>,
    pub tracking_summary_label: Option<String>,
    pub tracking_health_label: Option<String>,
    pub tracking_health_tone: String,
    pub tracking_consent_required: bool,
    pub tracking_consent_granted: bool,
    pub tracking_consent_text: String,
    pub tracking_retention_label: String,
    pub customer_tracking_scope_label: String,
    pub field_capture_strategy_label: String,
    pub offline_strategy_label: String,
    pub mobile_support_label: String,
    pub geofence_status_label: Option<String>,
    pub geofence_status_tone: String,
    pub eta_risk_label: Option<String>,
    pub eta_risk_tone: String,
    pub closeout_ready: bool,
    pub closeout_package_label: String,
    pub closeout_package_tone: String,
    pub closeout_export_path: Option<String>,
    pub customer_tracking_path: Option<String>,
    pub offline_submission_count: u64,
    pub pending_offline_submission_count: u64,
    pub offline_submission_status_label: String,
    pub telematics_status_label: String,
    pub route_plan_label: String,
    pub route_plan_tone: String,
    pub closeout_checklist: Vec<ExecutionStatusItem>,
    pub claims_accessorial_items: Vec<ExecutionStatusItem>,
    pub next_action_label: Option<String>,
    pub can_manage_execution: bool,
    pub can_send_location_ping: bool,
    pub live_tracking_available: bool,
    pub live_tracking_note: Option<String>,
    pub can_upload_documents: bool,
    pub delivery_completion_ready: bool,
    pub delivery_completion_note: Option<String>,
    pub document_type_options: Vec<ExecutionDocumentTypeOption>,
    pub action_items: Vec<ExecutionActionItem>,
    pub timeline: Vec<ExecutionTimelineItem>,
    pub notes_history: Vec<ExecutionNoteItem>,
    pub tracking_points: Vec<ExecutionTrackingPointItem>,
    pub documents: Vec<ExecutionDocumentItem>,
    pub required_documents: Vec<RequiredDocumentChecklistItem>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionLegActionRequest {
    pub action_key: String,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTrackingConsentRequest {
    pub consent_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTrackingConsentResponse {
    pub success: bool,
    pub leg_id: u64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionOfflineSubmissionRequest {
    pub client_submission_id: String,
    pub submission_type: String,
    pub payload: serde_json::Value,
    pub captured_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionOfflineSubmissionResponse {
    pub success: bool,
    pub leg_id: u64,
    pub submission_id: Option<u64>,
    pub status_label: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionLegActionResponse {
    pub success: bool,
    pub leg_id: u64,
    pub status_label: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionLocationPingRequest {
    pub lat: f64,
    pub lng: f64,
    pub recorded_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionLocationPingResponse {
    pub success: bool,
    pub leg_id: u64,
    pub latest_location_label: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionUploadDocumentResponse {
    pub success: bool,
    pub leg_id: u64,
    pub document_id: Option<u64>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionCustomerTrackingScreen {
    pub leg_code: String,
    pub load_number: Option<String>,
    pub route_label: String,
    pub status_label: String,
    pub latest_location_label: Option<String>,
    pub latest_coordinate_label: Option<String>,
    pub tracking_health_label: Option<String>,
    pub geofence_status_label: Option<String>,
    pub eta_risk_label: Option<String>,
    pub expires_at_label: String,
    pub visibility_scope_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionCustomerTrackingLinkRequest {
    pub visibility_scope: String,
    pub expires_in_hours: Option<i64>,
    pub rotate_existing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionCustomerTrackingRevokeRequest {
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionCustomerTrackingLinkResponse {
    pub success: bool,
    pub leg_id: u64,
    pub customer_tracking_path: Option<String>,
    pub expires_at_label: Option<String>,
    pub status_label: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionCloseoutApprovalRequest {
    pub pod_review_status: String,
    pub export_path: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionFinanceExceptionRequest {
    pub exception_type: String,
    pub status: String,
    pub amount_cents: Option<i64>,
    pub visibility: String,
    pub description: String,
    pub evidence_document_id: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionFinanceExceptionDecisionRequest {
    pub exception_type: String,
    pub status: String,
    pub resolution_note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTelematicsConnectionRequest {
    pub provider_key: String,
    pub status: String,
    pub fallback_behavior: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTelematicsPingRequest {
    pub provider_key: String,
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    pub recorded_at: Option<String>,
    pub hos_status: Option<String>,
    pub truck_id: Option<String>,
    pub trailer_id: Option<String>,
    pub event_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRoutePlanRequest {
    pub provider_key: String,
    pub distance_miles: Option<f64>,
    pub duration_minutes: Option<i32>,
    pub truck_safe: bool,
    pub status: String,
    pub constraints: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionWorkflowMutationResponse {
    pub success: bool,
    pub leg_id: u64,
    pub status_label: String,
    pub message: String,
}
