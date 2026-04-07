use serde::{Deserialize, Serialize, de::DeserializeOwned};
use shared::{
    ApiResponse, AuthSessionState, BookLoadLegRequest, BookLoadLegResponse, ChatSendMessageRequest,
    ChatSendMessageResponse, ChatWorkspaceScreen, ConversationReadResponse, CreateLoadRequest,
    CreateLoadResponse, EscrowFundRequest, EscrowHoldRequest, EscrowLifecycleResponse,
    EscrowReleaseRequest, LoadBoardScreen, LoadBuilderScreen, LocationUpsertRequest, LoginRequest,
    LoginResponse, LogoutResponse, MasterDataMutationResponse, MasterDataScreen,
    OfferReviewRequest, OfferReviewResponse, RealtimeTopic, ResolveSyncErrorRequest,
    ResolveSyncErrorResponse, SimpleCatalogUpsertRequest, StloadsOperationsScreen,
    StloadsReconciliationScreen, StripeWebhookRequest, StripeWebhookResponse, TmsCloseRequest,
    TmsHandoffPayload, TmsHandoffResponse, TmsRequeueRequest, TmsStatusWebhookRequest,
    TmsWebhookResponse, TmsWithdrawRequest,
};

#[derive(Debug, Clone, Deserialize)]
pub struct AdminOverview {
    pub screen_routes: Vec<String>,
    pub operational_views: usize,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaymentsOverview {
    pub contract: serde_json::Value,
    pub escrow_statuses: usize,
    pub webhook_events: usize,
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

#[cfg(target_arch = "wasm32")]
const AUTH_TOKEN_STORAGE_KEY: &str = "stloads_rust_auth_token";

pub async fn fetch_auth_session() -> Result<AuthSessionState, String> {
    get_api("/auth/session").await
}

pub async fn login(payload: &LoginRequest) -> Result<LoginResponse, String> {
    let response: LoginResponse = post_api("/auth/login", payload).await?;
    if response.success {
        if let Some(token) = response.token.as_deref() {
            store_auth_token(token);
        }
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

pub async fn fetch_admin_overview() -> Result<AdminOverview, String> {
    get_api("/admin").await
}

pub async fn fetch_master_data_screen() -> Result<MasterDataScreen, String> {
    get_api("/master-data/screen").await
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

pub async fn fetch_load_builder_screen() -> Result<LoadBuilderScreen, String> {
    get_api("/dispatch/load-builder").await
}

pub async fn create_load(payload: &CreateLoadRequest) -> Result<CreateLoadResponse, String> {
    post_api("/dispatch/loads", payload).await
}

pub async fn fetch_load_board_screen(tab: &str) -> Result<LoadBoardScreen, String> {
    let path = format!("/dispatch/load-board?tab={}", tab);
    get_api(&path).await
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

pub async fn resolve_sync_error(
    sync_error_id: u64,
    payload: &ResolveSyncErrorRequest,
) -> Result<ResolveSyncErrorResponse, String> {
    let path = format!("/admin/stloads/sync-errors/{}/resolve", sync_error_id);
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

fn api_url(path: &str) -> String {
    let normalized_path = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{}", path)
    };

    let base = configured_api_base();
    if base.is_empty() {
        normalized_path
    } else {
        format!("{}{}", base, normalized_path)
    }
}

#[cfg(target_arch = "wasm32")]
fn configured_api_base() -> String {
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
    let mut request = gloo_net::http::Request::get(url);
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
    let mut request = reqwest::Client::new().get(url);
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
