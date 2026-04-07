use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use db::{
    dispatch::{find_load_leg_by_id, find_load_leg_scope},
    payments::{
        EscrowTransitionParams, apply_escrow_transition, find_escrow_by_payment_intent_id,
        find_escrow_for_leg, update_user_connect_state,
    },
};
use domain::{
    auth::UserRole,
    payments::{
        EscrowStatus, EscrowStatusDescriptor, PaymentsModuleContract, StripeWebhookEventDescriptor,
        escrow_status_descriptors, payments_module_contract, stripe_webhook_events,
    },
};
use serde::Serialize;
use shared::{
    ApiResponse, EscrowFundRequest, EscrowHoldRequest, EscrowLifecycleResponse,
    EscrowReleaseRequest, RealtimeEvent, RealtimeEventKind, RealtimeTopic, StripeWebhookRequest,
    StripeWebhookResponse,
};

use crate::{
    auth_session, auth_session::ResolvedSession, realtime_bus::RoutedRealtimeEvent, state::AppState,
};

#[derive(Debug, Serialize)]
struct PaymentsOverview {
    contract: PaymentsModuleContract,
    escrow_statuses: usize,
    webhook_events: usize,
}

pub fn router() -> Router<crate::state::AppState> {
    Router::new()
        .route("/", get(index))
        .route("/health", get(health))
        .route("/contract", get(contract))
        .route("/escrow-statuses", get(escrow_statuses))
        .route("/webhook-events", get(webhook_events))
        .route("/legs/{leg_id}/fund", post(fund_leg_escrow))
        .route("/legs/{leg_id}/hold", post(hold_leg_escrow))
        .route("/legs/{leg_id}/release", post(release_leg_escrow))
        .route("/webhooks/stripe", post(stripe_webhook))
}

async fn index() -> Json<ApiResponse<PaymentsOverview>> {
    let contract = payments_module_contract();
    Json(ApiResponse::ok(PaymentsOverview {
        escrow_statuses: escrow_status_descriptors().len(),
        webhook_events: contract.webhook_events.len(),
        contract,
    }))
}

async fn health() -> Json<ApiResponse<&'static str>> {
    Json(ApiResponse::ok("payments route group ready"))
}

async fn contract() -> Json<ApiResponse<PaymentsModuleContract>> {
    Json(ApiResponse::ok(payments_module_contract()))
}

async fn escrow_statuses() -> Json<ApiResponse<Vec<EscrowStatusDescriptor>>> {
    Json(ApiResponse::ok(escrow_status_descriptors().to_vec()))
}

async fn webhook_events() -> Json<ApiResponse<Vec<StripeWebhookEventDescriptor>>> {
    Json(ApiResponse::ok(stripe_webhook_events().to_vec()))
}

