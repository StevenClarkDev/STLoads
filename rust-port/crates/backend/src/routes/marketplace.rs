use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::HeaderMap,
    routing::{get, post},
};
use db::{
    dispatch::{LoadLegScopeRecord, find_load_leg_scope},
    eligibility::{evaluate_carrier_eligibility, find_carrier_profile_id_for_user},
    marketplace::{
        BookNowInput, CarrierCancellationInput, CreateCounterofferInput, CreateTenderInviteInput,
        SubmitCarrierOfferInput, book_now_posting, create_counteroffer, create_message,
        create_tender_invite, find_conversation_by_id, find_offer_by_id, mark_conversation_read,
        request_carrier_cancellation, respond_to_counteroffer, respond_to_tender_invite,
        review_offer, submit_carrier_offer,
    },
};
use domain::{
    auth::UserRole,
    eligibility::EligibilityDecision,
    marketplace::{
        MarketplaceModuleContract, OfferStatus, OfferStatusDescriptor, marketplace_module_contract,
        offer_status_descriptors,
    },
};
use serde::{Deserialize, Serialize};
use shared::{
    ApiResponse, ChatSendMessageRequest, ChatSendMessageResponse, ChatWorkspaceScreen,
    ConversationReadResponse, CreateCounterofferRequest, CreateTenderInviteRequest,
    MarketplaceActionResponse, OfferReviewDecision, OfferReviewRequest, OfferReviewResponse,
    RealtimeEvent, RealtimeEventKind, RealtimeTopic, RespondCounterofferRequest,
    RespondTenderInviteRequest, SubmitOfferRequest,
};
use shared::{BookNowRequest, CarrierCancellationRequest};

use crate::{auth_session, realtime_bus::RoutedRealtimeEvent, screen_data, state::AppState};

#[derive(Debug, Serialize)]
struct MarketplaceOverview {
    contract: MarketplaceModuleContract,
    offer_statuses: usize,
    realtime_channels: usize,
    screen_routes: Vec<&'static str>,
}

#[derive(Debug, Deserialize)]
struct ChatWorkspaceQuery {
    conversation_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct EligibilityQuery {
    carrier_profile_id: Option<i64>,
}

#[derive(Debug, Serialize)]
struct EligibilityResponse {
    success: bool,
    decision: Option<EligibilityDecision>,
    message: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
        .route("/health", get(health))
        .route("/contract", get(contract))
        .route("/offer-statuses", get(offer_statuses))
        .route("/chat-workspace", get(chat_workspace))
        .route(
            "/postings/{posting_id}/eligibility",
            get(posting_eligibility_handler),
        )
        .route("/postings/{posting_id}/offers", post(submit_offer_handler))
        .route(
            "/offers/{offer_id}/counteroffers",
            post(create_counteroffer_handler),
        )
        .route(
            "/counteroffers/{counteroffer_id}/respond",
            post(respond_counteroffer_handler),
        )
        .route(
            "/postings/{posting_id}/tenders",
            post(create_tender_handler),
        )
        .route(
            "/tender-invites/{invite_id}/respond",
            post(respond_tender_handler),
        )
        .route("/postings/{posting_id}/book-now", post(book_now_handler))
        .route(
            "/postings/{posting_id}/cancellations",
            post(request_cancellation_handler),
        )
        .route("/offers/{offer_id}/review", post(review_offer_handler))
        .route(
            "/conversations/{conversation_id}/messages",
            post(send_message_handler),
        )
        .route(
            "/conversations/{conversation_id}/read",
            post(mark_conversation_read_handler),
        )
}

async fn submit_offer_handler(
    State(state): State<AppState>,
    Path(posting_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<SubmitOfferRequest>,
) -> Json<ApiResponse<MarketplaceActionResponse>> {
    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(marketplace_unavailable(&state)));
    };
    let Ok(Some(session)) = auth_session::resolve_session_from_headers(&state, &headers).await
    else {
        return Json(ApiResponse::ok(marketplace_unauthorized("submit an offer")));
    };
    let tenant_id = session_tenant_id(&session);
    let Some(carrier_profile_id) = resolve_carrier_profile(pool, &tenant_id, &session).await else {
        return Json(ApiResponse::ok(marketplace_failure(
            "Missing carrier",
            "No carrier profile is attached to this session.",
        )));
    };
    match submit_carrier_offer(
        pool,
        SubmitCarrierOfferInput {
            tenant_id: &tenant_id,
            posting_id,
            carrier_profile_id,
            carrier_user_id: session.user.id,
            amount: payload.amount,
            currency: payload.currency.as_deref().unwrap_or("USD"),
            message: payload.message.as_deref(),
            idempotency_key: payload.idempotency_key.as_deref(),
            created_by: session.user.id,
        },
    )
    .await
    {
        Ok(record) => Json(ApiResponse::ok(MarketplaceActionResponse {
            success: true,
            id: Some(record.id),
            status_label: "Pending".into(),
            message: "Offer submitted and queued for ATMP sync.".into(),
        })),
        Err(error) => Json(ApiResponse::ok(marketplace_error(
            "Offer submission failed",
            error,
        ))),
    }
}

