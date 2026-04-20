use axum::{
    Json, Router,
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use db::{
    auth::find_user_by_id,
    dispatch::{find_load_leg_by_id, find_load_leg_scope},
    payments::{
        EscrowTransitionParams, apply_escrow_transition, find_escrow_by_payment_intent_id,
        find_escrow_for_leg, set_user_stripe_connect_account_id, update_user_connect_state,
    },
};
use domain::{
    auth::UserRole,
    payments::{
        EscrowStatus, EscrowStatusDescriptor, PaymentsModuleContract, StripeWebhookEventDescriptor,
        escrow_status_descriptors, payments_module_contract, stripe_webhook_events,
    },
};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::Sha256;
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

#[derive(Debug, Deserialize)]
struct StripeConnectLinkRequest {
    refresh_url: Option<String>,
    return_url: Option<String>,
}

#[derive(Debug, Serialize)]
struct StripeConnectLinkResponse {
    success: bool,
    user_id: i64,
    account_id: Option<String>,
    onboarding_url: Option<String>,
    message: String,
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
        .route("/connect/onboarding-link", post(connect_onboarding_link))
        .route(
            "/connect/users/{user_id}/onboarding-link",
            post(admin_connect_onboarding_link),
        )
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

async fn connect_onboarding_link(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<StripeConnectLinkRequest>,
) -> Json<ApiResponse<StripeConnectLinkResponse>> {
    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(StripeConnectLinkResponse {
            success: false,
            user_id: 0,
            account_id: None,
            onboarding_url: None,
            message: format!(
                "Stripe Connect onboarding is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        }));
    };

    let Some(session) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        return Json(ApiResponse::ok(StripeConnectLinkResponse {
            success: false,
            user_id: 0,
            account_id: None,
            onboarding_url: None,
            message: "Sign in as a carrier before creating a Stripe Connect onboarding link."
                .into(),
        }));
    };

    if session.user.primary_role() != Some(UserRole::Carrier) {
        return Json(ApiResponse::ok(StripeConnectLinkResponse {
            success: false,
            user_id: session.user.id,
            account_id: session.user.stripe_connect_account_id.clone(),
            onboarding_url: None,
            message: "Only carrier accounts can start Stripe Connect payout onboarding.".into(),
        }));
    }

    Json(ApiResponse::ok(
        create_or_refresh_connect_link(&state, pool, &session.user, &payload).await,
    ))
}

async fn admin_connect_onboarding_link(
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<StripeConnectLinkRequest>,
) -> Result<Json<ApiResponse<StripeConnectLinkResponse>>, StatusCode> {
    let _session = resolve_payments_session(&state, &headers)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .filter(|session| session.user.primary_role() == Some(UserRole::Admin))
        .ok_or(StatusCode::FORBIDDEN)?;

    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(StripeConnectLinkResponse {
            success: false,
            user_id,
            account_id: None,
            onboarding_url: None,
            message: format!(
                "Stripe Connect onboarding is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        })));
    };

    let Some(user) = find_user_by_id(pool, user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    else {
        return Ok(Json(ApiResponse::ok(StripeConnectLinkResponse {
            success: false,
            user_id,
            account_id: None,
            onboarding_url: None,
            message: "The selected carrier account was not found.".into(),
        })));
    };

    if user.primary_role() != Some(UserRole::Carrier) {
        return Ok(Json(ApiResponse::ok(StripeConnectLinkResponse {
            success: false,
            user_id,
            account_id: user.stripe_connect_account_id.clone(),
            onboarding_url: None,
            message: "Only carrier accounts can receive Stripe Connect payout onboarding links."
                .into(),
        })));
    }

    Ok(Json(ApiResponse::ok(
        create_or_refresh_connect_link(&state, pool, &user, &payload).await,
    )))
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
            payment_intent_id: None,
            client_secret: None,
            transfer_id: None,
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
            payment_intent_id: None,
            client_secret: None,
            transfer_id: None,
            status_label: "Locked".into(),
            message: "Escrow cannot be funded until the leg has a booked carrier.".into(),
        }));
    };

    let Some(carrier) = find_user_by_id(pool, payee_user_id).await.ok().flatten() else {
        return Json(ApiResponse::ok(EscrowLifecycleResponse {
            success: false,
            leg_id,
            escrow_id: None,
            payment_intent_id: None,
            client_secret: None,
            transfer_id: None,
            status_label: "Missing Carrier".into(),
            message: "The booked carrier could not be found for this escrow funding flow.".into(),
        }));
    };

    let Some(payer_user_id) = scope.load_owner_user_id else {
        return Json(ApiResponse::ok(EscrowLifecycleResponse {
            success: false,
            leg_id,
            escrow_id: None,
            payment_intent_id: None,
            client_secret: None,
            transfer_id: None,
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
            payment_intent_id: None,
            client_secret: None,
            transfer_id: None,
            status_label: "Missing Amount".into(),
            message: "Escrow funding requires either an explicit amount or a priced leg.".into(),
        }));
    };

    if state.stripe.is_configured() && payload.payment_intent_id.is_none() {
        if carrier
            .stripe_connect_account_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .is_none()
        {
            return Json(ApiResponse::ok(EscrowLifecycleResponse {
                success: false,
                leg_id,
                escrow_id: None,
                payment_intent_id: None,
                client_secret: None,
                transfer_id: None,
                status_label: "Carrier Not Ready".into(),
                message: "Carrier has not completed Stripe Connect payout setup, so a live PaymentIntent cannot be created.".into(),
            }));
        }

        let currency = payload
            .currency
            .as_deref()
            .unwrap_or("usd")
            .to_ascii_lowercase();
        let transfer_group = payload
            .transfer_group
            .clone()
            .unwrap_or_else(|| format!("LEG_{}", leg_id));
        let description = format!(
            "Funding leg {} for ${:.2}",
            leg.leg_code
                .clone()
                .unwrap_or_else(|| format!("LEG-{}", leg_id)),
            amount as f64 / 100.0
        );
        let payment_intent = match state
            .stripe
            .create_payment_intent(amount, &currency, &transfer_group, leg_id, &description)
            .await
        {
            Ok(value) => value,
            Err(error) => {
                return Json(ApiResponse::ok(EscrowLifecycleResponse {
                    success: false,
                    leg_id,
                    escrow_id: None,
                    payment_intent_id: None,
                    client_secret: None,
                    transfer_id: None,
                    status_label: "Stripe Error".into(),
                    message: format!("Stripe PaymentIntent creation failed: {}", error),
                }));
            }
        };

        return match apply_escrow_transition(
            pool,
            EscrowTransitionParams {
                leg_id,
                payer_user_id,
                payee_user_id,
                amount,
                platform_fee: payload.platform_fee_cents.unwrap_or(0),
                currency: &currency,
                status: EscrowStatus::Unfunded,
                transfer_group: Some(&transfer_group),
                payment_intent_id: Some(&payment_intent.id),
                charge_id: None,
                transfer_id: None,
                actor_user_id: Some(session.user.id),
                note: payload
                    .note
                    .as_deref()
                    .or(Some("Live Stripe PaymentIntent created by the Rust payments route.")),
            },
        )
        .await
        {
            Ok(Some(escrow)) => Json(ApiResponse::ok(EscrowLifecycleResponse {
                success: true,
                leg_id,
                escrow_id: Some(escrow.id),
                payment_intent_id: Some(payment_intent.id),
                client_secret: payment_intent.client_secret,
                transfer_id: None,
                status_label: "Payment Intent Created".into(),
                message: "Live Stripe PaymentIntent created. The escrow will move to funded when Stripe sends payment_intent.succeeded.".into(),
            })),
            Ok(None) => Json(ApiResponse::ok(missing_leg_response(leg_id))),
            Err(error) => Json(ApiResponse::ok(EscrowLifecycleResponse {
                success: false,
                leg_id,
                escrow_id: None,
                payment_intent_id: Some(payment_intent.id),
                client_secret: payment_intent.client_secret,
                transfer_id: None,
                status_label: "Error".into(),
                message: format!("PaymentIntent was created, but escrow persistence failed: {}", error),
            })),
        };
    }

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
                payment_intent_id: escrow.payment_intent_id,
                client_secret: None,
                transfer_id: escrow.transfer_id,
                status_label: "Funded".into(),
                message: "Escrow funded through the Rust payments route; admin and load-board views will refresh through targeted realtime events.".into(),
            }))
        }
        Ok(None) => Json(ApiResponse::ok(missing_leg_response(leg_id))),
        Err(error) => Json(ApiResponse::ok(EscrowLifecycleResponse {
            success: false,
            leg_id,
            escrow_id: None,
            payment_intent_id: None,
            client_secret: None,
            transfer_id: None,
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
            payment_intent_id: None,
            client_secret: None,
            transfer_id: None,
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
            payment_intent_id: None,
            client_secret: None,
            transfer_id: None,
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
                payment_intent_id: escrow.payment_intent_id,
                client_secret: None,
                transfer_id: escrow.transfer_id,
                status_label: "On Hold".into(),
                message: "Escrow placed on hold through the Rust payments route.".into(),
            }))
        }
        Ok(None) => Json(ApiResponse::ok(missing_leg_response(leg_id))),
        Err(error) => Json(ApiResponse::ok(EscrowLifecycleResponse {
            success: false,
            leg_id,
            escrow_id: None,
            payment_intent_id: None,
            client_secret: None,
            transfer_id: None,
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
            payment_intent_id: None,
            client_secret: None,
            transfer_id: None,
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
            payment_intent_id: None,
            client_secret: None,
            transfer_id: None,
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

    let transfer_id = match payload
        .transfer_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(value) => Some(value.to_string()),
        None if state.stripe.is_configured() => {
            if existing_escrow.escrow_status() != Some(EscrowStatus::Funded) {
                return Json(ApiResponse::ok(EscrowLifecycleResponse {
                    success: false,
                    leg_id,
                    escrow_id: Some(existing_escrow.id),
                    payment_intent_id: existing_escrow.payment_intent_id,
                    client_secret: None,
                    transfer_id: existing_escrow.transfer_id,
                    status_label: "Not Funded".into(),
                    message: "Live Stripe release requires a funded escrow.".into(),
                }));
            }

            let Some(charge_id) = existing_escrow
                .charge_id
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
            else {
                return Json(ApiResponse::ok(EscrowLifecycleResponse {
                    success: false,
                    leg_id,
                    escrow_id: Some(existing_escrow.id),
                    payment_intent_id: existing_escrow.payment_intent_id,
                    client_secret: None,
                    transfer_id: existing_escrow.transfer_id,
                    status_label: "Missing Charge".into(),
                    message:
                        "Live Stripe release requires the charge_id from payment_intent.succeeded."
                            .into(),
                }));
            };

            let Some(carrier) = find_user_by_id(pool, payee_user_id).await.ok().flatten() else {
                return Json(ApiResponse::ok(EscrowLifecycleResponse {
                    success: false,
                    leg_id,
                    escrow_id: Some(existing_escrow.id),
                    payment_intent_id: existing_escrow.payment_intent_id,
                    client_secret: None,
                    transfer_id: existing_escrow.transfer_id,
                    status_label: "Missing Carrier".into(),
                    message: "The booked carrier could not be found for the live Stripe transfer."
                        .into(),
                }));
            };

            let Some(destination_account_id) = carrier
                .stripe_connect_account_id
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
            else {
                return Json(ApiResponse::ok(EscrowLifecycleResponse {
                    success: false,
                    leg_id,
                    escrow_id: Some(existing_escrow.id),
                    payment_intent_id: existing_escrow.payment_intent_id,
                    client_secret: None,
                    transfer_id: existing_escrow.transfer_id,
                    status_label: "Carrier Not Ready".into(),
                    message: "Carrier does not have a Stripe Connect account for payout.".into(),
                }));
            };

            let payout_amount = existing_escrow.amount - existing_escrow.platform_fee;
            if payout_amount <= 0 {
                return Json(ApiResponse::ok(EscrowLifecycleResponse {
                    success: false,
                    leg_id,
                    escrow_id: Some(existing_escrow.id),
                    payment_intent_id: existing_escrow.payment_intent_id,
                    client_secret: None,
                    transfer_id: existing_escrow.transfer_id,
                    status_label: "Invalid Payout".into(),
                    message: "Escrow amount minus platform fee must be positive before release."
                        .into(),
                }));
            }

            match state
                .stripe
                .create_transfer(
                    payout_amount,
                    &existing_escrow.currency,
                    destination_account_id,
                    charge_id,
                    existing_escrow.transfer_group.as_deref(),
                )
                .await
            {
                Ok(transfer) => Some(transfer.id),
                Err(error) => {
                    return Json(ApiResponse::ok(EscrowLifecycleResponse {
                        success: false,
                        leg_id,
                        escrow_id: Some(existing_escrow.id),
                        payment_intent_id: existing_escrow.payment_intent_id,
                        client_secret: None,
                        transfer_id: existing_escrow.transfer_id,
                        status_label: "Stripe Error".into(),
                        message: format!("Live Stripe transfer failed: {}", error),
                    }));
                }
            }
        }
        None if state.config.stripe_live_transfers_required => {
            return Json(ApiResponse::ok(EscrowLifecycleResponse {
                success: false,
                leg_id,
                escrow_id: Some(existing_escrow.id),
                payment_intent_id: existing_escrow.payment_intent_id,
                client_secret: None,
                transfer_id: existing_escrow.transfer_id,
                status_label: "Stripe Required".into(),
                message: "STRIPE_LIVE_TRANSFERS_REQUIRED is enabled, but no live Stripe transfer could be created because STRIPE_SECRET is not configured.".into(),
            }));
        }
        None => None,
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
            transfer_id: transfer_id.as_deref(),
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
                payment_intent_id: escrow.payment_intent_id,
                client_secret: None,
                transfer_id: escrow.transfer_id,
                status_label: "Released".into(),
                message: "Escrow released through the Rust payments route; payout readiness will refresh on admin and load-board views.".into(),
            }))
        }
        Ok(None) => Json(ApiResponse::ok(missing_leg_response(leg_id))),
        Err(error) => Json(ApiResponse::ok(EscrowLifecycleResponse {
            success: false,
            leg_id,
            escrow_id: None,
            payment_intent_id: None,
            client_secret: None,
            transfer_id: transfer_id,
            status_label: "Error".into(),
            message: format!("Escrow release failed: {}", error),
        })),
    }
}