async fn fund_leg_escrow(
    State(state): State<AppState>,
    Path(leg_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<EscrowFundRequest>,
) -> Json<ApiResponse<EscrowLifecycleResponse>> {
    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(unavailable_payment_response(
            &state, leg_id, "Funding",
        )));
    };

    let Ok(Some(session)) = resolve_payments_session(&state, &headers).await else {
        return Json(ApiResponse::ok(EscrowLifecycleResponse {
            success: false,
            leg_id,
            escrow_id: None,
            status_label: "Unauthorized".into(),
            message: "Sign in with payments access before funding escrow from the Rust route."
                .into(),
        }));
    };

    let Some(scope) = find_load_leg_scope(pool, leg_id).await.ok().flatten() else {
        return Json(ApiResponse::ok(missing_leg_response(leg_id)));
    };

    if !can_manage_leg_payments(&session, scope.load_owner_user_id) {
        return Json(ApiResponse::ok(forbidden_payment_response(
            leg_id,
            "Only the load owner or an admin can fund escrow for this leg.",
        )));
    }

    let Some(leg) = find_load_leg_by_id(pool, leg_id).await.ok().flatten() else {
        return Json(ApiResponse::ok(missing_leg_response(leg_id)));
    };

    let Some(payee_user_id) = leg.booked_carrier_id else {
        return Json(ApiResponse::ok(EscrowLifecycleResponse {
            success: false,
            leg_id,
            escrow_id: None,
            status_label: "Locked".into(),
            message: "Escrow cannot be funded until the leg has a booked carrier.".into(),
        }));
    };

    let Some(payer_user_id) = scope.load_owner_user_id else {
        return Json(ApiResponse::ok(EscrowLifecycleResponse {
            success: false,
            leg_id,
            escrow_id: None,
            status_label: "Unavailable".into(),
            message: "The load owner could not be resolved for this leg.".into(),
        }));
    };

    let Some(amount) = payload
        .amount_cents
        .or_else(|| leg.booked_amount.or(leg.price).map(currency_to_cents))
    else {
        return Json(ApiResponse::ok(EscrowLifecycleResponse {
            success: false,
            leg_id,
            escrow_id: None,
            status_label: "Missing Amount".into(),
            message: "Escrow funding requires either an explicit amount or a priced leg.".into(),
        }));
    };

    match apply_escrow_transition(
        pool,
        EscrowTransitionParams {
            leg_id,
            payer_user_id,
            payee_user_id,
            amount,
            platform_fee: payload.platform_fee_cents.unwrap_or(0),
            currency: payload.currency.as_deref().unwrap_or("USD"),
            status: EscrowStatus::Funded,
            transfer_group: payload.transfer_group.as_deref(),
            payment_intent_id: payload.payment_intent_id.as_deref(),
            charge_id: payload.charge_id.as_deref(),
            transfer_id: None,
            actor_user_id: Some(session.user.id),
            note: payload.note.as_deref(),
        },
    )
    .await
    {
        Ok(Some(escrow)) => {
            publish_payments_event(
                &state,
                leg_id,
                Some(session.user.id.max(0) as u64),
                Some(payee_user_id.max(0) as u64),
                vec![payer_user_id.max(0) as u64, payee_user_id.max(0) as u64],
                format!(
                    "{} funded escrow for load leg #{}.",
                    session.user.name, leg_id
                ),
            );

            Json(ApiResponse::ok(EscrowLifecycleResponse {
                success: true,
                leg_id,
                escrow_id: Some(escrow.id),
                status_label: "Funded".into(),
                message: "Escrow funded through the Rust payments route; admin and load-board views will refresh through targeted realtime events.".into(),
            }))
        }
        Ok(None) => Json(ApiResponse::ok(missing_leg_response(leg_id))),
        Err(error) => Json(ApiResponse::ok(EscrowLifecycleResponse {
            success: false,
            leg_id,
            escrow_id: None,
            status_label: "Error".into(),
            message: format!("Funding failed: {}", error),
        })),
    }
}

