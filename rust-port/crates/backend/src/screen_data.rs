use std::collections::HashMap;

use db::dispatch::{
    list_load_board_legs_filtered, list_load_board_legs_for_carrier_filtered,
    list_load_board_legs_for_owner_filtered, load_board_metrics, load_board_metrics_for_carrier,
    load_board_metrics_for_owner, load_board_tab_counts, load_board_tab_counts_for_carrier,
    load_board_tab_counts_for_owner,
};
use db::marketplace::{
    ConversationPresenceRecord, ConversationReadRecord, OfferRecord,
    count_unread_messages_for_conversation, find_active_peer_presence,
    find_conversation_read_state, find_conversation_workspace_record_for_user,
    find_peer_conversation_read_state, list_message_details_for_conversation, list_offers_for_leg,
    list_recent_conversation_workspace_records_for_user,
};
use db::tms::{
    count_handoffs_by_status, list_recent_handoffs_filtered,
    list_recent_reconciliation_logs_filtered, list_unresolved_sync_error_breakdown,
    list_unresolved_sync_errors, published_mismatch_counts,
};
use domain::auth::UserRole;
use domain::dispatch::LegacyLoadLegStatusCode;
use domain::marketplace::OfferStatus;
use domain::tms::reconciliation_action_descriptors;
use shared::{
    ChatConversationItem, ChatMessageItem, ChatOfferItem, ChatWorkspaceScreen, ErrorBreakdownRow,
    HandoffRow, LoadBoardMetric, LoadBoardRow, LoadBoardScreen, LoadBoardTab, MismatchCard,
    Pagination, ReconciliationLogRow, StatusCard, StloadsOperationsScreen,
    StloadsReconciliationScreen, SyncIssueRow, SyncIssueSummary, sample_stloads_operations_screen,
    sample_stloads_reconciliation_screen,
};
use tracing::warn;

use crate::{auth_session::ResolvedSession, state::AppState};

const STATUS_ORDER: &[(&str, &str, &str, &str)] = &[
    (
        "queued",
        "Queued",
        "warning",
        "Awaiting the next publish cycle.",
    ),
    (
        "push_in_progress",
        "In Progress",
        "info",
        "Currently publishing or republishing to STLOADS.",
    ),
    (
        "published",
        "Published",
        "success",
        "Live on the board and visible to carriers.",
    ),
    (
        "push_failed",
        "Failed",
        "danger",
        "Publish attempt failed and needs operator attention.",
    ),
    (
        "requeue_required",
        "Requeue",
        "primary",
        "Payload drift detected and a retry is needed.",
    ),
    (
        "withdrawn",
        "Withdrawn",
        "secondary",
        "Removed from the board after cancellation or closeout.",
    ),
    (
        "closed",
        "Closed",
        "dark",
        "Archived handoffs kept for audit and reconciliation history.",
    ),
];

const LOAD_BOARD_TAB_ORDER: &[(&str, &str)] = &[
    ("all", "All Loads"),
    ("recommended", "Recommended"),
    ("booked", "Booked"),
];