async fn create_counteroffer_handler(
    State(state): State<AppState>,
    Path(offer_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<CreateCounterofferRequest>,
) -> Json<ApiResponse<MarketplaceActionResponse>> {
    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(marketplace_unavailable(&state)));
    };
    let Ok(Some(session)) = auth_session::resolve_session_from_headers(&state, &headers).await
    else {
        return Json(ApiResponse::ok(marketplace_unauthorized(
            "create a counteroffer",
        )));
    };
    let tenant_id = session_tenant_id(&session);
    match create_counteroffer(
        pool,
        CreateCounterofferInput {
            tenant_id: &tenant_id,
            offer_id,
            from_party_type: payload.from_party_type.as_deref().unwrap_or("shipper"),
            to_party_type: payload.to_party_type.as_deref().unwrap_or("carrier"),
            amount: payload.amount,
            currency: payload.currency.as_deref().unwrap_or("USD"),
            message: payload.message.as_deref(),
            created_by: session.user.id,
        },
    )
    .await
    {
        Ok(record) => Json(ApiResponse::ok(MarketplaceActionResponse {
            success: true,
            id: Some(record.id),
            status_label: record.status,
            message: "Counteroffer created and queued for ATMP sync.".into(),
        })),
        Err(error) => Json(ApiResponse::ok(marketplace_error(
            "Counteroffer failed",
            error,
        ))),
    }
}

async fn respond_counteroffer_handler(
    State(state): State<AppState>,
    Path(counteroffer_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<RespondCounterofferRequest>,
) -> Json<ApiResponse<MarketplaceActionResponse>> {
    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(marketplace_unavailable(&state)));
    };
    let Ok(Some(session)) = auth_session::resolve_session_from_headers(&state, &headers).await
    else {
        return Json(ApiResponse::ok(marketplace_unauthorized(
            "respond to a counteroffer",
        )));
    };
    let tenant_id = session_tenant_id(&session);
    match respond_to_counteroffer(
        pool,
        &tenant_id,
        counteroffer_id,
        &payload.decision,
        payload.note.as_deref(),
        session.user.id,
    )
    .await
    {
        Ok(record) => Json(ApiResponse::ok(MarketplaceActionResponse {
            success: true,
            id: Some(record.id),
            status_label: record.status,
            message: "Counteroffer response saved.".into(),
        })),
        Err(error) => Json(ApiResponse::ok(marketplace_error(
            "Counteroffer response failed",
            error,
        ))),
    }
}

async fn create_tender_handler(
    State(state): State<AppState>,
    Path(posting_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<CreateTenderInviteRequest>,
) -> Json<ApiResponse<MarketplaceActionResponse>> {
    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(marketplace_unavailable(&state)));
    };
    let Ok(Some(session)) = auth_session::resolve_session_from_headers(&state, &headers).await
    else {
        return Json(ApiResponse::ok(marketplace_unauthorized("create a tender")));
    };
    let tenant_id = session_tenant_id(&session);
    match create_tender_invite(
        pool,
        CreateTenderInviteInput {
            tenant_id: &tenant_id,
            posting_id,
            carrier_profile_id: payload.carrier_profile_id,
            tender_type: payload.tender_type.as_deref().unwrap_or("direct"),
            expires_minutes: payload.expires_minutes,
            created_by: session.user.id,
        },
    )
    .await
    {
        Ok(record) => Json(ApiResponse::ok(MarketplaceActionResponse {
            success: true,
            id: Some(record.invite_id),
            status_label: record.invite_status,
            message: "Tender invite created.".into(),
        })),
        Err(error) => Json(ApiResponse::ok(marketplace_error(
            "Tender creation failed",
            error,
        ))),
    }
}