async fn hold_leg_escrow(
    State(state): State<AppState>,
    Path(leg_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<EscrowHoldRequest>,
) -> Json<ApiResponse<EscrowLifecycleResponse>> {
    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(unavailable_payment_response(
            &state, leg_id, "Hold",
        )));
    };

    let Ok(Some(session)) = resolve_payments_session(&state, &headers).await else {
        return Json(ApiResponse::ok(EscrowLifecycleResponse {
            success: false,
            leg_id,
            escrow_id: None,
            status_label: "Unauthorized".into(),
            message: "Sign in with payments access before holding escrow from the Rust route."
                .into(),
        }));
    };

    let Some(scope) = find_load_leg_scope(pool, leg_id).await.ok().flatten() else {
        return Json(ApiResponse::ok(missing_leg_response(leg_id)));
    };

    if !can_manage_leg_payments(&session, scope.load_owner_user_id) {
        return Json(ApiResponse::ok(forbidden_payment_response(
            leg_id,
            "Only the load owner or an admin can place escrow on hold.",
        )));
    }

    let Some(leg) = find_load_leg_by_id(pool, leg_id).await.ok().flatten() else {
        return Json(ApiResponse::ok(missing_leg_response(leg_id)));
    };
    let Some(payee_user_id) = leg.booked_carrier_id else {
        return Json(ApiResponse::ok(forbidden_payment_response(
            leg_id,
            "Escrow hold requires a booked carrier on the selected leg.",
        )));
    };
    let Some(payer_user_id) = scope.load_owner_user_id else {
        return Json(ApiResponse::ok(forbidden_payment_response(
            leg_id,
            "The load owner could not be resolved for this leg.",
        )));
    };

    let Some(amount) = find_escrow_for_leg(pool, leg_id)
        .await
        .ok()
        .flatten()
        .map(|escrow| escrow.amount)
        .or_else(|| leg.booked_amount.or(leg.price).map(currency_to_cents))
    else {
        return Json(ApiResponse::ok(EscrowLifecycleResponse {
            success: false,
            leg_id,
            escrow_id: None,
            status_label: "Missing Amount".into(),
            message: "Escrow hold requires an existing funded amount or a priced leg.".into(),
        }));
    };

    match apply_escrow_transition(
        pool,
        EscrowTransitionParams {
            leg_id,
            payer_user_id,
            payee_user_id,
            amount,
            platform_fee: 0,
            currency: "USD",
            status: EscrowStatus::OnHold,
            transfer_group: None,
            payment_intent_id: None,
            charge_id: None,
            transfer_id: None,
            actor_user_id: Some(session.user.id),
            note: payload.note.as_deref(),
        },
    )
    .await
    {
        Ok(Some(escrow)) => {
            publish_payments_event(
                &state,
                leg_id,
                Some(session.user.id.max(0) as u64),
                Some(payee_user_id.max(0) as u64),
                vec![payer_user_id.max(0) as u64, payee_user_id.max(0) as u64],
                format!(
                    "{} placed escrow on hold for load leg #{}.",
                    session.user.name, leg_id
                ),
            );

            Json(ApiResponse::ok(EscrowLifecycleResponse {
                success: true,
                leg_id,
                escrow_id: Some(escrow.id),
                status_label: "On Hold".into(),
                message: "Escrow placed on hold through the Rust payments route.".into(),
            }))
        }
        Ok(None) => Json(ApiResponse::ok(missing_leg_response(leg_id))),
        Err(error) => Json(ApiResponse::ok(EscrowLifecycleResponse {
            success: false,
            leg_id,
            escrow_id: None,
            status_label: "Error".into(),
            message: format!("Escrow hold failed: {}", error),
        })),
    }
}

async fn release_leg_escrow(
    State(state): State<AppState>,
    Path(leg_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<EscrowReleaseRequest>,
) -> Json<ApiResponse<EscrowLifecycleResponse>> {
    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(unavailable_payment_response(
            &state, leg_id, "Release",
        )));
    };

    let Ok(Some(session)) = resolve_payments_session(&state, &headers).await else {
        return Json(ApiResponse::ok(EscrowLifecycleResponse {
            success: false,
            leg_id,
            escrow_id: None,
            status_label: "Unauthorized".into(),
            message: "Sign in with payments access before releasing escrow from the Rust route."
                .into(),
        }));
    };

    let Some(scope) = find_load_leg_scope(pool, leg_id).await.ok().flatten() else {
        return Json(ApiResponse::ok(missing_leg_response(leg_id)));
    };

    if !can_manage_leg_payments(&session, scope.load_owner_user_id) {
        return Json(ApiResponse::ok(forbidden_payment_response(
            leg_id,
            "Only the load owner or an admin can release escrow for this leg.",
        )));
    }

    let Some(leg) = find_load_leg_by_id(pool, leg_id).await.ok().flatten() else {
        return Json(ApiResponse::ok(missing_leg_response(leg_id)));
    };
    let Some(existing_escrow) = find_escrow_for_leg(pool, leg_id).await.ok().flatten() else {
        return Json(ApiResponse::ok(EscrowLifecycleResponse {
            success: false,
            leg_id,
            escrow_id: None,
            status_label: "Missing Escrow".into(),
            message: "Escrow must exist before it can be released.".into(),
        }));
    };
    let Some(payee_user_id) = leg.booked_carrier_id else {
        return Json(ApiResponse::ok(forbidden_payment_response(
            leg_id,
            "Escrow release requires a booked carrier on the selected leg.",
        )));
    };
    let Some(payer_user_id) = scope.load_owner_user_id else {
        return Json(ApiResponse::ok(forbidden_payment_response(
            leg_id,
            "The load owner could not be resolved for this leg.",
        )));
    };

    match apply_escrow_transition(
        pool,
        EscrowTransitionParams {
            leg_id,
            payer_user_id,
            payee_user_id,
            amount: existing_escrow.amount,
            platform_fee: existing_escrow.platform_fee,
            currency: existing_escrow.currency.as_str(),
            status: EscrowStatus::Released,
            transfer_group: existing_escrow.transfer_group.as_deref(),
            payment_intent_id: existing_escrow.payment_intent_id.as_deref(),
            charge_id: existing_escrow.charge_id.as_deref(),
            transfer_id: payload.transfer_id.as_deref(),
            actor_user_id: Some(session.user.id),
            note: payload.note.as_deref(),
        },
    )
    .await
    {
        Ok(Some(escrow)) => {
            publish_payments_event(
                &state,
                leg_id,
                Some(session.user.id.max(0) as u64),
                Some(payee_user_id.max(0) as u64),
                vec![payer_user_id.max(0) as u64, payee_user_id.max(0) as u64],
                format!(
                    "{} released escrow for load leg #{}.",
                    session.user.name, leg_id
                ),
            );

            Json(ApiResponse::ok(EscrowLifecycleResponse {
                success: true,
                leg_id,
                escrow_id: Some(escrow.id),
                status_label: "Released".into(),
                message: "Escrow released through the Rust payments route; payout readiness will refresh on admin and load-board views.".into(),
            }))
        }
        Ok(None) => Json(ApiResponse::ok(missing_leg_response(leg_id))),
        Err(error) => Json(ApiResponse::ok(EscrowLifecycleResponse {
            success: false,
            leg_id,
            escrow_id: None,
            status_label: "Error".into(),
            message: format!("Escrow release failed: {}", error),
        })),
    }
}

