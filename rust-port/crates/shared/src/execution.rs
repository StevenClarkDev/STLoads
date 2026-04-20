use serde::{Deserialize, Serialize};

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
    pub created_at_label: String,
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
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionLegActionRequest {
    pub action_key: String,
    pub note: Option<String>,
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
