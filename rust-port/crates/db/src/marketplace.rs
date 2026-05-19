use chrono::{Duration, NaiveDateTime, Utc};
use domain::auth::UserRole;
use domain::eligibility::EligibilityDecision;
use domain::marketplace::{
    CounterofferDecision, OfferStatus, TenderDecision, marketplace_module_contract,
    offer_status_descriptors,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::FromRow;

use crate::{
    DbPool,
    eligibility::ensure_carrier_can_book_posting,
    tms::{EnqueueAtmpOutboundEvent, enqueue_atmp_outbound_event},
};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OfferStatusMasterRecord {
    pub id: i16,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub sort_order: i32,
    pub is_terminal: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OfferRecord {
    pub id: i64,
    pub load_leg_id: i64,
    pub carrier_id: i64,
    pub conversation_id: Option<i64>,
    pub amount: f64,
    pub status_id: i16,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitCarrierOfferInput<'a> {
    pub tenant_id: &'a str,
    pub posting_id: i64,
    pub carrier_profile_id: i64,
    pub carrier_user_id: i64,
    pub amount: f64,
    pub currency: &'a str,
    pub message: Option<&'a str>,
    pub idempotency_key: Option<&'a str>,
    pub created_by: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SubmittedOfferRecord {
    pub id: i64,
    pub load_leg_id: i64,
    pub carrier_id: i64,
    pub carrier_profile_id: Option<i64>,
    pub posting_id: Option<i64>,
    pub conversation_id: Option<i64>,
    pub amount: f64,
    pub currency: String,
    pub status_id: i16,
    pub version_count: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCounterofferInput<'a> {
    pub tenant_id: &'a str,
    pub offer_id: i64,
    pub from_party_type: &'a str,
    pub to_party_type: &'a str,
    pub amount: f64,
    pub currency: &'a str,
    pub message: Option<&'a str>,
    pub created_by: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CounterofferRecord {
    pub id: i64,
    pub tenant_id: String,
    pub offer_id: i64,
    pub posting_id: i64,
    pub amount: f64,
    pub currency: String,
    pub status: String,
    pub message: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTenderInviteInput<'a> {
    pub tenant_id: &'a str,
    pub posting_id: i64,
    pub carrier_profile_id: i64,
    pub tender_type: &'a str,
    pub expires_minutes: Option<i64>,
    pub created_by: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TenderInviteRecord {
    pub tender_id: i64,
    pub invite_id: i64,
    pub tenant_id: String,
    pub posting_id: i64,
    pub carrier_profile_id: i64,
    pub tender_status: String,
    pub invite_status: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookNowInput<'a> {
    pub tenant_id: &'a str,
    pub posting_id: i64,
    pub carrier_profile_id: i64,
    pub carrier_user_id: i64,
    pub offer_id: Option<i64>,
    pub tender_id: Option<i64>,
    pub amount: Option<f64>,
    pub currency: &'a str,
    pub terms_accepted: bool,
    pub idempotency_key: Option<&'a str>,
    pub created_by: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BookingAwardRecord {
    pub id: i64,
    pub tenant_id: String,
    pub posting_id: i64,
    pub offer_id: Option<i64>,
    pub tender_id: Option<i64>,
    pub carrier_profile_id: i64,
    pub award_number: String,
    pub status: String,
    pub awarded_amount: Option<f64>,
    pub currency: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarrierCancellationInput<'a> {
    pub tenant_id: &'a str,
    pub posting_id: i64,
    pub booking_award_id: Option<i64>,
    pub requested_by: i64,
    pub reason_code: &'a str,
    pub reason_detail: Option<&'a str>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CancellationRequestRecord {
    pub id: i64,
    pub tenant_id: String,
    pub posting_id: i64,
    pub booking_award_id: Option<i64>,
    pub requested_by: Option<i64>,
    pub reason_code: String,
    pub reason_detail: Option<String>,
    pub status: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl OfferRecord {
    pub fn status(&self) -> Option<OfferStatus> {
        OfferStatus::from_legacy_code(self.status_id)
    }
}

pub async fn submit_carrier_offer(
    pool: &DbPool,
    input: SubmitCarrierOfferInput<'_>,
) -> Result<SubmittedOfferRecord, sqlx::Error> {
    ensure_carrier_can_book_posting(
        pool,
        input.tenant_id,
        input.posting_id,
        input.carrier_profile_id,
    )
    .await
    .map_err(|error| sqlx::Error::Protocol(error.to_string().into()))?;

    let scope = posting_legacy_scope(pool, input.tenant_id, input.posting_id).await?;
    let mut tx = pool.begin().await?;
    let conversation_id = upsert_conversation_tx(
        &mut tx,
        scope.load_leg_id,
        scope.shipper_user_id,
        input.carrier_user_id,
    )
    .await?;

    let offer_id = sqlx::query_scalar::<_, i64>(
        r#"
        INSERT INTO offers
            (tenant_id, posting_id, carrier_profile_id, load_leg_id, carrier_id,
             conversation_id, amount, status_id, offer_type, currency, idempotency_key,
             metadata, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, 1, 'spot', $8, $9, $10, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        ON CONFLICT (tenant_id, idempotency_key) WHERE tenant_id IS NOT NULL AND idempotency_key IS NOT NULL
        DO UPDATE SET
            amount = EXCLUDED.amount,
            currency = EXCLUDED.currency,
            conversation_id = EXCLUDED.conversation_id,
            updated_at = CURRENT_TIMESTAMP
        RETURNING id
        "#,
    )
    .bind(input.tenant_id)
    .bind(input.posting_id)
    .bind(input.carrier_profile_id)
    .bind(scope.load_leg_id)
    .bind(input.carrier_user_id)
    .bind(conversation_id)
    .bind(input.amount)
    .bind(input.currency.trim().to_uppercase())
    .bind(input.idempotency_key.map(str::trim).filter(|value| !value.is_empty()))
    .bind(json!({ "message": input.message, "source": "stloads_marketplace" }))
    .fetch_one(&mut *tx)
    .await?;

    insert_offer_version_tx(
        &mut tx,
        input.tenant_id,
        offer_id,
        input.amount,
        input.currency,
        input.message,
        input.created_by,
    )
    .await?;

    if let Some(message) = input
        .message
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        create_system_message_tx(&mut tx, conversation_id, input.created_by, message).await?;
    }

    let record = find_submitted_offer_tx(&mut tx, offer_id).await?;
    tx.commit().await?;

    enqueue_marketplace_event(
        pool,
        input.tenant_id,
        "offer_submitted",
        input.posting_id,
        None,
        json!({
            "offer_id": offer_id,
            "posting_id": input.posting_id,
            "carrier_profile_id": input.carrier_profile_id,
            "amount": input.amount,
            "currency": input.currency,
        }),
    )
    .await?;

    Ok(record)
}

pub async fn create_counteroffer(
    pool: &DbPool,
    input: CreateCounterofferInput<'_>,
) -> Result<CounterofferRecord, sqlx::Error> {
    let offer = find_offer_by_id(pool, input.offer_id)
        .await?
        .ok_or_else(|| sqlx::Error::RowNotFound)?;
    let posting_id = offer_posting_id(pool, input.tenant_id, input.offer_id).await?;
    let counter = sqlx::query_as::<_, CounterofferRecord>(
        r#"
        INSERT INTO counteroffers
            (tenant_id, offer_id, posting_id, from_party_type, to_party_type, amount,
             currency, message, status, created_by, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'pending', $9, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        RETURNING id, tenant_id, offer_id, posting_id, amount::double precision AS amount,
                  currency, status, message, created_at, updated_at
        "#,
    )
    .bind(input.tenant_id)
    .bind(input.offer_id)
    .bind(posting_id)
    .bind(input.from_party_type.trim())
    .bind(input.to_party_type.trim())
    .bind(input.amount)
    .bind(input.currency.trim().to_uppercase())
    .bind(input.message)
    .bind(input.created_by)
    .fetch_one(pool)
    .await?;

    if let Some(conversation_id) = offer.conversation_id {
        if let Some(message) = input
            .message
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            let _ = create_message(pool, conversation_id, input.created_by, message).await?;
        }
    }

    enqueue_marketplace_event(
        pool,
        input.tenant_id,
        "counteroffer_submitted",
        posting_id,
        None,
        json!({
            "counteroffer_id": counter.id,
            "offer_id": input.offer_id,
            "posting_id": posting_id,
            "amount": input.amount,
            "currency": input.currency,
        }),
    )
    .await?;

    Ok(counter)
}

pub async fn respond_to_counteroffer(
    pool: &DbPool,
    tenant_id: &str,
    counteroffer_id: i64,
    decision: &str,
    note: Option<&str>,
    actor_user_id: i64,
) -> Result<CounterofferRecord, sqlx::Error> {
    let decision = CounterofferDecision::parse(decision)
        .ok_or_else(|| sqlx::Error::Protocol("invalid counteroffer decision".into()))?;
    let status = match decision {
        CounterofferDecision::Accept => "accepted",
        CounterofferDecision::Reject => "rejected",
    };

    let mut tx = pool.begin().await?;
    let counter = sqlx::query_as::<_, CounterofferRecord>(
        r#"
        UPDATE counteroffers
        SET status = $1, updated_at = CURRENT_TIMESTAMP
        WHERE tenant_id = $2 AND id = $3 AND status = 'pending'
        RETURNING id, tenant_id, offer_id, posting_id, amount::double precision AS amount,
                  currency, status, message, created_at, updated_at
        "#,
    )
    .bind(status)
    .bind(tenant_id)
    .bind(counteroffer_id)
    .fetch_one(&mut *tx)
    .await?;

    if matches!(decision, CounterofferDecision::Accept) {
        sqlx::query(
            "UPDATE offers SET amount = $1, currency = $2, updated_at = CURRENT_TIMESTAMP WHERE id = $3",
        )
        .bind(counter.amount)
        .bind(&counter.currency)
        .bind(counter.offer_id)
        .execute(&mut *tx)
        .await?;
        insert_offer_version_tx(
            &mut tx,
            tenant_id,
            counter.offer_id,
            counter.amount,
            &counter.currency,
            note,
            actor_user_id,
        )
        .await?;
    }

    tx.commit().await?;
    Ok(counter)
}

pub async fn create_tender_invite(
    pool: &DbPool,
    input: CreateTenderInviteInput<'_>,
) -> Result<TenderInviteRecord, sqlx::Error> {
    ensure_carrier_can_book_posting(
        pool,
        input.tenant_id,
        input.posting_id,
        input.carrier_profile_id,
    )
    .await
    .map_err(|error| sqlx::Error::Protocol(error.to_string().into()))?;

    let mut tx = pool.begin().await?;
    let tender_number = format!("TND-{}-{}", input.posting_id, uuid::Uuid::new_v4().simple());
    let expires_at = input
        .expires_minutes
        .map(|minutes| Utc::now().naive_utc() + Duration::minutes(minutes.max(1)));
    let tender_id = sqlx::query_scalar::<_, i64>(
        r#"
        INSERT INTO tenders
            (tenant_id, posting_id, tender_number, tender_type, status, expires_at,
             created_by, created_at, updated_at)
        VALUES ($1, $2, $3, $4, 'sent', $5, $6, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        RETURNING id
        "#,
    )
    .bind(input.tenant_id)
    .bind(input.posting_id)
    .bind(tender_number)
    .bind(input.tender_type.trim())
    .bind(expires_at)
    .bind(input.created_by)
    .fetch_one(&mut *tx)
    .await?;

    let invite_id = sqlx::query_scalar::<_, i64>(
        r#"
        INSERT INTO tender_invites
            (tenant_id, tender_id, carrier_profile_id, invite_status, sent_at, created_at, updated_at)
        VALUES ($1, $2, $3, 'sent', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        RETURNING id
        "#,
    )
    .bind(input.tenant_id)
    .bind(tender_id)
    .bind(input.carrier_profile_id)
    .fetch_one(&mut *tx)
    .await?;

    let record = find_tender_invite_tx(&mut tx, invite_id).await?;
    tx.commit().await?;
    Ok(record)
}

pub async fn respond_to_tender_invite(
    pool: &DbPool,
    tenant_id: &str,
    invite_id: i64,
    decision: &str,
    response_note: Option<&str>,
    _actor_user_id: i64,
) -> Result<TenderInviteRecord, sqlx::Error> {
    let decision = TenderDecision::parse(decision)
        .ok_or_else(|| sqlx::Error::Protocol("invalid tender decision".into()))?;
    let invite_status = match decision {
        TenderDecision::Accept => "accepted",
        TenderDecision::Reject => "declined",
    };
    let tender_status = match decision {
        TenderDecision::Accept => "accepted",
        TenderDecision::Reject => "declined",
    };

    let mut tx = pool.begin().await?;
    let invite = sqlx::query_as::<_, TenderInviteRecord>(
        r#"
        UPDATE tender_invites invite
        SET invite_status = $1,
            response_note = $2,
            responded_at = CURRENT_TIMESTAMP,
            updated_at = CURRENT_TIMESTAMP
        FROM tenders tender
        WHERE invite.tender_id = tender.id
          AND invite.tenant_id = $3
          AND invite.id = $4
          AND invite.invite_status = 'sent'
        RETURNING tender.id AS tender_id,
                  invite.id AS invite_id,
                  invite.tenant_id,
                  tender.posting_id,
                  invite.carrier_profile_id,
                  tender.status AS tender_status,
                  invite.invite_status,
                  invite.created_at,
                  invite.updated_at
        "#,
    )
    .bind(invite_status)
    .bind(response_note)
    .bind(tenant_id)
    .bind(invite_id)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        "UPDATE tenders SET status = $1, accepted_at = CASE WHEN $1 = 'accepted' THEN CURRENT_TIMESTAMP ELSE accepted_at END, declined_at = CASE WHEN $1 = 'declined' THEN CURRENT_TIMESTAMP ELSE declined_at END, updated_at = CURRENT_TIMESTAMP WHERE tenant_id = $2 AND id = $3",
    )
    .bind(tender_status)
    .bind(tenant_id)
    .bind(invite.tender_id)
    .execute(&mut *tx)
    .await?;

    let updated = find_tender_invite_tx(&mut tx, invite_id).await?;
    tx.commit().await?;
    Ok(updated)
}

pub async fn book_now_posting(
    pool: &DbPool,
    input: BookNowInput<'_>,
) -> Result<BookingAwardRecord, sqlx::Error> {
    if !input.terms_accepted {
        return Err(sqlx::Error::Protocol(
            "book-now requires explicit terms confirmation".into(),
        ));
    }

    let _eligibility: EligibilityDecision = ensure_carrier_can_book_posting(
        pool,
        input.tenant_id,
        input.posting_id,
        input.carrier_profile_id,
    )
    .await
    .map_err(|error| sqlx::Error::Protocol(error.to_string().into()))?;

    let scope = posting_legacy_scope(pool, input.tenant_id, input.posting_id).await?;
    let mut tx = pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock($1)")
        .bind(input.posting_id)
        .execute(&mut *tx)
        .await?;

    let active_award = sqlx::query_scalar::<_, i64>(
        "SELECT id FROM booking_awards WHERE tenant_id = $1 AND posting_id = $2 AND status IN ('awarded', 'accepted', 'in_transit') LIMIT 1",
    )
    .bind(input.tenant_id)
    .bind(input.posting_id)
    .fetch_optional(&mut *tx)
    .await?;
    if active_award.is_some() {
        tx.rollback().await?;
        return Err(sqlx::Error::Protocol(
            "posting already has an active booking award".into(),
        ));
    }

    let lock_token = format!(
        "{}:{}:{}",
        input.tenant_id,
        input.posting_id,
        input.idempotency_key.unwrap_or("book_now")
    );
    sqlx::query(
        "INSERT INTO booking_locks
            (tenant_id, posting_id, locked_by, lock_reason, lock_token, expires_at, created_at)
         VALUES ($1, $2, $3, 'book_now', $4, CURRENT_TIMESTAMP + INTERVAL '10 minutes', CURRENT_TIMESTAMP)",
    )
    .bind(input.tenant_id)
    .bind(input.posting_id)
    .bind(input.carrier_profile_id.to_string())
    .bind(lock_token)
    .execute(&mut *tx)
    .await?;

    let award_number = format!("AWD-{}-{}", input.posting_id, uuid::Uuid::new_v4().simple());
    let award_id = sqlx::query_scalar::<_, i64>(
        r#"
        INSERT INTO booking_awards
            (tenant_id, posting_id, offer_id, tender_id, carrier_profile_id, award_number,
             status, awarded_amount, currency, metadata, created_by, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, 'awarded', $7, $8, $9, $10, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        RETURNING id
        "#,
    )
    .bind(input.tenant_id)
    .bind(input.posting_id)
    .bind(input.offer_id)
    .bind(input.tender_id)
    .bind(input.carrier_profile_id)
    .bind(award_number)
    .bind(input.amount)
    .bind(input.currency.trim().to_uppercase())
    .bind(json!({
        "terms_accepted": input.terms_accepted,
        "idempotency_key": input.idempotency_key,
        "source": "stloads_book_now",
    }))
    .bind(input.created_by)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        "UPDATE stloads_postings SET status = 'awarded', updated_at = CURRENT_TIMESTAMP WHERE tenant_id = $1 AND id = $2",
    )
    .bind(input.tenant_id)
    .bind(input.posting_id)
    .execute(&mut *tx)
    .await?;

    if let Some(offer_id) = input.offer_id {
        sqlx::query(
            "UPDATE offers SET status_id = 3, updated_at = CURRENT_TIMESTAMP WHERE id = $1",
        )
        .bind(offer_id)
        .execute(&mut *tx)
        .await?;
        sqlx::query(
            "UPDATE offers SET status_id = 0, updated_at = CURRENT_TIMESTAMP WHERE posting_id = $1 AND id <> $2 AND status_id = 1",
        )
        .bind(input.posting_id)
        .bind(offer_id)
        .execute(&mut *tx)
        .await?;
    }

    sqlx::query(
        "UPDATE load_legs
         SET accepted_offer_id = $1,
             booked_carrier_id = $2,
             booked_amount = $3,
             booked_at = COALESCE(booked_at, CURRENT_TIMESTAMP),
             status_id = CASE WHEN status_id < 4 THEN 4 ELSE status_id END,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $4",
    )
    .bind(input.offer_id)
    .bind(input.carrier_user_id)
    .bind(input.amount)
    .bind(scope.load_leg_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO load_history (load_id, admin_id, status, remarks, created_at, updated_at)
         VALUES ($1, $2, 4, 'STLoads book-now awarded this posting.', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(scope.load_id)
    .bind(input.created_by)
    .execute(&mut *tx)
    .await?;

    let award = find_booking_award_tx(&mut tx, award_id).await?;
    tx.commit().await?;

    if input.offer_id.is_some() {
        enqueue_marketplace_event(
            pool,
            input.tenant_id,
            "offer_accepted",
            input.posting_id,
            Some(award.id),
            json!({
                "posting_id": input.posting_id,
                "offer_id": input.offer_id,
                "booking_award_id": award.id,
            }),
        )
        .await?;
    }
    enqueue_marketplace_event(
        pool,
        input.tenant_id,
        "carrier_booked",
        input.posting_id,
        Some(award.id),
        json!({
            "posting_id": input.posting_id,
            "booking_award_id": award.id,
            "carrier_profile_id": input.carrier_profile_id,
            "amount": input.amount,
            "currency": input.currency,
        }),
    )
    .await?;

    Ok(award)
}

pub async fn request_carrier_cancellation(
    pool: &DbPool,
    input: CarrierCancellationInput<'_>,
) -> Result<CancellationRequestRecord, sqlx::Error> {
    let request = sqlx::query_as::<_, CancellationRequestRecord>(
        r#"
        INSERT INTO cancellation_requests
            (tenant_id, posting_id, booking_award_id, requested_by, request_source,
             reason_code, reason_detail, status, created_at, updated_at)
        VALUES ($1, $2, $3, $4, 'carrier', $5, $6, 'pending', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        RETURNING id, tenant_id, posting_id, booking_award_id, requested_by,
                  reason_code, reason_detail, status, created_at, updated_at
        "#,
    )
    .bind(input.tenant_id)
    .bind(input.posting_id)
    .bind(input.booking_award_id)
    .bind(input.requested_by)
    .bind(input.reason_code.trim())
    .bind(input.reason_detail)
    .fetch_one(pool)
    .await?;

    enqueue_marketplace_event(
        pool,
        input.tenant_id,
        "booking_canceled",
        input.posting_id,
        input.booking_award_id,
        json!({
            "posting_id": input.posting_id,
            "booking_award_id": input.booking_award_id,
            "cancellation_request_id": request.id,
            "reason_code": input.reason_code,
        }),
    )
    .await?;

    Ok(request)
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ConversationRecord {
    pub id: i64,
    pub load_leg_id: i64,
    pub shipper_id: i64,
    pub carrier_id: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MessageRecord {
    pub id: i64,
    pub conversation_id: i64,
    pub user_id: i64,
    pub body: Option<String>,
    pub meta: Option<Value>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ConversationWorkspaceRecord {
    pub id: i64,
    pub load_leg_id: i64,
    pub load_leg_code: Option<String>,
    pub shipper_id: i64,
    pub shipper_name: String,
    pub carrier_id: i64,
    pub carrier_name: String,
    pub last_message_body: Option<String>,
    pub last_message_user_id: Option<i64>,
    pub last_activity_at: NaiveDateTime,
    pub message_count: i64,
    pub offer_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MessageDetailRecord {
    pub id: i64,
    pub conversation_id: i64,
    pub user_id: i64,
    pub author_name: String,
    pub body: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ConversationReadRecord {
    pub conversation_id: i64,
    pub user_id: i64,
    pub last_read_message_id: Option<i64>,
    pub last_read_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ConversationPresenceRecord {
    pub conversation_id: i64,
    pub user_id: i64,
    pub state: String,
    pub last_seen_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

pub async fn list_offers_for_leg(
    pool: &DbPool,
    load_leg_id: i64,
) -> Result<Vec<OfferRecord>, sqlx::Error> {
    sqlx::query_as::<_, OfferRecord>(
        "SELECT id, load_leg_id, carrier_id, conversation_id, amount::double precision AS amount, status_id, created_at, updated_at
         FROM offers
         WHERE load_leg_id = $1
         ORDER BY id DESC",
    )
    .bind(load_leg_id)
    .fetch_all(pool)
    .await
}

pub async fn list_pending_offers_for_leg(
    pool: &DbPool,
    load_leg_id: i64,
) -> Result<Vec<OfferRecord>, sqlx::Error> {
    sqlx::query_as::<_, OfferRecord>(
        "SELECT id, load_leg_id, carrier_id, conversation_id, amount::double precision AS amount, status_id, created_at, updated_at
         FROM offers
         WHERE load_leg_id = $1 AND status_id = 1
         ORDER BY id DESC",
    )
    .bind(load_leg_id)
    .fetch_all(pool)
    .await
}

pub async fn list_conversations_for_leg(
    pool: &DbPool,
    load_leg_id: i64,
) -> Result<Vec<ConversationRecord>, sqlx::Error> {
    sqlx::query_as::<_, ConversationRecord>(
        "SELECT id, load_leg_id, shipper_id, carrier_id, created_at, updated_at
         FROM conversations
         WHERE load_leg_id = $1
         ORDER BY id DESC",
    )
    .bind(load_leg_id)
    .fetch_all(pool)
    .await
}

pub async fn find_conversation_for_leg_and_carrier(
    pool: &DbPool,
    load_leg_id: i64,
    carrier_id: i64,
) -> Result<Option<ConversationRecord>, sqlx::Error> {
    sqlx::query_as::<_, ConversationRecord>(
        "SELECT id, load_leg_id, shipper_id, carrier_id, created_at, updated_at
         FROM conversations
         WHERE load_leg_id = $1 AND carrier_id = $2
         LIMIT 1",
    )
    .bind(load_leg_id)
    .bind(carrier_id)
    .fetch_optional(pool)
    .await
}

pub async fn list_messages_for_conversation(
    pool: &DbPool,
    conversation_id: i64,
) -> Result<Vec<MessageRecord>, sqlx::Error> {
    sqlx::query_as::<_, MessageRecord>(
        "SELECT id, conversation_id, user_id, body, meta, created_at, updated_at
         FROM messages
         WHERE conversation_id = $1
         ORDER BY id ASC",
    )
    .bind(conversation_id)
    .fetch_all(pool)
    .await
}

pub async fn list_recent_conversation_workspace_records(
    pool: &DbPool,
    limit: i64,
) -> Result<Vec<ConversationWorkspaceRecord>, sqlx::Error> {
    sqlx::query_as::<_, ConversationWorkspaceRecord>(&format!(
        "{}\n        ORDER BY last_activity_at DESC, c.id DESC\n        LIMIT $1",
        conversation_workspace_select_sql()
    ))
    .bind(limit)
    .fetch_all(pool)
    .await
}

pub async fn list_recent_conversation_workspace_records_for_user(
    pool: &DbPool,
    viewer_user_id: i64,
    viewer_role: Option<UserRole>,
    limit: i64,
) -> Result<Vec<ConversationWorkspaceRecord>, sqlx::Error> {
    if viewer_role == Some(UserRole::Admin) {
        return list_recent_conversation_workspace_records(pool, limit).await;
    }

    sqlx::query_as::<_, ConversationWorkspaceRecord>(&format!(
        "{}\n        WHERE c.shipper_id = $1 OR c.carrier_id = $2\n        ORDER BY last_activity_at DESC, c.id DESC\n        LIMIT $3",
        conversation_workspace_select_sql()
    ))
    .bind(viewer_user_id)
    .bind(viewer_user_id)
    .bind(limit)
    .fetch_all(pool)
    .await
}

pub async fn find_conversation_workspace_record(
    pool: &DbPool,
    conversation_id: i64,
) -> Result<Option<ConversationWorkspaceRecord>, sqlx::Error> {
    sqlx::query_as::<_, ConversationWorkspaceRecord>(&format!(
        "{}\n        WHERE c.id = $1\n        LIMIT 1",
        conversation_workspace_select_sql()
    ))
    .bind(conversation_id)
    .fetch_optional(pool)
    .await
}

pub async fn find_conversation_workspace_record_for_user(
    pool: &DbPool,
    conversation_id: i64,
    viewer_user_id: i64,
    viewer_role: Option<UserRole>,
) -> Result<Option<ConversationWorkspaceRecord>, sqlx::Error> {
    if viewer_role == Some(UserRole::Admin) {
        return find_conversation_workspace_record(pool, conversation_id).await;
    }

    sqlx::query_as::<_, ConversationWorkspaceRecord>(&format!(
        "{}\n        WHERE c.id = $1 AND (c.shipper_id = $2 OR c.carrier_id = $3)\n        LIMIT 1",
        conversation_workspace_select_sql()
    ))
    .bind(conversation_id)
    .bind(viewer_user_id)
    .bind(viewer_user_id)
    .fetch_optional(pool)
    .await
}

pub async fn list_message_details_for_conversation(
    pool: &DbPool,
    conversation_id: i64,
    limit: i64,
) -> Result<Vec<MessageDetailRecord>, sqlx::Error> {
    sqlx::query_as::<_, MessageDetailRecord>(
        r#"
        SELECT
            m.id,
            m.conversation_id,
            m.user_id,
            u.name AS author_name,
            m.body,
            m.created_at
        FROM messages m
        INNER JOIN users u ON u.id = m.user_id
        WHERE m.conversation_id = $1
        ORDER BY m.id DESC
        LIMIT $2
        "#,
    )
    .bind(conversation_id)
    .bind(limit)
    .fetch_all(pool)
    .await
}

pub async fn find_offer_by_id(
    pool: &DbPool,
    offer_id: i64,
) -> Result<Option<OfferRecord>, sqlx::Error> {
    sqlx::query_as::<_, OfferRecord>(
        "SELECT id, load_leg_id, carrier_id, conversation_id, amount::double precision AS amount, status_id, created_at, updated_at
         FROM offers
         WHERE id = $1
         LIMIT 1",
    )
    .bind(offer_id)
    .fetch_optional(pool)
    .await
}

pub async fn find_conversation_by_id(
    pool: &DbPool,
    conversation_id: i64,
) -> Result<Option<ConversationRecord>, sqlx::Error> {
    sqlx::query_as::<_, ConversationRecord>(
        "SELECT id, load_leg_id, shipper_id, carrier_id, created_at, updated_at
         FROM conversations
         WHERE id = $1
         LIMIT 1",
    )
    .bind(conversation_id)
    .fetch_optional(pool)
    .await
}

pub async fn latest_message_id_for_conversation(
    pool: &DbPool,
    conversation_id: i64,
) -> Result<Option<i64>, sqlx::Error> {
    let row = sqlx::query_scalar::<_, i64>(
        "SELECT id
         FROM messages
         WHERE conversation_id = $1
         ORDER BY id DESC
         LIMIT 1",
    )
    .bind(conversation_id)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn mark_conversation_read(
    pool: &DbPool,
    conversation_id: i64,
    user_id: i64,
) -> Result<Option<ConversationReadRecord>, sqlx::Error> {
    let last_read_message_id = latest_message_id_for_conversation(pool, conversation_id).await?;

    sqlx::query(
        "INSERT INTO conversation_reads (conversation_id, user_id, last_read_message_id, last_read_at, created_at, updated_at)
         VALUES ($1, $2, $3, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         ON CONFLICT (conversation_id, user_id) DO UPDATE SET
             last_read_message_id = EXCLUDED.last_read_message_id,
             last_read_at = EXCLUDED.last_read_at,
             updated_at = CURRENT_TIMESTAMP",
    )
    .bind(conversation_id)
    .bind(user_id)
    .bind(last_read_message_id)
    .execute(pool)
    .await?;

    find_conversation_read_state(pool, conversation_id, user_id).await
}

pub async fn find_conversation_read_state(
    pool: &DbPool,
    conversation_id: i64,
    user_id: i64,
) -> Result<Option<ConversationReadRecord>, sqlx::Error> {
    sqlx::query_as::<_, ConversationReadRecord>(
        "SELECT conversation_id, user_id, last_read_message_id, last_read_at, created_at, updated_at
         FROM conversation_reads
         WHERE conversation_id = $1 AND user_id = $2
         LIMIT 1",
    )
    .bind(conversation_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

pub async fn find_peer_conversation_read_state(
    pool: &DbPool,
    conversation_id: i64,
    viewer_user_id: i64,
) -> Result<Option<ConversationReadRecord>, sqlx::Error> {
    sqlx::query_as::<_, ConversationReadRecord>(
        "SELECT conversation_id, user_id, last_read_message_id, last_read_at, created_at, updated_at
         FROM conversation_reads
         WHERE conversation_id = $1 AND user_id <> $2
         ORDER BY last_read_at DESC
         LIMIT 1",
    )
    .bind(conversation_id)
    .bind(viewer_user_id)
    .fetch_optional(pool)
    .await
}

pub async fn count_unread_messages_for_conversation(
    pool: &DbPool,
    conversation_id: i64,
    user_id: i64,
) -> Result<u64, sqlx::Error> {
    let unread = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM messages m
        WHERE m.conversation_id = $1
          AND m.user_id <> $2
          AND m.id > COALESCE(
                (
                    SELECT cr.last_read_message_id
                    FROM conversation_reads cr
                    WHERE cr.conversation_id = $3 AND cr.user_id = $4
                    LIMIT 1
                ),
                0
          )
        "#,
    )
    .bind(conversation_id)
    .bind(user_id)
    .bind(conversation_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(unread.max(0) as u64)
}

pub async fn upsert_conversation_presence(
    pool: &DbPool,
    conversation_id: i64,
    user_id: i64,
    state: &str,
) -> Result<Option<ConversationPresenceRecord>, sqlx::Error> {
    sqlx::query(
        "INSERT INTO conversation_presence (conversation_id, user_id, state, last_seen_at, created_at, updated_at)
         VALUES ($1, $2, $3, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         ON CONFLICT (conversation_id, user_id) DO UPDATE SET
             state = EXCLUDED.state,
             last_seen_at = EXCLUDED.last_seen_at,
             updated_at = CURRENT_TIMESTAMP",
    )
    .bind(conversation_id)
    .bind(user_id)
    .bind(state)
    .execute(pool)
    .await?;

    find_conversation_presence_state(pool, conversation_id, user_id).await
}

pub async fn find_conversation_presence_state(
    pool: &DbPool,
    conversation_id: i64,
    user_id: i64,
) -> Result<Option<ConversationPresenceRecord>, sqlx::Error> {
    sqlx::query_as::<_, ConversationPresenceRecord>(
        "SELECT conversation_id, user_id, state, last_seen_at, created_at, updated_at
         FROM conversation_presence
         WHERE conversation_id = $1 AND user_id = $2
         LIMIT 1",
    )
    .bind(conversation_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

pub async fn delete_conversation_presence(
    pool: &DbPool,
    conversation_id: i64,
    user_id: i64,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "DELETE FROM conversation_presence
         WHERE conversation_id = $1 AND user_id = $2",
    )
    .bind(conversation_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

pub async fn find_active_peer_presence(
    pool: &DbPool,
    conversation_id: i64,
    viewer_user_id: i64,
    max_age_seconds: i64,
) -> Result<Option<ConversationPresenceRecord>, sqlx::Error> {
    let threshold = Utc::now().naive_utc() - Duration::seconds(max_age_seconds.max(5));

    sqlx::query_as::<_, ConversationPresenceRecord>(
        "SELECT conversation_id, user_id, state, last_seen_at, created_at, updated_at
         FROM conversation_presence
         WHERE conversation_id = $1 AND user_id <> $2 AND last_seen_at >= $3
         ORDER BY last_seen_at DESC
         LIMIT 1",
    )
    .bind(conversation_id)
    .bind(viewer_user_id)
    .bind(threshold)
    .fetch_optional(pool)
    .await
}

pub async fn review_offer(
    pool: &DbPool,
    offer_id: i64,
    accept: bool,
    actor_user_id: Option<i64>,
) -> Result<Option<OfferRecord>, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let Some(offer) = sqlx::query_as::<_, OfferRecord>(
        "SELECT id, load_leg_id, carrier_id, conversation_id, amount::double precision AS amount, status_id, created_at, updated_at
         FROM offers
         WHERE id = $1
         LIMIT 1",
    )
    .bind(offer_id)
    .fetch_optional(&mut *tx)
    .await?
    else {
        tx.rollback().await?;
        return Ok(None);
    };

    let final_status_id = if accept { 3 } else { 0 };

    sqlx::query(
        "UPDATE offers
         SET status_id = $1, updated_at = CURRENT_TIMESTAMP
         WHERE id = $2",
    )
    .bind(final_status_id)
    .bind(offer_id)
    .execute(&mut *tx)
    .await?;

    if accept {
        sqlx::query(
            "UPDATE offers
             SET status_id = 0, updated_at = CURRENT_TIMESTAMP
             WHERE load_leg_id = $1 AND id <> $2 AND status_id = 1",
        )
        .bind(offer.load_leg_id)
        .bind(offer_id)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            "UPDATE load_legs
             SET accepted_offer_id = $1,
                 booked_carrier_id = $2,
                 booked_amount = $3,
                 booked_at = COALESCE(booked_at, CURRENT_TIMESTAMP),
                 status_id = CASE WHEN status_id < 4 THEN 4 ELSE status_id END,
                 updated_at = CURRENT_TIMESTAMP
             WHERE id = $4",
        )
        .bind(offer_id)
        .bind(offer.carrier_id)
        .bind(offer.amount)
        .bind(offer.load_leg_id)
        .execute(&mut *tx)
        .await?;
    }

    let load_history_status = if accept { 4 } else { 3 };
    let load_history_note = if accept {
        "Rust marketplace accepted an offer"
    } else {
        "Rust marketplace declined an offer"
    };

    sqlx::query(
        "INSERT INTO load_history (load_id, admin_id, status, remarks, created_at, updated_at)
         SELECT load_id, $1, $2, $3, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
         FROM load_legs
         WHERE id = $4",
    )
    .bind(actor_user_id)
    .bind(load_history_status)
    .bind(load_history_note)
    .bind(offer.load_leg_id)
    .execute(&mut *tx)
    .await?;

    let updated = sqlx::query_as::<_, OfferRecord>(
        "SELECT id, load_leg_id, carrier_id, conversation_id, amount::double precision AS amount, status_id, created_at, updated_at
         FROM offers
         WHERE id = $1
         LIMIT 1",
    )
    .bind(offer_id)
    .fetch_optional(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(updated)
}

pub async fn create_message(
    pool: &DbPool,
    conversation_id: i64,
    user_id: i64,
    body: &str,
) -> Result<Option<MessageDetailRecord>, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let Some(_conversation) = sqlx::query_as::<_, ConversationRecord>(
        "SELECT id, load_leg_id, shipper_id, carrier_id, created_at, updated_at
         FROM conversations
         WHERE id = $1
         LIMIT 1",
    )
    .bind(conversation_id)
    .fetch_optional(&mut *tx)
    .await?
    else {
        tx.rollback().await?;
        return Ok(None);
    };

    let message_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO messages (conversation_id, user_id, body, created_at, updated_at)
         VALUES ($1, $2, $3, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         RETURNING id",
    )
    .bind(conversation_id)
    .bind(user_id)
    .bind(body)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        "UPDATE conversations
         SET updated_at = CURRENT_TIMESTAMP
         WHERE id = $1",
    )
    .bind(conversation_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO conversation_reads (conversation_id, user_id, last_read_message_id, last_read_at, created_at, updated_at)
         VALUES ($1, $2, $3, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         ON CONFLICT (conversation_id, user_id) DO UPDATE SET
             last_read_message_id = EXCLUDED.last_read_message_id,
             last_read_at = EXCLUDED.last_read_at,
             updated_at = CURRENT_TIMESTAMP",
    )
    .bind(conversation_id)
    .bind(user_id)
    .bind(message_id)
    .execute(&mut *tx)
    .await?;

    let message = sqlx::query_as::<_, MessageDetailRecord>(
        r#"
        SELECT
            m.id,
            m.conversation_id,
            m.user_id,
            u.name AS author_name,
            m.body,
            m.created_at
        FROM messages m
        INNER JOIN users u ON u.id = m.user_id
        WHERE m.id = $1
        LIMIT 1
        "#,
    )
    .bind(message_id)
    .fetch_optional(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(message)
}

pub async fn marketplace_contract_summary() -> domain::marketplace::MarketplaceModuleContract {
    marketplace_module_contract()
}

pub async fn offer_status_catalog() -> &'static [domain::marketplace::OfferStatusDescriptor] {
    offer_status_descriptors()
}

#[derive(Debug, Clone, FromRow)]
struct PostingLegacyScope {
    load_id: i64,
    load_leg_id: i64,
    shipper_user_id: i64,
}

async fn posting_legacy_scope(
    pool: &DbPool,
    tenant_id: &str,
    posting_id: i64,
) -> Result<PostingLegacyScope, sqlx::Error> {
    sqlx::query_as::<_, PostingLegacyScope>(
        r#"
        SELECT leg.load_id,
               leg.id AS load_leg_id,
               load_record.user_id AS shipper_user_id
        FROM stloads_postings posting
        INNER JOIN load_legs leg
          ON posting.source_leg_id IS NOT NULL
         AND posting.source_leg_id = leg.id::text
         AND leg.deleted_at IS NULL
        INNER JOIN loads load_record
          ON load_record.id = leg.load_id
         AND load_record.deleted_at IS NULL
        WHERE posting.tenant_id = $1
          AND posting.id = $2
          AND posting.deleted_at IS NULL
        LIMIT 1
        "#,
    )
    .bind(tenant_id)
    .bind(posting_id)
    .fetch_one(pool)
    .await
}

async fn upsert_conversation_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    load_leg_id: i64,
    shipper_id: i64,
    carrier_id: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        r#"
        INSERT INTO conversations
            (load_leg_id, shipper_id, carrier_id, created_at, updated_at)
        VALUES ($1, $2, $3, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        ON CONFLICT (load_leg_id, carrier_id)
        DO UPDATE SET updated_at = CURRENT_TIMESTAMP
        RETURNING id
        "#,
    )
    .bind(load_leg_id)
    .bind(shipper_id)
    .bind(carrier_id)
    .fetch_one(&mut **tx)
    .await
}

async fn create_system_message_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    conversation_id: i64,
    user_id: i64,
    body: &str,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        "INSERT INTO messages (conversation_id, user_id, body, created_at, updated_at)
         VALUES ($1, $2, $3, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         RETURNING id",
    )
    .bind(conversation_id)
    .bind(user_id)
    .bind(body)
    .fetch_one(&mut **tx)
    .await
}

async fn insert_offer_version_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: &str,
    offer_id: i64,
    amount: f64,
    currency: &str,
    message: Option<&str>,
    created_by: i64,
) -> Result<(), sqlx::Error> {
    let next_version = sqlx::query_scalar::<_, i32>(
        "SELECT COALESCE(MAX(version_number), 0) + 1 FROM offer_versions WHERE tenant_id = $1 AND offer_id = $2",
    )
    .bind(tenant_id)
    .bind(offer_id)
    .fetch_one(&mut **tx)
    .await?;
    let terms = json!({ "message": message });
    let payload_hash = format!(
        "offer-version-{}-{}",
        offer_id,
        uuid::Uuid::new_v4().simple()
    );
    sqlx::query(
        "INSERT INTO offer_versions
            (tenant_id, offer_id, version_number, amount, currency, terms,
             payload_hash, created_by, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, CURRENT_TIMESTAMP)",
    )
    .bind(tenant_id)
    .bind(offer_id)
    .bind(next_version)
    .bind(amount)
    .bind(currency.trim().to_uppercase())
    .bind(terms)
    .bind(payload_hash)
    .bind(created_by)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn find_submitted_offer_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    offer_id: i64,
) -> Result<SubmittedOfferRecord, sqlx::Error> {
    sqlx::query_as::<_, SubmittedOfferRecord>(
        r#"
        SELECT offer.id,
               offer.load_leg_id,
               offer.carrier_id,
               offer.carrier_profile_id,
               offer.posting_id,
               offer.conversation_id,
               offer.amount::double precision AS amount,
               offer.currency,
               offer.status_id,
               (SELECT COUNT(*) FROM offer_versions version WHERE version.offer_id = offer.id) AS version_count,
               offer.created_at,
               offer.updated_at
        FROM offers offer
        WHERE offer.id = $1
        LIMIT 1
        "#,
    )
    .bind(offer_id)
    .fetch_one(&mut **tx)
    .await
}

async fn offer_posting_id(
    pool: &DbPool,
    tenant_id: &str,
    offer_id: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        "SELECT posting_id FROM offers WHERE tenant_id = $1 AND id = $2 AND posting_id IS NOT NULL LIMIT 1",
    )
    .bind(tenant_id)
    .bind(offer_id)
    .fetch_one(pool)
    .await
}

async fn find_tender_invite_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    invite_id: i64,
) -> Result<TenderInviteRecord, sqlx::Error> {
    sqlx::query_as::<_, TenderInviteRecord>(
        r#"
        SELECT tender.id AS tender_id,
               invite.id AS invite_id,
               invite.tenant_id,
               tender.posting_id,
               invite.carrier_profile_id,
               tender.status AS tender_status,
               invite.invite_status,
               invite.created_at,
               invite.updated_at
        FROM tender_invites invite
        INNER JOIN tenders tender ON tender.id = invite.tender_id
        WHERE invite.id = $1
        LIMIT 1
        "#,
    )
    .bind(invite_id)
    .fetch_one(&mut **tx)
    .await
}

async fn find_booking_award_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    award_id: i64,
) -> Result<BookingAwardRecord, sqlx::Error> {
    sqlx::query_as::<_, BookingAwardRecord>(
        r#"
        SELECT id,
               tenant_id,
               posting_id,
               offer_id,
               tender_id,
               carrier_profile_id,
               award_number,
               status,
               awarded_amount::double precision AS awarded_amount,
               currency,
               created_at,
               updated_at
        FROM booking_awards
        WHERE id = $1
        LIMIT 1
        "#,
    )
    .bind(award_id)
    .fetch_one(&mut **tx)
    .await
}

async fn enqueue_marketplace_event(
    pool: &DbPool,
    tenant_id: &str,
    event_type: &str,
    posting_id: i64,
    booking_award_id: Option<i64>,
    payload: Value,
) -> Result<i64, sqlx::Error> {
    enqueue_atmp_outbound_event(
        pool,
        EnqueueAtmpOutboundEvent {
            tenant_id,
            event_type,
            posting_id: Some(posting_id),
            booking_award_id,
            target_url: None,
            payload,
            correlation_id: None,
        },
    )
    .await
}

fn conversation_workspace_select_sql() -> &'static str {
    r#"
        SELECT
            c.id,
            c.load_leg_id,
            ll.leg_code AS load_leg_code,
            c.shipper_id,
            shipper.name AS shipper_name,
            c.carrier_id,
            carrier.name AS carrier_name,
            (
                SELECT m.body
                FROM messages m
                WHERE m.conversation_id = c.id
                ORDER BY m.id DESC
                LIMIT 1
            ) AS last_message_body,
            (
                SELECT m.user_id
                FROM messages m
                WHERE m.conversation_id = c.id
                ORDER BY m.id DESC
                LIMIT 1
            ) AS last_message_user_id,
            COALESCE(
                (
                    SELECT m.created_at
                    FROM messages m
                    WHERE m.conversation_id = c.id
                    ORDER BY m.id DESC
                    LIMIT 1
                ),
                c.updated_at
            ) AS last_activity_at,
            (
                SELECT COUNT(*)
                FROM messages m
                WHERE m.conversation_id = c.id
            ) AS message_count,
            (
                SELECT COUNT(*)
                FROM offers o
                WHERE o.conversation_id = c.id OR (o.conversation_id IS NULL AND o.load_leg_id = c.load_leg_id)
            ) AS offer_count
        FROM conversations c
        INNER JOIN load_legs ll ON ll.id = c.load_leg_id AND ll.deleted_at IS NULL
        INNER JOIN users shipper ON shipper.id = c.shipper_id
        INNER JOIN users carrier ON carrier.id = c.carrier_id
    "#
}