async fn stripe_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<StripeWebhookRequest>,
) -> Result<Json<ApiResponse<StripeWebhookResponse>>, StatusCode> {
    let actor_session = authorize_payments_webhook(&state, &headers).await?;
    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(StripeWebhookResponse {
            acknowledged: false,
            event_type: payload.event_type,
            leg_id: payload.leg_id,
            user_id: None,
            message: format!(
                "Stripe webhook handling is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        })));
    };

    match payload.event_type.as_str() {
        "payment_intent.succeeded" => {
            let Some((leg_id, payer_user_id, payee_user_id, amount, platform_fee, currency)) =
                resolve_webhook_escrow_context(pool, &payload).await
            else {
                return Ok(Json(ApiResponse::ok(StripeWebhookResponse {
                    acknowledged: false,
                    event_type: payload.event_type,
                    leg_id: payload.leg_id,
                    user_id: None,
                    message: "The webhook could not resolve an escrow context for funding.".into(),
                })));
            };

            let updated = apply_escrow_transition(
                pool,
                EscrowTransitionParams {
                    leg_id,
                    payer_user_id,
                    payee_user_id,
                    amount,
                    platform_fee,
                    currency: currency.as_str(),
                    status: EscrowStatus::Funded,
                    transfer_group: payload.transfer_group.as_deref(),
                    payment_intent_id: payload.payment_intent_id.as_deref(),
                    charge_id: payload.charge_id.as_deref(),
                    transfer_id: None,
                    actor_user_id: actor_session.as_ref().map(|session| session.user.id),
                    note: payload.note.as_deref(),
                },
            )
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            if updated.is_some() {
                publish_payments_event(
                    &state,
                    leg_id,
                    actor_session
                        .as_ref()
                        .map(|session| session.user.id.max(0) as u64),
                    Some(payee_user_id.max(0) as u64),
                    vec![payer_user_id.max(0) as u64, payee_user_id.max(0) as u64],
                    format!("Stripe webhook funded escrow for load leg #{}.", leg_id),
                );
            }

            Ok(Json(ApiResponse::ok(StripeWebhookResponse {
                acknowledged: updated.is_some(),
                event_type: payload.event_type,
                leg_id: Some(leg_id),
                user_id: None,
                message: "Stripe funding webhook applied through the Rust payments route.".into(),
            })))
        }
        "payment_intent.payment_failed" => {
            let Some((leg_id, payer_user_id, payee_user_id, amount, platform_fee, currency)) =
                resolve_webhook_escrow_context(pool, &payload).await
            else {
                return Ok(Json(ApiResponse::ok(StripeWebhookResponse {
                    acknowledged: false,
                    event_type: payload.event_type,
                    leg_id: payload.leg_id,
                    user_id: None,
                    message:
                        "The webhook could not resolve an escrow context for the failed payment."
                            .into(),
                })));
            };

            let updated = apply_escrow_transition(
                pool,
                EscrowTransitionParams {
                    leg_id,
                    payer_user_id,
                    payee_user_id,
                    amount,
                    platform_fee,
                    currency: currency.as_str(),
                    status: EscrowStatus::Failed,
                    transfer_group: payload.transfer_group.as_deref(),
                    payment_intent_id: payload.payment_intent_id.as_deref(),
                    charge_id: payload.charge_id.as_deref(),
                    transfer_id: None,
                    actor_user_id: actor_session.as_ref().map(|session| session.user.id),
                    note: payload.note.as_deref(),
                },
            )
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            if updated.is_some() {
                publish_payments_event(
                    &state,
                    leg_id,
                    actor_session
                        .as_ref()
                        .map(|session| session.user.id.max(0) as u64),
                    Some(payee_user_id.max(0) as u64),
                    vec![payer_user_id.max(0) as u64, payee_user_id.max(0) as u64],
                    format!(
                        "Stripe webhook marked escrow as failed for load leg #{}.",
                        leg_id
                    ),
                );
            }

            Ok(Json(ApiResponse::ok(StripeWebhookResponse {
                acknowledged: updated.is_some(),
                event_type: payload.event_type,
                leg_id: Some(leg_id),
                user_id: None,
                message: "Stripe failure webhook applied through the Rust payments route.".into(),
            })))
        }
        "account.updated" => {
            let Some(stripe_account_id) = payload.stripe_account_id.as_deref() else {
                return Ok(Json(ApiResponse::ok(StripeWebhookResponse {
                    acknowledged: false,
                    event_type: payload.event_type,
                    leg_id: None,
                    user_id: None,
                    message: "Stripe account.updated requires stripe_account_id.".into(),
                })));
            };

            let updated_user = update_user_connect_state(
                pool,
                stripe_account_id,
                payload.payouts_enabled.unwrap_or(false),
                payload.kyc_status.as_deref(),
            )
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            if let Some(user) = updated_user.as_ref() {
                state.publish_realtime(
                    RoutedRealtimeEvent::new(RealtimeEvent {
                        kind: RealtimeEventKind::PaymentsOperationsUpdated,
                        leg_id: None,
                        conversation_id: None,
                        offer_id: None,
                        message_id: None,
                        actor_user_id: actor_session
                            .as_ref()
                            .map(|session| session.user.id.max(0) as u64),
                        subject_user_id: Some(user.id.max(0) as u64),
                        presence_state: None,
                        last_read_message_id: None,
                        summary: format!(
                            "Stripe account update synced payout readiness for user #{}.",
                            user.id
                        ),
                    })
                    .for_user_ids([user.id.max(0) as u64])
                    .for_permission_keys(["manage_payments"])
                    .with_topics([
                        RealtimeTopic::AdminPayments.as_key(),
                        RealtimeTopic::AdminDashboard.as_key(),
                    ]),
                );
            }

            Ok(Json(ApiResponse::ok(StripeWebhookResponse {
                acknowledged: updated_user.is_some(),
                event_type: payload.event_type,
                leg_id: None,
                user_id: updated_user.as_ref().map(|user| user.id),
                message: "Stripe account update synced through the Rust payments route.".into(),
            })))
        }
        _ => Ok(Json(ApiResponse::ok(StripeWebhookResponse {
            acknowledged: false,
            event_type: payload.event_type,
            leg_id: payload.leg_id,
            user_id: None,
            message: "Unsupported Stripe webhook event for the current Rust payments slice.".into(),
        }))),
    }
}

async fn resolve_payments_session(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<Option<ResolvedSession>, String> {
    let session = auth_session::resolve_session_from_headers(state, headers).await?;
    Ok(session.filter(|session| {
        session.session.permissions.iter().any(|permission| {
            permission == "manage_payments" || permission == "access_admin_portal"
        })
    }))
}

async fn authorize_payments_webhook(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<Option<ResolvedSession>, StatusCode> {
    if let Some(expected_secret) = state.config.stripe_webhook_shared_secret.as_deref() {
        let supplied_secret = headers
            .get("x-stripe-webhook-secret")
            .and_then(|value| value.to_str().ok())
            .map(str::trim)
            .filter(|value| !value.is_empty());

        if supplied_secret == Some(expected_secret) {
            return Ok(None);
        }
    }

    match resolve_payments_session(state, headers).await {
        Ok(Some(session)) => Ok(Some(session)),
        Ok(None) => Err(StatusCode::UNAUTHORIZED),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn resolve_webhook_escrow_context(
    pool: &db::DbPool,
    payload: &StripeWebhookRequest,
) -> Option<(i64, i64, i64, i64, i64, String)> {
    if let Some(payment_intent_id) = payload.payment_intent_id.as_deref() {
        if let Some(existing_escrow) = find_escrow_by_payment_intent_id(pool, payment_intent_id)
            .await
            .ok()
            .flatten()
        {
            return Some((
                existing_escrow.leg_id,
                existing_escrow.payer_user_id,
                existing_escrow.payee_user_id,
                payload.amount_cents.unwrap_or(existing_escrow.amount),
                payload
                    .platform_fee_cents
                    .unwrap_or(existing_escrow.platform_fee),
                payload
                    .currency
                    .clone()
                    .unwrap_or_else(|| existing_escrow.currency),
            ));
        }
    }

    let leg_id = payload.leg_id?;
    let scope = find_load_leg_scope(pool, leg_id).await.ok().flatten()?;
    let leg = find_load_leg_by_id(pool, leg_id).await.ok().flatten()?;
    let payee_user_id = leg.booked_carrier_id?;
    let payer_user_id = scope.load_owner_user_id?;
    let amount = payload
        .amount_cents
        .or_else(|| leg.booked_amount.or(leg.price).map(currency_to_cents))?;

    Some((
        leg_id,
        payer_user_id,
        payee_user_id,
        amount,
        payload.platform_fee_cents.unwrap_or(0),
        payload.currency.clone().unwrap_or_else(|| "USD".into()),
    ))
}

fn can_manage_leg_payments(session: &ResolvedSession, load_owner_user_id: Option<i64>) -> bool {
    session.user.primary_role() == Some(UserRole::Admin)
        || load_owner_user_id == Some(session.user.id)
}

fn publish_payments_event(
    state: &AppState,
    leg_id: i64,
    actor_user_id: Option<u64>,
    subject_user_id: Option<u64>,
    target_user_ids: Vec<u64>,
    summary: String,
) {
    state.publish_realtime(
        RoutedRealtimeEvent::new(RealtimeEvent {
            kind: RealtimeEventKind::PaymentsOperationsUpdated,
            leg_id: Some(leg_id.max(0) as u64),
            conversation_id: None,
            offer_id: None,
            message_id: None,
            actor_user_id,
            subject_user_id,
            presence_state: None,
            last_read_message_id: None,
            summary,
        })
        .for_user_ids(target_user_ids)
        .for_permission_keys(["manage_payments"])
        .with_topics([
            RealtimeTopic::AdminPayments.as_key(),
            RealtimeTopic::AdminDashboard.as_key(),
            RealtimeTopic::LoadBoard.as_key(),
        ]),
    );
}

fn currency_to_cents(value: f64) -> i64 {
    (value * 100.0).round() as i64
}

fn unavailable_payment_response(
    state: &AppState,
    leg_id: i64,
    action_label: &str,
) -> EscrowLifecycleResponse {
    EscrowLifecycleResponse {
        success: false,
        leg_id,
        escrow_id: None,
        status_label: "Unavailable".into(),
        message: format!(
            "{} action is unavailable because the database is {} on {}.",
            action_label,
            state.database_state(),
            state.config.deployment_target
        ),
    }
}

fn missing_leg_response(leg_id: i64) -> EscrowLifecycleResponse {
    EscrowLifecycleResponse {
        success: false,
        leg_id,
        escrow_id: None,
        status_label: "Missing".into(),
        message: "The requested load leg was not found.".into(),
    }
}

fn forbidden_payment_response(leg_id: i64, message: &str) -> EscrowLifecycleResponse {
    EscrowLifecycleResponse {
        success: false,
        leg_id,
        escrow_id: None,
        status_label: "Forbidden".into(),
        message: message.into(),
    }
}