async fn stripe_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<ApiResponse<StripeWebhookResponse>>, StatusCode> {
    let actor_session = authorize_payments_webhook(&state, &headers, &body).await?;
    let payload = parse_stripe_webhook_payload(&body).map_err(|_| StatusCode::BAD_REQUEST)?;
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

async fn create_or_refresh_connect_link(
    state: &AppState,
    pool: &db::DbPool,
    user: &db::auth::UserRecord,
    payload: &StripeConnectLinkRequest,
) -> StripeConnectLinkResponse {
    if !state.stripe.is_configured() {
        return StripeConnectLinkResponse {
            success: false,
            user_id: user.id,
            account_id: user.stripe_connect_account_id.clone(),
            onboarding_url: None,
            message: "STRIPE_SECRET is not configured, so Rust cannot create a live Stripe Connect onboarding link.".into(),
        };
    }

    let account_id = match user
        .stripe_connect_account_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(existing) => existing.to_string(),
        None => match state.stripe.create_express_account(&user.email).await {
            Ok(account) => {
                if let Err(error) =
                    set_user_stripe_connect_account_id(pool, user.id, &account.id).await
                {
                    return StripeConnectLinkResponse {
                        success: false,
                        user_id: user.id,
                        account_id: Some(account.id),
                        onboarding_url: None,
                        message: format!(
                            "Stripe account was created, but saving it to the Rust database failed: {}",
                            error
                        ),
                    };
                }
                account.id
            }
            Err(error) => {
                return StripeConnectLinkResponse {
                    success: false,
                    user_id: user.id,
                    account_id: None,
                    onboarding_url: None,
                    message: format!("Stripe Express account creation failed: {}", error),
                };
            }
        },
    };

    match state
        .stripe
        .create_account_link_with_urls(
            &account_id,
            payload.refresh_url.as_deref(),
            payload.return_url.as_deref(),
        )
        .await
    {
        Ok(link) => StripeConnectLinkResponse {
            success: true,
            user_id: user.id,
            account_id: Some(account_id),
            onboarding_url: Some(link.url),
            message: "Stripe Connect onboarding link created through the Rust payments backend."
                .into(),
        },
        Err(error) => StripeConnectLinkResponse {
            success: false,
            user_id: user.id,
            account_id: Some(account_id),
            onboarding_url: None,
            message: format!("Stripe Connect onboarding link creation failed: {}", error),
        },
    }
}