pub async fn load_board_screen(
    state: &AppState,
    viewer: Option<&ResolvedSession>,
    tab_filter: Option<String>,
) -> LoadBoardScreen {
    let active_tab = normalize_load_board_tab(tab_filter.as_deref());
    let Some(viewer) = viewer else {
        return empty_load_board_screen(
            state,
            "Secure Load Board",
            "Marketplace access requires a Rust session.",
            vec![
                "Sign in before viewing dispatch inventory from the Rust port.".into(),
                "This screen intentionally avoids sample marketplace data during staged cutover."
                    .into(),
            ],
            Some(("Open Rust Login".into(), "/auth/login".into())),
            active_tab,
        );
    };

    if !can_access_load_board(viewer) {
        return empty_load_board_screen(
            state,
            "Secure Load Board",
            viewer_role_workspace(viewer),
            vec![
                "The authenticated account does not have load-board access in the Rust slice."
                    .into(),
            ],
            None,
            active_tab,
        );
    }

    let Some(pool) = state.pool.as_ref() else {
        return empty_load_board_screen(
            state,
            "Manage Loads",
            viewer_role_workspace(viewer),
            vec![format!(
                "Load board data is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            )],
            None,
            active_tab,
        );
    };

    match build_load_board_screen(state, pool, viewer, active_tab.clone()).await {
        Ok(screen) => screen,
        Err(error) => {
            warn!(error = %error, "failed to build auth-scoped load board screen");
            empty_load_board_screen(
                state,
                "Manage Loads",
                viewer_role_workspace(viewer),
                vec![format!(
                    "The Rust load board could not be loaded for this session: {}",
                    error
                )],
                None,
                active_tab,
            )
        }
    }
}

pub async fn chat_workspace_screen(
    state: &AppState,
    viewer: Option<&ResolvedSession>,
    conversation_id: Option<i64>,
) -> ChatWorkspaceScreen {
    let Some(viewer) = viewer else {
        return empty_chat_workspace_screen(
            state,
            vec![
                "Sign in before opening private chat from the Rust port.".into(),
                "This screen intentionally avoids sample conversation data during staged cutover."
                    .into(),
            ],
        );
    };

    if !can_access_chat_workspace(viewer) {
        return empty_chat_workspace_screen(
            state,
            vec!["The authenticated account does not have marketplace chat access in the Rust slice.".into()],
        );
    }

    let Some(pool) = state.pool.as_ref() else {
        return empty_chat_workspace_screen(
            state,
            vec![format!(
                "Chat data is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            )],
        );
    };

    match build_chat_workspace_screen(state, pool, viewer, conversation_id).await {
        Ok(screen) => screen,
        Err(error) => {
            warn!(error = %error, "failed to build auth-scoped chat workspace screen");
            empty_chat_workspace_screen(
                state,
                vec![format!(
                    "The Rust chat workspace could not be loaded for this session: {}",
                    error
                )],
            )
        }
    }
}

pub async fn stloads_operations_screen(
    state: &AppState,
    status_filter: Option<String>,
) -> StloadsOperationsScreen {
    match state.pool.as_ref() {
        Some(pool) => match build_stloads_operations_screen(pool, status_filter.clone()).await {
            Ok(screen) => screen,
            Err(error) => {
                warn!(error = %error, "failed to build DB-backed STLOADS operations screen; serving fallback sample");
                fallback_operations_screen(state, status_filter, Some(error.to_string()))
            }
        },
        None => fallback_operations_screen(state, status_filter, None),
    }
}

pub async fn stloads_reconciliation_screen(
    state: &AppState,
    action_filter: Option<String>,
) -> StloadsReconciliationScreen {
    match state.pool.as_ref() {
        Some(pool) => {
            match build_stloads_reconciliation_screen(pool, action_filter.clone()).await {
                Ok(screen) => screen,
                Err(error) => {
                    warn!(error = %error, "failed to build DB-backed STLOADS reconciliation screen; serving fallback sample");
                    fallback_reconciliation_screen(state, action_filter, Some(error.to_string()))
                }
            }
        }
        None => fallback_reconciliation_screen(state, action_filter, None),
    }
}

async fn build_load_board_screen(
    state: &AppState,
    pool: &db::DbPool,
    viewer: &ResolvedSession,
    active_tab: String,
) -> Result<LoadBoardScreen, sqlx::Error> {
    let viewer_role = viewer.user.primary_role();
    let (tab_counts, metrics, rows, role_label, mut recommendation_notes) = match viewer_role {
        Some(UserRole::Admin) => (
            load_board_tab_counts(pool).await?,
            load_board_metrics(pool).await?,
            list_load_board_legs_filtered(pool, Some(active_tab.as_str()), 20).await?,
            "Admin Workspace".to_string(),
            vec![
                "This load board is globally scoped because the authenticated session has admin visibility.".into(),
                "Realtime delivery is still narrower than read visibility so operator refresh remains the safest source of truth during cutover.".into(),
            ],
        ),
        Some(UserRole::Carrier) => (
            load_board_tab_counts_for_carrier(pool, viewer.user.id).await?,
            load_board_metrics_for_carrier(pool, viewer.user.id).await?,
            list_load_board_legs_for_carrier_filtered(pool, viewer.user.id, Some(active_tab.as_str()), 20)
                .await?,
            "Carrier Workspace".to_string(),
            vec![
                "This load board is scoped to open board inventory plus legs already booked by the authenticated carrier account.".into(),
                "Carrier booking updates are broadcast only to carrier sessions and direct stakeholders during staged cutover.".into(),
            ],
        ),
        Some(UserRole::Shipper) | Some(UserRole::Broker) | Some(UserRole::FreightForwarder) => (
            load_board_tab_counts_for_owner(pool, viewer.user.id).await?,
            load_board_metrics_for_owner(pool, viewer.user.id).await?,
            list_load_board_legs_for_owner_filtered(pool, viewer.user.id, Some(active_tab.as_str()), 20)
                .await?,
            viewer_role_workspace(viewer),
            vec![
                "This load board is scoped to loads owned by the authenticated account.".into(),
                "Offer review and booking refreshes are now restricted to the matching load stakeholders.".into(),
            ],
        ),
        None => {
            return Ok(empty_load_board_screen(
                state,
                "Manage Loads",
                "Secure Workspace",
                vec!["The authenticated account has no mapped Rust role, so the load board stays locked.".into()],
                None,
                active_tab,
            ));
        }
    };

    let count_map: HashMap<String, u64> = tab_counts
        .into_iter()
        .map(|row| (row.tab_key, row.total.max(0) as u64))
        .collect();

    let tabs = LOAD_BOARD_TAB_ORDER
        .iter()
        .map(|(key, label)| LoadBoardTab {
            key: (*key).to_string(),
            label: (*label).to_string(),
            count: count_map.get(*key).copied().unwrap_or(0),
            is_active: active_tab == *key,
        })
        .collect::<Vec<_>>();

    let metrics = vec![
        LoadBoardMetric {
            label: "Open Board".into(),
            value: format!("{} legs", metrics.open_board_total.max(0)),
            note: "Visible for booking or assignment within this authenticated scope.".into(),
        },
        LoadBoardMetric {
            label: "Recommended Matches".into(),
            value: format!("{} legs", metrics.recommended_total.max(0)),
            note: "Still heuristic until carrier preferences and owner-specific filters are fully ported.".into(),
        },
        LoadBoardMetric {
            label: "Funding Watch".into(),
            value: format!("{} legs", metrics.funding_watch_total.max(0)),
            note: "Booked or executing legs that still need escrow follow-up in this authenticated scope.".into(),
        },
    ];

    let rows = rows
        .into_iter()
        .map(|row| {
            let status_label = load_leg_status_label(row.status_id);
            let status_tone = load_leg_status_tone(row.status_id).to_string();
            let recommended_score = recommendation_score(
                row.status_id,
                row.price,
                row.pickup_date.as_ref(),
                row.stloads_alert_title.is_none(),
            );
            let amount_label = format_currency(row.booked_amount.or(row.price));
            let payment_label = payment_label(
                row.escrow_status.as_deref(),
                row.booked_carrier_id.is_some() || row.status_id >= 4,
            );
            let primary_action_label = load_board_primary_action(
                row.status_id,
                row.booked_carrier_id.is_some(),
                row.escrow_status.as_deref(),
                row.stloads_alert_title.as_deref(),
            );

            LoadBoardRow {
                leg_id: row.leg_id.max(0) as u64,
                leg_code: row.leg_code.clone().unwrap_or_else(|| {
                    format!(
                        "{}-{}",
                        row.load_number
                            .unwrap_or_else(|| format!("LOAD-{}", row.load_id)),
                        row.leg_no
                    )
                }),
                origin_label: row
                    .pickup_location_name
                    .unwrap_or_else(|| "Unknown pickup".into()),
                destination_label: row
                    .delivery_location_name
                    .unwrap_or_else(|| "Unknown delivery".into()),
                pickup_date_label: format_date(row.pickup_date.as_ref()),
                delivery_date_label: format_date(row.delivery_date.as_ref()),
                status_label,
                status_tone,
                stloads_label: row
                    .stloads_status
                    .as_ref()
                    .map(|value| title_case_legacy_label(value)),
                stloads_tone: row
                    .stloads_status
                    .as_deref()
                    .map(|value| handoff_status_tone(value).to_string()),
                stloads_alert: row.stloads_alert_title.clone(),
                remarks_label: Some(row.load_title),
                carrier_label: row.booked_carrier_name,
                booked_carrier_id: row.booked_carrier_id.map(|value| value.max(0) as u64),
                bid_status_label: row
                    .bid_status
                    .as_ref()
                    .map(|value| title_case_legacy_label(value))
                    .unwrap_or_else(|| "Open".into()),
                amount_label,
                payment_label,
                recommended_score,
                primary_action_label,
            }
        })
        .collect::<Vec<_>>();

    if let Some(public_base_url) = state.config.public_base_url.as_ref() {
        recommendation_notes.push(format!(
            "IBM deployment note: PUBLIC_BASE_URL is set to {} so auth-scoped booking actions and websocket upgrades stay proxy-safe.",
            public_base_url
        ));
    }

    Ok(LoadBoardScreen {
        title: "Manage Loads".into(),
        role_label,
        primary_action_label: None,
        primary_action_href: None,
        tabs,
        metrics,
        rows,
        recommendation_notes,
        pagination: Pagination {
            page: 1,
            per_page: 20,
            total: count_map.get(active_tab.as_str()).copied().unwrap_or(0),
        },
    })
}

async fn build_chat_workspace_screen(
    state: &AppState,
    pool: &db::DbPool,
    viewer: &ResolvedSession,
    requested_conversation_id: Option<i64>,
) -> Result<ChatWorkspaceScreen, sqlx::Error> {
    let viewer_user_id = viewer.user.id;
    let viewer_role = viewer.user.primary_role();
    let mut conversations =
        list_recent_conversation_workspace_records_for_user(pool, viewer_user_id, viewer_role, 25)
            .await?;

    let active_conversation = match requested_conversation_id {
        Some(requested_id) => {
            if let Some(found) = conversations
                .iter()
                .find(|item| item.id == requested_id)
                .cloned()
            {
                Some(found)
            } else {
                find_conversation_workspace_record_for_user(
                    pool,
                    requested_id,
                    viewer_user_id,
                    viewer_role,
                )
                .await?
            }
        }
        None => conversations.first().cloned(),
    };

    let Some(active_conversation) = active_conversation else {
        let mut notes = vec![
            "No authorized conversations exist yet for the authenticated account, so the Rust workspace is returning an empty chat shell.".into(),
            "This route now stays session-scoped rather than falling back to shared sample conversation data.".into(),
        ];

        if let Some(public_base_url) = state.config.public_base_url.as_ref() {
            notes.push(format!(
                "IBM deployment note: PUBLIC_BASE_URL is set to {} for proxy-safe websocket upgrades during staged cutover.",
                public_base_url
            ));
        }

        return Ok(ChatWorkspaceScreen {
            title: "Private Chat".into(),
            active_conversation_id: None,
            active_participant: "No conversations yet".into(),
            active_participant_user_id: None,
            active_participant_presence_label: None,
            active_participant_presence_tone: None,
            active_participant_last_read_label: None,
            active_load_leg: "n/a".into(),
            composer_user_id: Some(viewer_user_id.max(0) as u64),
            smart_offer_label: "No active offers".into(),
            smart_offer_tone: "info".into(),
            conversations: Vec::new(),
            messages: Vec::new(),
            offers: Vec::new(),
            notes,
        });
    };

    if !conversations
        .iter()
        .any(|item| item.id == active_conversation.id)
    {
        conversations.insert(0, active_conversation.clone());
    }

    let mut messages =
        list_message_details_for_conversation(pool, active_conversation.id, 50).await?;
    messages.sort_by_key(|row| row.id);
    let offers = list_offers_for_leg(pool, active_conversation.load_leg_id).await?;
    let viewer_read_state =
        find_conversation_read_state(pool, active_conversation.id, viewer_user_id).await?;
    let peer_read_state =
        find_peer_conversation_read_state(pool, active_conversation.id, viewer_user_id).await?;
    let active_peer_presence =
        find_active_peer_presence(pool, active_conversation.id, viewer_user_id, 45).await?;
    let (smart_offer_label, smart_offer_tone) = smart_offer_summary(&offers);
    let active_participant_user_id = if viewer_user_id == active_conversation.carrier_id {
        active_conversation.shipper_id
    } else {
        active_conversation.carrier_id
    };
    let last_outgoing_message_id = messages
        .iter()
        .rev()
        .find(|message| message.user_id == viewer_user_id)
        .map(|message| message.id);

    let mut conversation_items = Vec::new();
    for item in conversations {
        let participant_name = if viewer_user_id == item.carrier_id {
            item.shipper_name.clone()
        } else {
            item.carrier_name.clone()
        };
        let participant_user_id = if viewer_user_id == item.carrier_id {
            item.shipper_id
        } else {
            item.carrier_id
        };
        let unread_count =
            count_unread_messages_for_conversation(pool, item.id, viewer_user_id).await?;
        let (presence_label, presence_tone) = if item.id == active_conversation.id {
            peer_presence_badge(active_peer_presence.as_ref(), peer_read_state.as_ref())
        } else {
            (None, None)
        };

        conversation_items.push(ChatConversationItem {
            id: item.id.max(0) as u64,
            participant_user_id: participant_user_id.max(0) as u64,
            participant_name: participant_name.clone(),
            participant_initials: initials(&participant_name),
            load_leg_code: item
                .load_leg_code
                .clone()
                .unwrap_or_else(|| format!("Leg #{}", item.load_leg_id)),
            last_message_preview: preview_message(item.last_message_body.as_deref()),
            last_seen_label: format_datetime(&item.last_activity_at),
            unread_count,
            presence_label,
            presence_tone,
            is_active: item.id == active_conversation.id,
        });
    }

    let message_items = messages
        .into_iter()
        .map(|message| {
            let (receipt_label, receipt_tone) = outgoing_receipt_badge(
                message.id,
                message.user_id == viewer_user_id,
                last_outgoing_message_id,
                peer_read_state.as_ref(),
                active_peer_presence.as_ref(),
            );

            ChatMessageItem {
                id: message.id.max(0) as u64,
                author_user_id: message.user_id.max(0) as u64,
                author_name: if message.user_id == viewer_user_id {
                    "You".into()
                } else {
                    message.author_name
                },
                sent_at_label: format_datetime(&message.created_at),
                body: preview_message(message.body.as_deref()),
                direction: if message.user_id == viewer_user_id {
                    "outgoing".into()
                } else {
                    "incoming".into()
                },
                receipt_label,
                receipt_tone,
            }
        })
        .collect::<Vec<_>>();

    let offer_items = offers
        .iter()
        .map(|offer| {
            let (status_label, status_tone) = offer_status_badge(offer.status_id);
            ChatOfferItem {
                offer_id: offer.id.max(0) as u64,
                amount_label: format_currency(Some(offer.amount)),
                status_label,
                status_tone: status_tone.to_string(),
                created_at_label: format!("Submitted {}", format_datetime(&offer.created_at)),
                can_accept: matches!(offer.status(), Some(OfferStatus::Pending)),
            }
        })
        .collect::<Vec<_>>();

    let (active_participant_presence_label, active_participant_presence_tone) =
        peer_presence_badge(active_peer_presence.as_ref(), peer_read_state.as_ref());
    let active_participant_last_read_label = peer_last_read_label(peer_read_state.as_ref());

    let mut notes = vec![
        "This workspace now pulls only conversations authorized for the current Rust session.".into(),
        "Read receipts and presence are now backed by conversation-scoped SQLx tables plus targeted websocket events.".into(),
    ];

    if viewer_read_state.is_some() {
        notes.push(
            "The current session already has a persisted read cursor for this conversation.".into(),
        );
    } else {
        notes.push(
            "The first open of each conversation seeds a read cursor so unread counts stay session-aware.".into(),
        );
    }

    if let Some(public_base_url) = state.config.public_base_url.as_ref() {
        notes.push(format!(
            "IBM deployment note: PUBLIC_BASE_URL is set to {} so the realtime transport can sit cleanly behind an IBM reverse proxy.",
            public_base_url
        ));
    }

    Ok(ChatWorkspaceScreen {
        title: "Private Chat".into(),
        active_conversation_id: Some(active_conversation.id.max(0) as u64),
        active_participant: if viewer_user_id == active_conversation.carrier_id {
            active_conversation.shipper_name
        } else {
            active_conversation.carrier_name
        },
        active_participant_user_id: Some(active_participant_user_id.max(0) as u64),
        active_participant_presence_label,
        active_participant_presence_tone,
        active_participant_last_read_label,
        active_load_leg: active_conversation
            .load_leg_code
            .unwrap_or_else(|| format!("Leg #{}", active_conversation.load_leg_id)),
        composer_user_id: Some(viewer_user_id.max(0) as u64),
        smart_offer_label,
        smart_offer_tone: smart_offer_tone.to_string(),
        conversations: conversation_items,
        messages: message_items,
        offers: offer_items,
        notes,
    })
}
async fn build_stloads_operations_screen(
    pool: &db::DbPool,
    status_filter: Option<String>,
) -> Result<StloadsOperationsScreen, sqlx::Error> {
    let status_counts = count_handoffs_by_status(pool).await?;
    let handoffs = list_recent_handoffs_filtered(pool, status_filter.as_deref(), 25).await?;
    let unresolved_errors = list_unresolved_sync_errors(pool).await?;

    let count_map: HashMap<String, u64> = status_counts
        .into_iter()
        .map(|row| (row.status, row.total.max(0) as u64))
        .collect();

    let status_cards = STATUS_ORDER
        .iter()
        .map(|(key, label, tone, note)| StatusCard {
            key: (*key).to_string(),
            label: (*label).to_string(),
            value: count_map.get(*key).copied().unwrap_or(0),
            tone: (*tone).to_string(),
            note: Some((*note).to_string()),
            is_active: status_filter.as_deref() == Some(*key),
        })
        .collect::<Vec<_>>();

    let sync_issue_summary = SyncIssueSummary {
        total: unresolved_errors.len() as u64,
        critical: unresolved_errors
            .iter()
            .filter(|row| row.severity == "critical")
            .count() as u64,
        error: unresolved_errors
            .iter()
            .filter(|row| row.severity == "error")
            .count() as u64,
        warning: unresolved_errors
            .iter()
            .filter(|row| row.severity == "warning")
            .count() as u64,
    };

    let recent_sync_issues = unresolved_errors
        .iter()
        .take(10)
        .map(|row| SyncIssueRow {
            id: row.id.max(0) as u64,
            severity: row.severity.clone(),
            error_class: row.error_class.clone(),
            title: row.title.clone(),
            handoff_ref: row.handoff_id.map(|id| format!("#{}", id)),
            created_at_label: format_datetime(&row.created_at),
        })
        .collect::<Vec<_>>();

    let handoff_rows = handoffs
        .into_iter()
        .map(|row| HandoffRow {
            handoff_id: row.id.max(0) as u64,
            handoff_ref: format!("#{}", row.id),
            tms_load_id: row.tms_load_id,
            route_label: format_route(
                row.pickup_city.as_deref(),
                row.pickup_state.as_deref(),
                row.dropoff_city.as_deref(),
                row.dropoff_state.as_deref(),
            ),
            freight_mode: row.freight_mode.unwrap_or_else(|| "Unknown".into()),
            equipment_type: row.equipment_type.unwrap_or_else(|| "Unknown".into()),
            rate_label: format_currency(row.board_rate),
            status_key: row.status.clone(),
            status_label: title_case_legacy_label(&row.status),
            status_tone: handoff_status_tone(&row.status).to_string(),
            load_number: row.load_number,
            retry_count: row.retry_count.max(0) as u64,
            pushed_at_label: format_datetime(&row.created_at),
        })
        .collect::<Vec<_>>();

    let total_records = status_filter
        .as_deref()
        .and_then(|filter| count_map.get(filter).copied())
        .unwrap_or_else(|| count_map.values().copied().sum());

    Ok(StloadsOperationsScreen {
        title: "STLOADS Operations".into(),
        active_filter: status_filter,
        sync_issue_summary,
        status_cards,
        recent_sync_issues,
        handoffs: handoff_rows,
        notes: vec![
            "This screen is now populated from the Rust SQLx layer when DATABASE_URL is configured.".into(),
            "Counts stay card-driven so ops can pivot straight from lifecycle totals into the matching handoff rows.".into(),
        ],
        pagination: Pagination {
            page: 1,
            per_page: 25,
            total: total_records,
        },
    })
}

async fn build_stloads_reconciliation_screen(
    pool: &db::DbPool,
    action_filter: Option<String>,
) -> Result<StloadsReconciliationScreen, sqlx::Error> {
    let mismatch_counts = published_mismatch_counts(pool).await?;
    let error_breakdown = list_unresolved_sync_error_breakdown(pool).await?;
    let normalized_filter = action_filter
        .clone()
        .filter(|value| value != "all" && !value.trim().is_empty());
    let logs =
        list_recent_reconciliation_logs_filtered(pool, normalized_filter.as_deref(), 30).await?;

    let mismatch_cards = vec![
        MismatchCard {
            label: "Published".into(),
            value: mismatch_counts.total_published.max(0) as u64,
            tone: "success".into(),
            note: "All currently active board postings.".into(),
        },
        MismatchCard {
            label: "TMS Cancelled".into(),
            value: mismatch_counts.tms_cancelled.max(0) as u64,
            tone: "danger".into(),
            note: "Cancelled upstream but still visible on STLOADS.".into(),
        },
        MismatchCard {
            label: "TMS Delivered".into(),
            value: mismatch_counts.tms_delivered.max(0) as u64,
            tone: "warning".into(),
            note: "Delivered upstream while the board record remains live.".into(),
        },
        MismatchCard {
            label: "TMS Invoiced/Settled".into(),
            value: mismatch_counts.tms_invoiced.max(0) as u64,
            tone: "info".into(),
            note: "Finance completed upstream before STLOADS archived the posting.".into(),
        },
        MismatchCard {
            label: "No TMS Status".into(),
            value: mismatch_counts.no_tms_status.max(0) as u64,
            tone: "secondary".into(),
            note: "Published records that never received a webhook update.".into(),
        },
        MismatchCard {
            label: "Stale 30d+".into(),
            value: mismatch_counts.stale_30d.max(0) as u64,
            tone: "dark".into(),
            note: "No webhook activity for more than thirty days.".into(),
        },
    ];

    let error_breakdown_rows = error_breakdown
        .into_iter()
        .map(|row| ErrorBreakdownRow {
            error_class: row.error_class,
            severity: row.severity,
            count: row.count.max(0) as u64,
        })
        .collect::<Vec<_>>();

    let log_rows = logs
        .into_iter()
        .map(|row| ReconciliationLogRow {
            id: row.id.max(0) as u64,
            action: row.action.clone(),
            action_tone: reconciliation_action_tone(&row.action).to_string(),
            handoff_ref: Some(format!("#{}", row.handoff_id)),
            tms_transition: transition_label(
                row.tms_status_from.as_deref(),
                row.tms_status_to.as_deref(),
            ),
            stloads_transition: transition_label(
                row.stloads_status_from.as_deref(),
                row.stloads_status_to.as_deref(),
            ),
            detail: row.detail.unwrap_or_else(|| "No detail provided.".into()),
            triggered_by: row.triggered_by.unwrap_or_else(|| "system".into()),
            created_at_label: format_datetime(&row.created_at),
        })
        .collect::<Vec<_>>();

    let mut action_filters = vec!["all".to_string()];
    action_filters.extend(
        reconciliation_action_descriptors()
            .iter()
            .map(|descriptor| descriptor.legacy_label.to_string()),
    );

    Ok(StloadsReconciliationScreen {
        title: "STLOADS Reconciliation".into(),
        mismatch_cards,
        action_filters,
        active_action: Some(action_filter.unwrap_or_else(|| "all".into())),
        error_breakdown: error_breakdown_rows,
        logs: log_rows,
        callouts: vec![
            "This screen now reads mismatch counts, unresolved sync errors, and reconciliation logs from SQLx-backed queries.".into(),
            "Operator action filters are preserved so the staged cutover keeps the same cleanup workflow shape as Laravel.".into(),
        ],
        pagination: Pagination {
            page: 1,
            per_page: 30,
            total: mismatch_counts.total_published.max(0) as u64,
        },
    })
}

fn empty_load_board_screen(
    state: &AppState,
    title: &str,
    role_label: impl Into<String>,
    mut notes: Vec<String>,
    primary_action: Option<(String, String)>,
    active_tab: String,
) -> LoadBoardScreen {
    if let Some(public_base_url) = state.config.public_base_url.as_ref() {
        notes.push(format!(
            "IBM deployment note: PUBLIC_BASE_URL is set to {} so authenticated load-board traffic stays proxy-safe during cutover.",
            public_base_url
        ));
    }

    LoadBoardScreen {
        title: title.into(),
        role_label: role_label.into(),
        primary_action_label: primary_action.as_ref().map(|(label, _)| label.clone()),
        primary_action_href: primary_action.as_ref().map(|(_, href)| href.clone()),
        tabs: LOAD_BOARD_TAB_ORDER
            .iter()
            .map(|(key, label)| LoadBoardTab {
                key: (*key).into(),
                label: (*label).into(),
                count: 0,
                is_active: active_tab == *key,
            })
            .collect(),
        metrics: vec![
            LoadBoardMetric {
                label: "Open Board".into(),
                value: "0 legs".into(),
                note: "No authenticated data is currently available.".into(),
            },
            LoadBoardMetric {
                label: "Recommended Matches".into(),
                value: "0 legs".into(),
                note: "Recommendations will appear after secure data access is available.".into(),
            },
            LoadBoardMetric {
                label: "Funding Watch".into(),
                value: "0 legs".into(),
                note: "Escrow follow-up appears after secure data access is available.".into(),
            },
        ],
        rows: Vec::new(),
        recommendation_notes: notes,
        pagination: Pagination {
            page: 1,
            per_page: 20,
            total: 0,
        },
    }
}

fn empty_chat_workspace_screen(state: &AppState, mut notes: Vec<String>) -> ChatWorkspaceScreen {
    if let Some(public_base_url) = state.config.public_base_url.as_ref() {
        notes.push(format!(
            "IBM deployment note: PUBLIC_BASE_URL is set to {} so chat websocket upgrades stay proxy-safe during cutover.",
            public_base_url
        ));
    }

    ChatWorkspaceScreen {
        title: "Private Chat".into(),
        active_conversation_id: None,
        active_participant: "Secure session required".into(),
        active_participant_user_id: None,
        active_participant_presence_label: None,
        active_participant_presence_tone: None,
        active_participant_last_read_label: None,
        active_load_leg: "n/a".into(),
        composer_user_id: None,
        smart_offer_label: "No active offers".into(),
        smart_offer_tone: "info".into(),
        conversations: Vec::new(),
        messages: Vec::new(),
        offers: Vec::new(),
        notes,
    }
}

fn can_access_load_board(viewer: &ResolvedSession) -> bool {
    has_any_permission(
        viewer,
        &["manage_marketplace", "manage_loads", "manage_dispatch_desk"],
    )
}

fn can_access_chat_workspace(viewer: &ResolvedSession) -> bool {
    has_any_permission(viewer, &["manage_marketplace"])
}

fn has_any_permission(viewer: &ResolvedSession, permission_keys: &[&str]) -> bool {
    viewer.session.permissions.iter().any(|permission| {
        permission_keys
            .iter()
            .any(|expected| permission == expected)
    })
}

fn viewer_role_workspace(viewer: &ResolvedSession) -> String {
    viewer
        .session
        .user
        .as_ref()
        .map(|user| format!("{} Workspace", user.role_label))
        .unwrap_or_else(|| "Secure Workspace".into())
}

fn fallback_operations_screen(
    state: &AppState,
    status_filter: Option<String>,
    error: Option<String>,
) -> StloadsOperationsScreen {
    let mut screen = sample_stloads_operations_screen();
    screen.active_filter = status_filter.or(screen.active_filter);
    screen.notes.insert(
        0,
        format!(
            "Serving sample operations data because the database is {} on {}.",
            state.database_state(),
            state.config.deployment_target
        ),
    );
    if let Some(error) = error {
        screen
            .notes
            .insert(1, format!("Fallback reason: {}", error));
    }
    screen
}

fn fallback_reconciliation_screen(
    state: &AppState,
    action_filter: Option<String>,
    error: Option<String>,
) -> StloadsReconciliationScreen {
    let mut screen = sample_stloads_reconciliation_screen();
    screen.active_action = Some(action_filter.unwrap_or_else(|| "all".into()));
    screen.callouts.insert(
        0,
        format!(
            "Serving sample reconciliation data because the database is {} on {}.",
            state.database_state(),
            state.config.deployment_target
        ),
    );
    if let Some(error) = error {
        screen
            .callouts
            .insert(1, format!("Fallback reason: {}", error));
    }
    screen
}

fn normalize_load_board_tab(value: Option<&str>) -> String {
    match value.unwrap_or("all") {
        "recommended" => "recommended".into(),
        "booked" => "booked".into(),
        _ => "all".into(),
    }
}

fn load_leg_status_label(status_id: i16) -> String {
    match LegacyLoadLegStatusCode::from_legacy_code(status_id) {
        Some(LegacyLoadLegStatusCode::Draft) => "Draft".into(),
        Some(LegacyLoadLegStatusCode::New) => "New".into(),
        Some(LegacyLoadLegStatusCode::Reviewed) => "Reviewed".into(),
        Some(LegacyLoadLegStatusCode::OfferReady) => "Offer Ready".into(),
        Some(LegacyLoadLegStatusCode::Booked) => "Booked".into(),
        Some(LegacyLoadLegStatusCode::PickupStarted) => "Pickup Started".into(),
        Some(LegacyLoadLegStatusCode::AtPickup) => "At Pickup".into(),
        Some(LegacyLoadLegStatusCode::InTransit) => "In Transit".into(),
        Some(LegacyLoadLegStatusCode::EscrowFunded) => "Escrow Funded".into(),
        Some(LegacyLoadLegStatusCode::AtDelivery) => "At Delivery".into(),
        Some(LegacyLoadLegStatusCode::Delivered) => "Delivered".into(),
        Some(LegacyLoadLegStatusCode::PaidOut) => "Paid Out".into(),
        None => format!("Status {}", status_id),
    }
}

fn load_leg_status_tone(status_id: i16) -> &'static str {
    match LegacyLoadLegStatusCode::from_legacy_code(status_id) {
        Some(LegacyLoadLegStatusCode::Draft) => "secondary",
        Some(LegacyLoadLegStatusCode::New) => "warning",
        Some(LegacyLoadLegStatusCode::Reviewed) => "info",
        Some(LegacyLoadLegStatusCode::OfferReady) => "primary",
        Some(LegacyLoadLegStatusCode::Booked) => "primary",
        Some(LegacyLoadLegStatusCode::PickupStarted)
        | Some(LegacyLoadLegStatusCode::AtPickup)
        | Some(LegacyLoadLegStatusCode::InTransit)
        | Some(LegacyLoadLegStatusCode::AtDelivery) => "info",
        Some(LegacyLoadLegStatusCode::EscrowFunded) | Some(LegacyLoadLegStatusCode::Delivered) => {
            "success"
        }
        Some(LegacyLoadLegStatusCode::PaidOut) => "dark",
        None => "secondary",
    }
}

fn recommendation_score(
    status_id: i16,
    price: Option<f64>,
    pickup_date: Option<&chrono::NaiveDateTime>,
    has_no_sync_alert: bool,
) -> Option<u8> {
    if matches!(
        LegacyLoadLegStatusCode::from_legacy_code(status_id),
        Some(
            LegacyLoadLegStatusCode::Booked
                | LegacyLoadLegStatusCode::PickupStarted
                | LegacyLoadLegStatusCode::AtPickup
                | LegacyLoadLegStatusCode::InTransit
                | LegacyLoadLegStatusCode::EscrowFunded
                | LegacyLoadLegStatusCode::AtDelivery
                | LegacyLoadLegStatusCode::Delivered
                | LegacyLoadLegStatusCode::PaidOut
        )
    ) {
        return None;
    }

    let mut score = 55_i32;
    if price.is_some() {
        score += 15;
    }
    if pickup_date.is_some() {
        score += 10;
    }
    if has_no_sync_alert {
        score += 10;
    } else {
        score -= 10;
    }

    Some(score.clamp(0, 98) as u8)
}

fn payment_label(status: Option<&str>, is_booked_or_live: bool) -> String {
    match status {
        Some("released") | Some("paid_out") => "Released".into(),
        Some("funded") => "Funded".into(),
        Some("pending") | Some("hold") => "Escrow pending".into(),
        Some("unfunded") => "Not funded".into(),
        Some(other) => title_case_legacy_label(other),
        None if is_booked_or_live => "Funding setup needed".into(),
        None => "Not funded".into(),
    }
}

fn load_board_primary_action(
    status_id: i16,
    is_booked: bool,
    escrow_status: Option<&str>,
    sync_alert: Option<&str>,
) -> String {
    if sync_alert.is_some() {
        return "Review sync".into();
    }

    if is_booked {
        return match escrow_status {
            Some("released") | Some("paid_out") => "Review payout".into(),
            Some("funded") => "Track leg".into(),
            _ => "Fund escrow".into(),
        };
    }

    match LegacyLoadLegStatusCode::from_legacy_code(status_id) {
        Some(LegacyLoadLegStatusCode::Draft) => "Finish draft".into(),
        Some(LegacyLoadLegStatusCode::New)
        | Some(LegacyLoadLegStatusCode::Reviewed)
        | Some(LegacyLoadLegStatusCode::OfferReady) => "View offers".into(),
        Some(LegacyLoadLegStatusCode::PickupStarted)
        | Some(LegacyLoadLegStatusCode::AtPickup)
        | Some(LegacyLoadLegStatusCode::InTransit)
        | Some(LegacyLoadLegStatusCode::AtDelivery)
        | Some(LegacyLoadLegStatusCode::Delivered) => "Track leg".into(),
        Some(LegacyLoadLegStatusCode::PaidOut) => "Review docs".into(),
        _ => "Open leg".into(),
    }
}

fn preview_message(body: Option<&str>) -> String {
    body.map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
        .unwrap_or_else(|| "Attachment or system event".into())
}

fn initials(name: &str) -> String {
    let initials = name
        .split_whitespace()
        .filter_map(|part| part.chars().next())
        .take(2)
        .collect::<String>();

    if initials.is_empty() {
        "NA".into()
    } else {
        initials.to_uppercase()
    }
}

fn peer_presence_badge(
    presence: Option<&ConversationPresenceRecord>,
    peer_read_state: Option<&ConversationReadRecord>,
) -> (Option<String>, Option<String>) {
    if presence.is_some() {
        (Some("Online now".into()), Some("success".into()))
    } else if peer_read_state.is_some() {
        (Some("Seen recently".into()), Some("secondary".into()))
    } else {
        (None, None)
    }
}

fn peer_last_read_label(peer_read_state: Option<&ConversationReadRecord>) -> Option<String> {
    peer_read_state.and_then(|state| {
        state
            .last_read_message_id
            .map(|_| format!("Read through {}", format_datetime(&state.last_read_at)))
    })
}

fn outgoing_receipt_badge(
    message_id: i64,
    is_outgoing: bool,
    last_outgoing_message_id: Option<i64>,
    peer_read_state: Option<&ConversationReadRecord>,
    peer_presence: Option<&ConversationPresenceRecord>,
) -> (Option<String>, Option<String>) {
    if !is_outgoing || Some(message_id) != last_outgoing_message_id {
        return (None, None);
    }

    if peer_read_state
        .and_then(|state| state.last_read_message_id)
        .map(|last_read_message_id| last_read_message_id >= message_id)
        .unwrap_or(false)
    {
        return (Some("Read".into()), Some("success".into()));
    }

    if peer_presence.is_some() {
        return (Some("Delivered".into()), Some("info".into()));
    }

    (Some("Sent".into()), Some("secondary".into()))
}
fn smart_offer_summary(offers: &[OfferRecord]) -> (String, &'static str) {
    match offers.first().and_then(|offer| offer.status()) {
        Some(OfferStatus::Pending) => ("Pending - awaiting shipper".into(), "warning"),
        Some(OfferStatus::Accepted) => ("Accepted - booking ready".into(), "success"),
        Some(OfferStatus::Declined) => ("Declined - awaiting revision".into(), "danger"),
        None if offers.is_empty() => ("No active offers".into(), "info"),
        None => ("Offer state needs review".into(), "secondary"),
    }
}

fn offer_status_badge(status_id: i16) -> (String, &'static str) {
    match OfferStatus::from_legacy_code(status_id) {
        Some(OfferStatus::Pending) => ("Pending".into(), "warning"),
        Some(OfferStatus::Accepted) => ("Approved".into(), "success"),
        Some(OfferStatus::Declined) => ("Declined".into(), "danger"),
        None => (format!("Status {}", status_id), "secondary"),
    }
}

fn handoff_status_tone(status: &str) -> &'static str {
    match status {
        "queued" => "warning",
        "push_in_progress" => "info",
        "published" => "success",
        "push_failed" => "danger",
        "requeue_required" => "primary",
        "withdrawn" => "secondary",
        "closed" => "dark",
        _ => "secondary",
    }
}

fn reconciliation_action_tone(action: &str) -> &'static str {
    match action {
        "status_update" => "info",
        "auto_withdraw" => "warning",
        "auto_close" => "secondary",
        "auto_archive" => "dark",
        "rate_update" => "primary",
        "mismatch_detected" => "danger",
        "force_sync" => "success",
        _ => "secondary",
    }
}

fn format_route(
    pickup_city: Option<&str>,
    pickup_state: Option<&str>,
    dropoff_city: Option<&str>,
    dropoff_state: Option<&str>,
) -> String {
    let origin = [pickup_city.unwrap_or("Unknown"), pickup_state.unwrap_or("")]
        .into_iter()
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>()
        .join(", ");
    let destination = [
        dropoff_city.unwrap_or("Unknown"),
        dropoff_state.unwrap_or(""),
    ]
    .into_iter()
    .filter(|value| !value.is_empty())
    .collect::<Vec<_>>()
    .join(", ");
    format!("{} -> {}", origin, destination)
}

fn format_currency(value: Option<f64>) -> String {
    value
        .map(|amount| format!("${:.2}", amount))
        .unwrap_or_else(|| "Rate unavailable".into())
}

fn format_date(value: Option<&chrono::NaiveDateTime>) -> String {
    value
        .map(|date| date.format("%b %d, %Y").to_string())
        .unwrap_or_else(|| "TBD".into())
}

fn format_datetime(value: &chrono::NaiveDateTime) -> String {
    value.format("%b %d, %H:%M").to_string()
}

fn title_case_legacy_label(value: &str) -> String {
    value
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn transition_label(from: Option<&str>, to: Option<&str>) -> Option<String> {
    match (from, to) {
        (None, None) => None,
        (Some(from), Some(to)) if from == to => Some(title_case_legacy_label(to)),
        (Some(from), Some(to)) => Some(format!(
            "{} -> {}",
            title_case_legacy_label(from),
            title_case_legacy_label(to)
        )),
        (Some(from), None) => Some(title_case_legacy_label(from)),
        (None, Some(to)) => Some(title_case_legacy_label(to)),
    }
}