async fn respond_tender_handler(
    State(state): State<AppState>,
    Path(invite_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<RespondTenderInviteRequest>,
) -> Json<ApiResponse<MarketplaceActionResponse>> {
    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(marketplace_unavailable(&state)));
    };
    let Ok(Some(session)) = auth_session::resolve_session_from_headers(&state, &headers).await
    else {
        return Json(ApiResponse::ok(marketplace_unauthorized(
            "respond to a tender",
        )));
    };
    let tenant_id = session_tenant_id(&session);
    match respond_to_tender_invite(
        pool,
        &tenant_id,
        invite_id,
        &payload.decision,
        payload.note.as_deref(),
        session.user.id,
    )
    .await
    {
        Ok(record) => Json(ApiResponse::ok(MarketplaceActionResponse {
            success: true,
            id: Some(record.invite_id),
            status_label: record.invite_status,
            message: "Tender response saved.".into(),
        })),
        Err(error) => Json(ApiResponse::ok(marketplace_error(
            "Tender response failed",
            error,
        ))),
    }
}

async fn book_now_handler(
    State(state): State<AppState>,
    Path(posting_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<BookNowRequest>,
) -> Json<ApiResponse<MarketplaceActionResponse>> {
    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(marketplace_unavailable(&state)));
    };
    let Ok(Some(session)) = auth_session::resolve_session_from_headers(&state, &headers).await
    else {
        return Json(ApiResponse::ok(marketplace_unauthorized(
            "book this posting",
        )));
    };
    let tenant_id = session_tenant_id(&session);
    let carrier_profile_id = match payload.carrier_profile_id {
        Some(value) => value,
        None => match resolve_carrier_profile(pool, &tenant_id, &session).await {
            Some(value) => value,
            None => {
                return Json(ApiResponse::ok(marketplace_failure(
                    "Missing carrier",
                    "No carrier profile is attached to this session.",
                )));
            }
        },
    };
    match book_now_posting(
        pool,
        BookNowInput {
            tenant_id: &tenant_id,
            posting_id,
            carrier_profile_id,
            carrier_user_id: session.user.id,
            offer_id: payload.offer_id,
            tender_id: payload.tender_id,
            amount: payload.amount,
            currency: payload.currency.as_deref().unwrap_or("USD"),
            terms_accepted: payload.terms_accepted,
            idempotency_key: payload.idempotency_key.as_deref(),
            created_by: session.user.id,
        },
    )
    .await
    {
        Ok(record) => Json(ApiResponse::ok(MarketplaceActionResponse {
            success: true,
            id: Some(record.id),
            status_label: record.status,
            message: "Posting booked and queued for ATMP sync.".into(),
        })),
        Err(error) => Json(ApiResponse::ok(marketplace_error("Book-now failed", error))),
    }
}

async fn request_cancellation_handler(
    State(state): State<AppState>,
    Path(posting_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<CarrierCancellationRequest>,
) -> Json<ApiResponse<MarketplaceActionResponse>> {
    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(marketplace_unavailable(&state)));
    };
    let Ok(Some(session)) = auth_session::resolve_session_from_headers(&state, &headers).await
    else {
        return Json(ApiResponse::ok(marketplace_unauthorized(
            "request cancellation",
        )));
    };
    let tenant_id = session_tenant_id(&session);
    match request_carrier_cancellation(
        pool,
        CarrierCancellationInput {
            tenant_id: &tenant_id,
            posting_id,
            booking_award_id: payload.booking_award_id,
            requested_by: session.user.id,
            reason_code: &payload.reason_code,
            reason_detail: payload.reason_detail.as_deref(),
        },
    )
    .await
    {
        Ok(record) => Json(ApiResponse::ok(MarketplaceActionResponse {
            success: true,
            id: Some(record.id),
            status_label: record.status,
            message: "Cancellation request queued for operator review and ATMP sync.".into(),
        })),
        Err(error) => Json(ApiResponse::ok(marketplace_error(
            "Cancellation request failed",
            error,
        ))),
    }
}