async fn authorize_payments_webhook(
    state: &AppState,
    headers: &HeaderMap,
    payload: &[u8],
) -> Result<Option<ResolvedSession>, StatusCode> {
    let configured_secrets = stripe_webhook_secrets(state);
    if !configured_secrets.is_empty() {
        if let Some(signature_header) = headers
            .get("stripe-signature")
            .and_then(|value| value.to_str().ok())
        {
            return verify_stripe_signature(signature_header, payload, &configured_secrets)
                .map(|_| None)
                .map_err(|_| StatusCode::BAD_REQUEST);
        }

        let supplied_secret = headers
            .get("x-stripe-webhook-secret")
            .and_then(|value| value.to_str().ok())
            .map(str::trim)
            .filter(|value| !value.is_empty());

        if supplied_secret
            .map(|supplied| configured_secrets.iter().any(|secret| supplied == *secret))
            .unwrap_or(false)
        {
            return Ok(None);
        }
    }

    match resolve_payments_session(state, headers).await {
        Ok(Some(session)) => Ok(Some(session)),
        Ok(None) => Err(StatusCode::UNAUTHORIZED),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

fn stripe_webhook_secrets(state: &AppState) -> Vec<&str> {
    [
        state.config.stripe_webhook_shared_secret.as_deref(),
        state.config.stripe_webhook_connect_secret.as_deref(),
    ]
    .into_iter()
    .flatten()
    .map(str::trim)
    .filter(|value| !value.is_empty())
    .collect()
}

fn parse_stripe_webhook_payload(body: &[u8]) -> Result<StripeWebhookRequest, String> {
    if let Ok(payload) = serde_json::from_slice::<StripeWebhookRequest>(body) {
        return Ok(payload);
    }

    let value = serde_json::from_slice::<Value>(body)
        .map_err(|error| format!("Stripe webhook JSON parsing failed: {}", error))?;
    let event_type = value
        .get("type")
        .and_then(Value::as_str)
        .ok_or_else(|| "Stripe webhook event type is missing.".to_string())?
        .to_string();
    let object = value
        .get("data")
        .and_then(|data| data.get("object"))
        .ok_or_else(|| "Stripe webhook data.object is missing.".to_string())?;

    match event_type.as_str() {
        "payment_intent.succeeded" | "payment_intent.payment_failed" => {
            let payment_intent_id = object.get("id").and_then(Value::as_str).map(str::to_string);
            let charge_id = object
                .get("latest_charge")
                .and_then(Value::as_str)
                .map(str::to_string)
                .or_else(|| {
                    object
                        .get("charges")
                        .and_then(|charges| charges.get("data"))
                        .and_then(Value::as_array)
                        .and_then(|items| items.first())
                        .and_then(|charge| charge.get("id"))
                        .and_then(Value::as_str)
                        .map(str::to_string)
                });

            Ok(StripeWebhookRequest {
                event_type,
                leg_id: object
                    .get("metadata")
                    .and_then(|metadata| metadata.get("leg_id"))
                    .and_then(|value| value.as_i64().or_else(|| value.as_str()?.parse().ok())),
                payment_intent_id,
                charge_id,
                transfer_id: None,
                transfer_group: object
                    .get("transfer_group")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                amount_cents: object.get("amount").and_then(Value::as_i64),
                currency: object
                    .get("currency")
                    .and_then(Value::as_str)
                    .map(str::to_uppercase),
                platform_fee_cents: object.get("application_fee_amount").and_then(Value::as_i64),
                stripe_account_id: None,
                payouts_enabled: None,
                kyc_status: None,
                note: Some("Parsed from a verified Stripe webhook payload.".into()),
            })
        }
        "account.updated" => {
            let requirements_due = object
                .get("requirements")
                .and_then(|requirements| requirements.get("currently_due"))
                .and_then(Value::as_array)
                .map(|items| !items.is_empty())
                .unwrap_or(false);

            Ok(StripeWebhookRequest {
                event_type,
                leg_id: None,
                payment_intent_id: None,
                charge_id: None,
                transfer_id: None,
                transfer_group: None,
                amount_cents: None,
                currency: None,
                platform_fee_cents: None,
                stripe_account_id: object.get("id").and_then(Value::as_str).map(str::to_string),
                payouts_enabled: object.get("payouts_enabled").and_then(Value::as_bool),
                kyc_status: Some(if requirements_due {
                    "pending".into()
                } else {
                    "verified".into()
                }),
                note: Some("Parsed from a verified Stripe account webhook payload.".into()),
            })
        }
        _ => Ok(StripeWebhookRequest {
            event_type,
            leg_id: None,
            payment_intent_id: None,
            charge_id: None,
            transfer_id: None,
            transfer_group: None,
            amount_cents: None,
            currency: None,
            platform_fee_cents: None,
            stripe_account_id: None,
            payouts_enabled: None,
            kyc_status: None,
            note: Some("Unsupported Stripe event parsed for acknowledgement routing.".into()),
        }),
    }
}

fn verify_stripe_signature(
    signature_header: &str,
    payload: &[u8],
    secrets: &[&str],
) -> Result<(), String> {
    let timestamp = parse_stripe_signature_part(signature_header, "t")
        .ok_or_else(|| "Stripe signature timestamp is missing.".to_string())?;
    let signatures = parse_stripe_signature_parts(signature_header, "v1");
    if signatures.is_empty() {
        return Err("Stripe v1 signature is missing.".into());
    }

    let timestamp_seconds = timestamp
        .parse::<i64>()
        .map_err(|_| "Stripe signature timestamp is invalid.".to_string())?;
    let now_seconds = chrono::Utc::now().timestamp();
    if (now_seconds - timestamp_seconds).abs() > 300 {
        return Err("Stripe signature timestamp is outside the allowed tolerance.".into());
    }

    let signed_payload = [timestamp.as_bytes(), b".", payload].concat();
    for secret in secrets {
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
            .map_err(|_| "Stripe webhook secret is invalid.".to_string())?;
        mac.update(&signed_payload);
        let expected = mac.finalize().into_bytes();

        for signature in &signatures {
            if let Ok(decoded) = decode_hex(signature) {
                if mac_verify_bytes(expected.as_slice(), decoded.as_slice()) {
                    return Ok(());
                }
            }
        }
    }

    Err("Stripe signature verification failed.".into())
}

fn parse_stripe_signature_part<'a>(header: &'a str, key: &str) -> Option<&'a str> {
    parse_stripe_signature_parts(header, key).into_iter().next()
}

