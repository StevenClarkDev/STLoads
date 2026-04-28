use serde::{Deserialize, Serialize, de::DeserializeOwned};
use shared::{
    AdminCreateUserRequest, AdminCreateUserResponse, AdminDeleteUserResponse, AdminLoadListScreen,
    AdminOnboardingReviewScreen, AdminReviewLoadRequest, AdminReviewLoadResponse,
    AdminRolePermissionScreen, AdminUpdateRolePermissionsRequest,
    AdminUpdateRolePermissionsResponse, AdminUpdateUserProfileRequest,
    AdminUpdateUserProfileResponse, AdminUpdateUserRequest, AdminUpdateUserResponse,
    AdminUserDirectoryScreen, AdminUserProfileScreen, ApiResponse, AuthOnboardingScreen,
    AuthSessionState, BookLoadLegRequest, BookLoadLegResponse, ChangePasswordRequest,
    ChangePasswordResponse, ChatSendMessageRequest, ChatSendMessageResponse, ChatWorkspaceScreen,
    CityUpsertRequest, ConversationReadResponse, CountryUpsertRequest, CreateLoadRequest,
    CreateLoadResponse, DeleteKycDocumentResponse, DispatchDeskFollowUpRequest,
    DispatchDeskFollowUpResponse, DispatchDeskScreen, EscrowFundRequest, EscrowHoldRequest,
    EscrowLifecycleResponse, EscrowReleaseRequest, ExecutionLegActionRequest,
    ExecutionLegActionResponse, ExecutionLegScreen, ExecutionLocationPingRequest,
    ExecutionLocationPingResponse, ForgotPasswordRequest, ForgotPasswordResponse, LoadBoardScreen,
    LoadBuilderScreen, LoadProfileScreen, LocationUpsertRequest, LoginRequest, LoginResponse,
    LogoutResponse, MasterDataDeleteRequest, MasterDataMutationResponse, MasterDataScreen,
    OfferReviewRequest, OfferReviewResponse, RealtimeTopic, RegisterRequest, RegisterResponse,
    ResendOtpRequest, ResendOtpResponse, ResetPasswordRequest, ResetPasswordResponse,
    ResolveSyncErrorRequest, ResolveSyncErrorResponse, ReviewOnboardingRequest,
    ReviewOnboardingResponse, SelfProfileScreen, SimpleCatalogUpsertRequest,
    StloadsOperationsScreen, StloadsReconciliationScreen, StripeWebhookRequest,
    StripeWebhookResponse, SubmitOnboardingRequest, SubmitOnboardingResponse, TmsCloseRequest,
    TmsHandoffPayload, TmsHandoffResponse, TmsRequeueRequest, TmsStatusWebhookRequest,
    TmsWebhookResponse, TmsWithdrawRequest, UpdateSelfProfileRequest, UpdateSelfProfileResponse,
    UpsertKycDocumentRequest, UpsertKycDocumentResponse, UpsertLoadDocumentRequest,
    UpsertLoadDocumentResponse, VerifyKycDocumentRequest, VerifyKycDocumentResponse,
    VerifyLoadDocumentRequest, VerifyLoadDocumentResponse, VerifyOtpRequest, VerifyOtpResponse,
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

pub async fn register(payload: &RegisterRequest) -> Result<RegisterResponse, String> {
    post_api("/auth/register", payload).await
}

pub async fn verify_otp(payload: &VerifyOtpRequest) -> Result<VerifyOtpResponse, String> {
    let response: VerifyOtpResponse = post_api("/auth/verify-otp", payload).await?;
    if response.success {
        if let Some(token) = response.token.as_deref() {
            store_auth_token(token);
        }
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

pub async fn change_password(
    payload: &ChangePasswordRequest,
) -> Result<ChangePasswordResponse, String> {
    post_api("/auth/change-password", payload).await
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

pub async fn fetch_load_board_screen(tab: &str) -> Result<LoadBoardScreen, String> {
    let path = format!("/dispatch/load-board?tab={}", tab);
    get_api(&path).await
}

pub async fn fetch_dispatch_desk_screen(desk_key: &str) -> Result<DispatchDeskScreen, String> {
    let path = format!("/dispatch/desk/{}", desk_key);
    get_api(&path).await
}

pub async fn add_dispatch_desk_follow_up(
    leg_id: u64,
    payload: &DispatchDeskFollowUpRequest,
) -> Result<DispatchDeskFollowUpResponse, String> {
    let path = format!("/dispatch/desk/legs/{}/follow-up", leg_id);
    post_api(&path, payload).await
}

pub async fn fetch_load_profile_screen(load_id: u64) -> Result<LoadProfileScreen, String> {
    let path = format!("/dispatch/loads/{}", load_id);
    get_api(&path).await
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

pub async fn fetch_admin_onboarding_reviews() -> Result<AdminOnboardingReviewScreen, String> {
    get_api("/admin/onboarding-reviews").await
}

pub async fn fetch_admin_user_directory() -> Result<AdminUserDirectoryScreen, String> {
    get_api("/admin/users").await
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

    let base = configured_api_base();
    if base.is_empty() {
        normalized_path
    } else {
        format!("{}{}", base, normalized_path)
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

pub async fn run_execution_leg_action(
    leg_id: u64,
    payload: &ExecutionLegActionRequest,
) -> Result<ExecutionLegActionResponse, String> {
    let path = format!("/execution/legs/{}/actions", leg_id);
    post_api(&path, payload).await
}

pub async fn send_execution_location_ping(
    leg_id: u64,
    payload: &ExecutionLocationPingRequest,
) -> Result<ExecutionLocationPingResponse, String> {
    let path = format!("/execution/legs/{}/location", leg_id);
    post_api(&path, payload).await
}