async fn posting_eligibility_handler(
    State(state): State<AppState>,
    Path(posting_id): Path<i64>,
    Query(query): Query<EligibilityQuery>,
    headers: HeaderMap,
) -> Json<ApiResponse<EligibilityResponse>> {
    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(EligibilityResponse {
            success: false,
            decision: None,
            message: format!(
                "Eligibility is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        }));
    };

    let Ok(Some(session)) = auth_session::resolve_session_from_headers(&state, &headers).await
    else {
        return Json(ApiResponse::ok(EligibilityResponse {
            success: false,
            decision: None,
            message: "Sign in before checking carrier eligibility.".into(),
        }));
    };

    let tenant_id = session_tenant_id(&session);
    let carrier_profile_id = match query.carrier_profile_id {
        Some(value) => value,
        None => match find_carrier_profile_id_for_user(pool, &tenant_id, session.user.id).await {
            Ok(Some(value)) => value,
            Ok(None) => {
                return Json(ApiResponse::ok(EligibilityResponse {
                    success: false,
                    decision: None,
                    message: "No carrier profile is attached to this session.".into(),
                }));
            }
            Err(error) => {
                return Json(ApiResponse::ok(EligibilityResponse {
                    success: false,
                    decision: None,
                    message: format!("Carrier profile lookup failed: {}", error),
                }));
            }
        },
    };

    let is_admin = session.user.primary_role() == Some(UserRole::Admin);
    if !is_admin && query.carrier_profile_id.is_some() {
        let Ok(Some(owned_profile_id)) =
            find_carrier_profile_id_for_user(pool, &tenant_id, session.user.id).await
        else {
            return Json(ApiResponse::ok(EligibilityResponse {
                success: false,
                decision: None,
                message: "Only admins can check eligibility for another carrier profile.".into(),
            }));
        };
        if owned_profile_id != carrier_profile_id {
            return Json(ApiResponse::ok(EligibilityResponse {
                success: false,
                decision: None,
                message: "Only admins can check eligibility for another carrier profile.".into(),
            }));
        }
    }

    match evaluate_carrier_eligibility(pool, &tenant_id, posting_id, carrier_profile_id).await {
        Ok(decision) => Json(ApiResponse::ok(EligibilityResponse {
            success: true,
            message: decision.result_detail.clone(),
            decision: Some(decision),
        })),
        Err(error) => Json(ApiResponse::ok(EligibilityResponse {
            success: false,
            decision: None,
            message: format!("Eligibility check failed: {}", error),
        })),
    }
}

async fn index() -> Json<ApiResponse<MarketplaceOverview>> {
    let contract = marketplace_module_contract();
    Json(ApiResponse::ok(MarketplaceOverview {
        offer_statuses: offer_status_descriptors().len(),
        realtime_channels: contract.realtime_channels.len(),
        screen_routes: vec!["/marketplace/chat-workspace"],
        contract,
    }))
}

async fn health() -> Json<ApiResponse<&'static str>> {
    Json(ApiResponse::ok("marketplace route group ready"))
}

async fn contract() -> Json<ApiResponse<MarketplaceModuleContract>> {
    Json(ApiResponse::ok(marketplace_module_contract()))
}

async fn offer_statuses() -> Json<ApiResponse<Vec<OfferStatusDescriptor>>> {
    Json(ApiResponse::ok(offer_status_descriptors().to_vec()))
}

async fn chat_workspace(
    State(state): State<AppState>,
    Query(query): Query<ChatWorkspaceQuery>,
    headers: HeaderMap,
) -> Json<ApiResponse<ChatWorkspaceScreen>> {
    let viewer = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten();

    Json(ApiResponse::ok(
        screen_data::chat_workspace_screen(&state, viewer.as_ref(), query.conversation_id).await,
    ))
}

async fn review_offer_handler(
    State(state): State<AppState>,
    Path(offer_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<OfferReviewRequest>,
) -> Json<ApiResponse<OfferReviewResponse>> {
    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(OfferReviewResponse {
            success: false,
            offer_id,
            leg_id: 0,
            status_label: "Unavailable".into(),
            message: format!(
                "Offer review is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        }));
    };

    let Ok(Some(session)) = auth_session::resolve_session_from_headers(&state, &headers).await
    else {
        return Json(ApiResponse::ok(OfferReviewResponse {
            success: false,
            offer_id,
            leg_id: 0,
            status_label: "Unauthorized".into(),
            message: "Sign in before reviewing marketplace offers.".into(),
        }));
    };

    if session.user.primary_role() == Some(UserRole::Carrier) {
        return Json(ApiResponse::ok(OfferReviewResponse {
            success: false,
            offer_id,
            leg_id: 0,
            status_label: "Forbidden".into(),
            message: "Carrier accounts cannot accept or decline offers from the Rust review route."
                .into(),
        }));
    }

    let Ok(Some(existing_offer)) = find_offer_by_id(pool, offer_id).await else {
        return Json(ApiResponse::ok(OfferReviewResponse {
            success: false,
            offer_id,
            leg_id: 0,
            status_label: "Missing".into(),
            message: "The requested offer was not found.".into(),
        }));
    };

    let is_admin = session.user.primary_role() == Some(UserRole::Admin);
    let load_scope = find_load_leg_scope(pool, existing_offer.load_leg_id)
        .await
        .ok()
        .flatten();
    let is_load_owner = load_scope
        .as_ref()
        .and_then(|scope| scope.load_owner_user_id)
        == Some(session.user.id);

    if !is_admin && !is_load_owner {
        return Json(ApiResponse::ok(OfferReviewResponse {
            success: false,
            offer_id,
            leg_id: existing_offer.load_leg_id,
            status_label: "Forbidden".into(),
            message: "Only the authenticated load owner or an admin can review this offer.".into(),
        }));
    }

    if existing_offer.status() != Some(OfferStatus::Pending) {
        return Json(ApiResponse::ok(OfferReviewResponse {
            success: false,
            offer_id,
            leg_id: existing_offer.load_leg_id,
            status_label: "Locked".into(),
            message:
                "Only pending offers can be accepted or declined from this Rust marketplace route."
                    .into(),
        }));
    }

    match review_offer(
        pool,
        offer_id,
        matches!(payload.decision, OfferReviewDecision::Accept),
        Some(session.user.id),
    )
    .await
    {
        Ok(Some(offer)) => {
            let (status_label, action_label) = match payload.decision {
                OfferReviewDecision::Accept => ("Accepted", "accepted"),
                OfferReviewDecision::Decline => ("Declined", "declined"),
            };

            let mut target_user_ids = collect_scope_user_ids(load_scope.as_ref());
            if offer.carrier_id > 0 {
                target_user_ids.push(offer.carrier_id as u64);
            }
            if let Some(conversation_id) = offer.conversation_id {
                if let Ok(Some(conversation)) = find_conversation_by_id(pool, conversation_id).await
                {
                    target_user_ids.extend(conversation_participant_user_ids(&conversation));
                }
            }
            target_user_ids.push(session.user.id.max(0) as u64);
            target_user_ids.sort_unstable();
            target_user_ids.dedup();

            state.publish_realtime(
                RoutedRealtimeEvent::new(RealtimeEvent {
                    kind: RealtimeEventKind::OfferReviewed,
                    leg_id: Some(offer.load_leg_id.max(0) as u64),
                    conversation_id: offer.conversation_id.map(|value| value.max(0) as u64),
                    offer_id: Some(offer_id.max(0) as u64),
                    message_id: None,
                    actor_user_id: Some(session.user.id.max(0) as u64),
                    subject_user_id: None,
                    presence_state: None,
                    last_read_message_id: None,
                    summary: format!(
                        "{} {} offer #{}.",
                        session.user.name, action_label, offer_id
                    ),
                })
                .for_user_ids(target_user_ids)
                .with_topics([
                    RealtimeTopic::Conversation.as_key(),
                    RealtimeTopic::LoadBoard.as_key(),
                ]),
            );

            Json(ApiResponse::ok(OfferReviewResponse {
                success: true,
                offer_id,
                leg_id: offer.load_leg_id,
                status_label: status_label.into(),
                message: format!(
                    "Offer {} from the authenticated Rust marketplace route; scoped participants will refresh through realtime events.",
                    action_label
                ),
            }))
        }
        Ok(None) => Json(ApiResponse::ok(OfferReviewResponse {
            success: false,
            offer_id,
            leg_id: 0,
            status_label: "Missing".into(),
            message: "The requested offer was not found.".into(),
        })),
        Err(error) => Json(ApiResponse::ok(OfferReviewResponse {
            success: false,
            offer_id,
            leg_id: 0,
            status_label: "Error".into(),
            message: format!("Offer review failed: {}", error),
        })),
    }
}

async fn send_message_handler(
    State(state): State<AppState>,
    Path(conversation_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<ChatSendMessageRequest>,
) -> Json<ApiResponse<ChatSendMessageResponse>> {
    let trimmed_body = payload.body.trim().to_string();
    if trimmed_body.is_empty() {
        return Json(ApiResponse::ok(ChatSendMessageResponse {
            success: false,
            conversation_id,
            message_id: 0,
            message: "Message body cannot be empty.".into(),
        }));
    }

    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(ChatSendMessageResponse {
            success: false,
            conversation_id,
            message_id: 0,
            message: format!(
                "Message send is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        }));
    };

    let Ok(Some(session)) = auth_session::resolve_session_from_headers(&state, &headers).await
    else {
        return Json(ApiResponse::ok(ChatSendMessageResponse {
            success: false,
            conversation_id,
            message_id: 0,
            message: "Sign in before sending chat messages through the Rust marketplace route."
                .into(),
        }));
    };

    let Ok(Some(conversation)) = find_conversation_by_id(pool, conversation_id).await else {
        return Json(ApiResponse::ok(ChatSendMessageResponse {
            success: false,
            conversation_id,
            message_id: 0,
            message: "The requested conversation was not found.".into(),
        }));
    };

    if session.user.id != conversation.shipper_id && session.user.id != conversation.carrier_id {
        return Json(ApiResponse::ok(ChatSendMessageResponse {
            success: false,
            conversation_id,
            message_id: 0,
            message:
                "This authenticated account is not a participant in the selected conversation."
                    .into(),
        }));
    }

    match create_message(pool, conversation_id, session.user.id, &trimmed_body).await {
        Ok(Some(message)) => {
            let mut target_user_ids = conversation_participant_user_ids(&conversation);
            target_user_ids.sort_unstable();
            target_user_ids.dedup();

            state.publish_realtime(
                RoutedRealtimeEvent::new(RealtimeEvent {
                    kind: RealtimeEventKind::MessageSent,
                    leg_id: Some(conversation.load_leg_id.max(0) as u64),
                    conversation_id: Some(conversation_id.max(0) as u64),
                    offer_id: None,
                    message_id: Some(message.id.max(0) as u64),
                    actor_user_id: Some(session.user.id.max(0) as u64),
                    subject_user_id: Some(session.user.id.max(0) as u64),
                    presence_state: None,
                    last_read_message_id: Some(message.id.max(0) as u64),
                    summary: format!("{} sent a chat message.", session.user.name),
                })
                .for_user_ids(target_user_ids)
                .with_topics([RealtimeTopic::Conversation.as_key()]),
            );

            Json(ApiResponse::ok(ChatSendMessageResponse {
                success: true,
                conversation_id,
                message_id: message.id,
                message: "Message stored through the authenticated Rust marketplace route.".into(),
            }))
        }
        Ok(None) => Json(ApiResponse::ok(ChatSendMessageResponse {
            success: false,
            conversation_id,
            message_id: 0,
            message: "The requested conversation was not found.".into(),
        })),
        Err(error) => Json(ApiResponse::ok(ChatSendMessageResponse {
            success: false,
            conversation_id,
            message_id: 0,
            message: format!("Message send failed: {}", error),
        })),
    }
}

async fn mark_conversation_read_handler(
    State(state): State<AppState>,
    Path(conversation_id): Path<i64>,
    headers: HeaderMap,
) -> Json<ApiResponse<ConversationReadResponse>> {
    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(ConversationReadResponse {
            success: false,
            conversation_id,
            last_read_message_id: None,
            message: format!(
                "Read receipts are unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        }));
    };

    let Ok(Some(session)) = auth_session::resolve_session_from_headers(&state, &headers).await
    else {
        return Json(ApiResponse::ok(ConversationReadResponse {
            success: false,
            conversation_id,
            last_read_message_id: None,
            message: "Sign in before marking a conversation as read.".into(),
        }));
    };

    let Ok(Some(conversation)) = find_conversation_by_id(pool, conversation_id).await else {
        return Json(ApiResponse::ok(ConversationReadResponse {
            success: false,
            conversation_id,
            last_read_message_id: None,
            message: "The requested conversation was not found.".into(),
        }));
    };

    if session.user.id != conversation.shipper_id && session.user.id != conversation.carrier_id {
        return Json(ApiResponse::ok(ConversationReadResponse {
            success: false,
            conversation_id,
            last_read_message_id: None,
            message:
                "This authenticated account is not a participant in the selected conversation."
                    .into(),
        }));
    }

    match mark_conversation_read(pool, conversation_id, session.user.id).await {
        Ok(Some(read_state)) => {
            let target_user_ids = conversation_participant_user_ids(&conversation);
            let last_read_message_id = read_state.last_read_message_id;

            state.publish_realtime(
                RoutedRealtimeEvent::new(RealtimeEvent {
                    kind: RealtimeEventKind::ConversationRead,
                    leg_id: Some(conversation.load_leg_id.max(0) as u64),
                    conversation_id: Some(conversation.id.max(0) as u64),
                    offer_id: None,
                    message_id: last_read_message_id.map(|value| value.max(0) as u64),
                    actor_user_id: Some(session.user.id.max(0) as u64),
                    subject_user_id: Some(session.user.id.max(0) as u64),
                    presence_state: None,
                    last_read_message_id: last_read_message_id.map(|value| value.max(0) as u64),
                    summary: format!("{} read the active conversation.", session.user.name),
                })
                .for_user_ids(target_user_ids)
                .with_topics([RealtimeTopic::Conversation.as_key()]),
            );

            Json(ApiResponse::ok(ConversationReadResponse {
                success: true,
                conversation_id,
                last_read_message_id,
                message: "Conversation marked as read for the authenticated Rust session.".into(),
            }))
        }
        Ok(None) => Json(ApiResponse::ok(ConversationReadResponse {
            success: false,
            conversation_id,
            last_read_message_id: None,
            message: "No read receipt state could be stored for this conversation.".into(),
        })),
        Err(error) => Json(ApiResponse::ok(ConversationReadResponse {
            success: false,
            conversation_id,
            last_read_message_id: None,
            message: format!("Read receipt update failed: {}", error),
        })),
    }
}

fn collect_scope_user_ids(scope: Option<&LoadLegScopeRecord>) -> Vec<u64> {
    let mut user_ids = Vec::new();

    if let Some(scope) = scope {
        if let Some(owner_id) = scope.load_owner_user_id {
            if owner_id > 0 {
                user_ids.push(owner_id as u64);
            }
        }

        if let Some(booked_carrier_id) = scope.booked_carrier_id {
            if booked_carrier_id > 0 {
                user_ids.push(booked_carrier_id as u64);
            }
        }
    }

    user_ids
}

fn session_tenant_id(session: &auth_session::ResolvedSession) -> String {
    session
        .session
        .tenant_scope
        .as_ref()
        .map(|scope| scope.tenant_id.clone())
        .unwrap_or_else(|| "legacy".into())
}

async fn resolve_carrier_profile(
    pool: &db::DbPool,
    tenant_id: &str,
    session: &auth_session::ResolvedSession,
) -> Option<i64> {
    find_carrier_profile_id_for_user(pool, tenant_id, session.user.id)
        .await
        .ok()
        .flatten()
}

fn marketplace_unavailable(state: &AppState) -> MarketplaceActionResponse {
    marketplace_failure(
        "Unavailable",
        &format!(
            "Marketplace action is unavailable because the database is {} on {}.",
            state.database_state(),
            state.config.deployment_target
        ),
    )
}

fn marketplace_unauthorized(action: &str) -> MarketplaceActionResponse {
    marketplace_failure("Unauthorized", &format!("Sign in before you {action}."))
}

fn marketplace_error(label: &str, error: sqlx::Error) -> MarketplaceActionResponse {
    marketplace_failure(label, &format!("{}: {}", label, error))
}

fn marketplace_failure(label: &str, message: &str) -> MarketplaceActionResponse {
    MarketplaceActionResponse {
        success: false,
        id: None,
        status_label: label.into(),
        message: message.into(),
    }
}

fn conversation_participant_user_ids(
    conversation: &db::marketplace::ConversationRecord,
) -> Vec<u64> {
    let mut user_ids = Vec::new();

    if conversation.shipper_id > 0 {
        user_ids.push(conversation.shipper_id as u64);
    }

    if conversation.carrier_id > 0 {
        user_ids.push(conversation.carrier_id as u64);
    }

    user_ids
}