fn parse_stripe_signature_parts<'a>(header: &'a str, key: &str) -> Vec<&'a str> {
    header
        .split(',')
        .filter_map(|part| {
            let (part_key, value) = part.trim().split_once('=')?;
            (part_key == key).then_some(value.trim())
        })
        .filter(|value| !value.is_empty())
        .collect()
}

fn decode_hex(value: &str) -> Result<Vec<u8>, String> {
    let value = value.trim();
    if !value.len().is_multiple_of(2) {
        return Err("Hex value has an odd length.".into());
    }

    let mut bytes = Vec::with_capacity(value.len() / 2);
    for pair in value.as_bytes().chunks_exact(2) {
        let high = hex_value(pair[0])?;
        let low = hex_value(pair[1])?;
        bytes.push((high << 4) | low);
    }
    Ok(bytes)
}

fn hex_value(byte: u8) -> Result<u8, String> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err("Invalid hex digit.".into()),
    }
}

fn mac_verify_bytes(expected: &[u8], supplied: &[u8]) -> bool {
    if expected.len() != supplied.len() {
        return false;
    }

    expected
        .iter()
        .zip(supplied.iter())
        .fold(0_u8, |acc, (left, right)| acc | (left ^ right))
        == 0
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
        payment_intent_id: None,
        client_secret: None,
        transfer_id: None,
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
        payment_intent_id: None,
        client_secret: None,
        transfer_id: None,
        status_label: "Missing".into(),
        message: "The requested load leg was not found.".into(),
    }
}

fn forbidden_payment_response(leg_id: i64, message: &str) -> EscrowLifecycleResponse {
    EscrowLifecycleResponse {
        success: false,
        leg_id,
        escrow_id: None,
        payment_intent_id: None,
        client_secret: None,
        transfer_id: None,
        status_label: "Forbidden".into(),
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verifies_valid_stripe_signature() {
        let payload = br#"{"event_type":"payment_intent.succeeded","payment_intent_id":"pi_test"}"#;
        let secret = "whsec_test";
        let timestamp = chrono::Utc::now().timestamp().to_string();
        let signature = sign_test_payload(secret, &timestamp, payload);
        let header = format!("t={},v1={}", timestamp, signature);

        assert!(verify_stripe_signature(&header, payload, &[secret]).is_ok());
    }

    #[test]
    fn rejects_invalid_stripe_signature() {
        let payload = br#"{"event_type":"payment_intent.succeeded","payment_intent_id":"pi_test"}"#;
        let timestamp = chrono::Utc::now().timestamp().to_string();
        let header = format!("t={},v1=deadbeef", timestamp);

        assert!(verify_stripe_signature(&header, payload, &["whsec_test"]).is_err());
    }

    #[test]
    fn parses_real_stripe_payment_intent_payload() {
        let payload = br#"{
            "id": "evt_test",
            "type": "payment_intent.succeeded",
            "data": {
                "object": {
                    "id": "pi_test",
                    "amount": 125000,
                    "currency": "usd",
                    "latest_charge": "ch_test",
                    "transfer_group": "leg_9301",
                    "application_fee_amount": 1250,
                    "metadata": { "leg_id": "9311" }
                }
            }
        }"#;

        let parsed = parse_stripe_webhook_payload(payload).expect("payload should parse");

        assert_eq!(parsed.event_type, "payment_intent.succeeded");
        assert_eq!(parsed.payment_intent_id.as_deref(), Some("pi_test"));
        assert_eq!(parsed.charge_id.as_deref(), Some("ch_test"));
        assert_eq!(parsed.leg_id, Some(9311));
        assert_eq!(parsed.amount_cents, Some(125000));
        assert_eq!(parsed.currency.as_deref(), Some("USD"));
        assert_eq!(parsed.platform_fee_cents, Some(1250));
    }

    fn sign_test_payload(secret: &str, timestamp: &str, payload: &[u8]) -> String {
        let signed_payload = [timestamp.as_bytes(), b".", payload].concat();
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).expect("test secret");
        mac.update(&signed_payload);
        mac.finalize()
            .into_bytes()
            .iter()
            .map(|byte| format!("{:02x}", byte))
            .collect::<String>()
    }
}
