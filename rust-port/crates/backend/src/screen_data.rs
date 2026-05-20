use std::collections::HashMap;

use db::dispatch::{
    LoadBoardSearchFilters, count_dispatch_desk_legs_filtered, list_dispatch_desk_legs_filtered,
    list_load_board_legs_filtered, list_load_board_legs_for_carrier_filtered,
    list_load_board_legs_for_owner_filtered, list_load_board_saved_searches, load_board_metrics,
    load_board_metrics_for_carrier, load_board_metrics_for_owner, load_board_tab_counts,
    load_board_tab_counts_for_carrier, load_board_tab_counts_for_owner,
    search_load_board_for_carrier,
};
use db::marketplace::{
    ConversationPresenceRecord, ConversationReadRecord, OfferRecord,
    count_unread_messages_for_conversation, find_active_peer_presence,
    find_conversation_read_state, find_conversation_workspace_record_for_user,
    find_peer_conversation_read_state, list_message_details_for_conversation, list_offers_for_leg,
    list_recent_conversation_workspace_records_for_user,
};
use db::tms::{
    count_atmp_outbound_events_by_status, count_handoffs_by_status,
    count_unresolved_sync_errors_by_class, list_recent_handoffs_filtered,
    list_recent_reconciliation_logs_filtered, list_unresolved_sync_error_breakdown,
    list_unresolved_sync_errors, published_mismatch_counts,
};
use domain::auth::UserRole;
use domain::dispatch::LegacyLoadLegStatusCode;
use domain::marketplace::OfferStatus;
use domain::tms::reconciliation_action_descriptors;
use shared::{
    ChatConversationItem, ChatMessageItem, ChatOfferItem, ChatWorkspaceScreen, DeadLetterEventRow,
    DispatchDeskLink, DispatchDeskRow, DispatchDeskScreen, ErrorBreakdownRow, HandoffRow,
    LoadBoardFilterState, LoadBoardMetric, LoadBoardRow, LoadBoardSavedSearchItem, LoadBoardScreen,
    LoadBoardTab, MismatchCard, OperationalHealthCard, Pagination, ReconciliationLogRow,
    StatusCard, StloadsOperationsScreen, StloadsReconciliationScreen, SyncIssueRow,
    SyncIssueSummary, load_board_sort_options, load_board_visibility_options,
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

const DISPATCH_DESK_ORDER: &[(&str, &str)] = &[
    ("quote", "Quote Desk"),
    ("tender", "Tender Desk"),
    ("facility", "Facility Desk"),
    ("closeout", "Closeout Desk"),
    ("collections", "Collections Desk"),
];

pub async fn load_board_screen(
    state: &AppState,
    viewer: Option<&ResolvedSession>,
    tab_filter: Option<String>,
    search_filters: LoadBoardSearchFilters,
) -> LoadBoardScreen {
    let active_tab = normalize_load_board_tab(tab_filter.as_deref());
    let Some(viewer) = viewer else {
        return empty_load_board_screen(
            state,
            "Marketplace Loads",
            "Sign in to view marketplace freight.",
            vec!["Sign in to view available marketplace loads.".into()],
            Some(("Sign in".into(), "/auth/login".into())),
            active_tab,
        );
    };

    if !can_access_load_board(viewer) {
        return empty_load_board_screen(
            state,
            "Secure Load Board",
            viewer_role_workspace(viewer),
            vec!["This account does not have marketplace load-board access.".into()],
            None,
            active_tab,
        );
    }

    let Some(pool) = state.pool.as_ref() else {
        return empty_load_board_screen(
            state,
            "Marketplace Loads",
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

    match build_load_board_screen(state, pool, viewer, active_tab.clone(), search_filters).await {
        Ok(screen) => screen,
        Err(error) => {
            warn!(error = %error, "failed to build auth-scoped load board screen");
            empty_load_board_screen(
                state,
                "Manage Loads",
                viewer_role_workspace(viewer),
                vec![format!("Marketplace loads could not be loaded: {}", error)],
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
            vec!["Sign in to open private marketplace chat.".into()],
        );
    };

    if !can_access_chat_workspace(viewer) {
        return empty_chat_workspace_screen(
            state,
            vec!["This account does not have marketplace chat access.".into()],
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
                vec![format!("Private chat could not be loaded: {}", error)],
            )
        }
    }
}

pub async fn dispatch_desk_screen(
    state: &AppState,
    viewer: Option<&ResolvedSession>,
    desk_key: Option<String>,
) -> DispatchDeskScreen {
    let active_desk = normalize_dispatch_desk_key(desk_key.as_deref());
    let Some(viewer) = viewer else {
        return empty_dispatch_desk_screen(
            state,
            &active_desk,
            "Secure Dispatch Desk",
            "Sign in to open marketplace desk work.",
            vec![
                "Sign in before opening quote, tender, facility, closeout, or collections boards."
                    .into(),
            ],
        );
    };

    if !can_access_dispatch_desk(viewer) {
        let workspace = viewer_role_workspace(viewer);
        return empty_dispatch_desk_screen(
            state,
            &active_desk,
            desk_title(&active_desk),
            &workspace,
            vec!["This account does not have dispatch-desk access.".into()],
        );
    }

    let Some(pool) = state.pool.as_ref() else {
        let workspace = viewer_role_workspace(viewer);
        return empty_dispatch_desk_screen(
            state,
            &active_desk,
            desk_title(&active_desk),
            &workspace,
            vec![format!(
                "Dispatch desk data is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            )],
        );
    };

    match build_dispatch_desk_screen(state, pool, viewer, &active_desk).await {
        Ok(screen) => screen,
        Err(error) => {
            let workspace = viewer_role_workspace(viewer);
            warn!(error = %error, "failed to build auth-scoped dispatch desk screen");
            empty_dispatch_desk_screen(
                state,
                &active_desk,
                desk_title(&active_desk),
                &workspace,
                vec![format!("Dispatch desk could not be loaded: {}", error)],
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
                warn!(error = %error, "failed to build DB-backed STLOADS operations screen; serving empty fallback");
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
                    warn!(error = %error, "failed to build DB-backed STLOADS reconciliation screen; serving empty fallback");
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
    search_filters: LoadBoardSearchFilters,
) -> Result<LoadBoardScreen, sqlx::Error> {
    let viewer_role = viewer.user.primary_role();
    let (tab_counts, metrics, rows, role_label, recommendation_notes) = match viewer_role {
        Some(UserRole::Admin) => (
            load_board_tab_counts(pool).await?,
            load_board_metrics(pool).await?,
            list_load_board_legs_filtered(pool, Some(active_tab.as_str()), 20).await?,
            "Admin Workspace".to_string(),
            vec![
                "Admin view includes all marketplace loads.".into(),
                "Refresh the board when reconciling active marketplace work.".into(),
            ],
        ),
        Some(UserRole::Carrier) => (
            load_board_tab_counts_for_carrier(pool, viewer.user.id).await?,
            load_board_metrics_for_carrier(pool, viewer.user.id).await?,
            if search_filters_has_values(&search_filters) {
                search_load_board_for_carrier(pool, viewer.user.id, &search_filters).await?
            } else {
                list_load_board_legs_for_carrier_filtered(
                    pool,
                    viewer.user.id,
                    Some(active_tab.as_str()),
                    20,
                )
                .await?
            },
            "Carrier Workspace".to_string(),
            vec![
                "This load board is scoped to open board inventory plus legs already booked by the authenticated carrier account.".into(),
                "Carrier booking updates are sent to carrier sessions and direct stakeholders.".into(),
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
                "Marketplace Loads",
                "Secure Workspace",
                vec!["This account does not have a marketplace role assigned.".into()],
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
                load_id: row.load_id.max(0) as u64,
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

    let tenant_id = viewer
        .session
        .tenant_scope
        .as_ref()
        .map(|scope| scope.tenant_id.clone())
        .unwrap_or_else(|| "legacy".into());
    let saved_searches = list_load_board_saved_searches(pool, &tenant_id, viewer.user.id)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|row| LoadBoardSavedSearchItem {
            id: row.id.max(0) as u64,
            name: row.name,
            alert_enabled: row.alert_enabled,
            updated_at_label: format_date(Some(&row.updated_at)),
        })
        .collect::<Vec<_>>();

    Ok(LoadBoardScreen {
        title: "Marketplace Loads".into(),
        role_label,
        primary_action_label: None,
        primary_action_href: None,
        filters: load_board_filter_state(&search_filters),
        sort_options: load_board_sort_options(),
        visibility_options: load_board_visibility_options(),
        saved_searches,
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
    _state: &AppState,
    pool: &db::DbPool,
    viewer: &ResolvedSession,
    requested_conversation_id: Option<i64>,
) -> Result<ChatWorkspaceScreen, sqlx::Error> {
    let viewer_user_id = viewer.user.id;
    let viewer_role = viewer.user.primary_role();
    let tenant_id = viewer
        .session
        .tenant_scope
        .as_ref()
        .map(|scope| scope.tenant_id.as_str())
        .unwrap_or("legacy");
    let mut conversations = list_recent_conversation_workspace_records_for_user(
        pool,
        tenant_id,
        viewer_user_id,
        viewer_role,
        25,
    )
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
                    tenant_id,
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
        let notes = vec![
            "No conversations are available for this account.".into(),
            "Marketplace messages will appear after an offer, booking, or support thread starts."
                .into(),
        ];

        return Ok(ChatWorkspaceScreen {
            title: "Private Chat".into(),
            active_conversation_id: None,
            active_posting_id: None,
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
    let active_posting_id = find_posting_id_for_leg(pool, active_conversation.load_leg_id).await?;
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
        "Only authorized marketplace conversations are shown.".into(),
        "Read status and presence update automatically when available.".into(),
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

    Ok(ChatWorkspaceScreen {
        title: "Private Chat".into(),
        active_conversation_id: Some(active_conversation.id.max(0) as u64),
        active_posting_id: active_posting_id.map(|value| value.max(0) as u64),
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

async fn build_dispatch_desk_screen(
    _state: &AppState,
    pool: &db::DbPool,
    viewer: &ResolvedSession,
    desk_key: &str,
) -> Result<DispatchDeskScreen, sqlx::Error> {
    let status_ids = dispatch_desk_statuses(desk_key);
    let owner_scope = match viewer.user.primary_role() {
        Some(UserRole::Admin) => None,
        _ => Some(viewer.user.id),
    };
    let rows = list_dispatch_desk_legs_filtered(pool, owner_scope, status_ids, 20).await?;
    let total = count_dispatch_desk_legs_filtered(pool, owner_scope, status_ids).await?;

    let sync_error_count = if desk_key == "collections" {
        count_unresolved_sync_errors_by_class(pool, "delivered_still_open").await?
    } else {
        0
    };

    let status_cards = build_dispatch_desk_status_cards(desk_key, &rows, sync_error_count);
    let notes = vec![
        "Desk work is grouped by quote, tender, facility, closeout, and collections phase.".into(),
        "Admins see all marketplace work; account users see only their authorized loads.".into(),
    ];

    Ok(DispatchDeskScreen {
        desk_key: desk_key.to_string(),
        title: desk_title(desk_key).into(),
        subtitle: desk_subtitle(desk_key).into(),
        desks: DISPATCH_DESK_ORDER
            .iter()
            .map(|(key, label)| DispatchDeskLink {
                key: (*key).to_string(),
                label: (*label).to_string(),
                href: format!("/desk/{}", key),
                is_active: *key == desk_key,
            })
            .collect(),
        quick_links: dispatch_desk_quick_links(desk_key),
        status_cards,
        rows: rows
            .into_iter()
            .map(|row| map_dispatch_desk_row(desk_key, row))
            .collect(),
        notes,
        pagination: Pagination {
            page: 1,
            per_page: 20,
            total: total.max(0) as u64,
        },
    })
}

async fn find_posting_id_for_leg(
    pool: &db::DbPool,
    load_leg_id: i64,
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        "SELECT id FROM stloads_postings WHERE source_leg_id = $1 AND deleted_at IS NULL ORDER BY updated_at DESC, id DESC LIMIT 1",
    )
    .bind(load_leg_id.to_string())
    .fetch_optional(pool)
    .await
}
async fn build_stloads_operations_screen(
    pool: &db::DbPool,
    status_filter: Option<String>,
) -> Result<StloadsOperationsScreen, sqlx::Error> {
    let status_counts = count_handoffs_by_status(pool).await?;
    let handoffs = list_recent_handoffs_filtered(pool, status_filter.as_deref(), 25).await?;
    let unresolved_errors = list_unresolved_sync_errors(pool).await?;
    let health_cards = list_operational_health_cards(pool).await?;
    let dead_letter_events = list_unresolved_dead_letter_events(pool, 10).await?;

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
        health_cards,
        status_cards,
        recent_sync_issues,
        dead_letter_events,
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

async fn list_operational_health_cards(
    pool: &db::DbPool,
) -> Result<Vec<OperationalHealthCard>, sqlx::Error> {
    let queue_depth = query_count(
        pool,
        "SELECT COUNT(*) FROM atmp_outbound_events WHERE status IN ('queued', 'retry', 'processing')",
    )
    .await?;
    let webhook_failures = query_count(
        pool,
        "SELECT COUNT(*) FROM atmp_inbound_events WHERE validation_status IN ('failed', 'invalid') OR failed_at IS NOT NULL",
    )
    .await?;
    let stale_postings = query_count(
        pool,
        "SELECT COUNT(*) FROM stloads_handoffs WHERE status = 'published' AND created_at < CURRENT_TIMESTAMP - INTERVAL '30 days' AND (last_webhook_at IS NULL OR last_webhook_at < CURRENT_TIMESTAMP - INTERVAL '30 days')",
    )
    .await?;
    let stale_tracking = query_count(
        pool,
        "SELECT COUNT(*) FROM load_legs WHERE status_id IN (5, 6, 7, 9) AND updated_at < CURRENT_TIMESTAMP - INTERVAL '12 hours'",
    )
    .await?;
    let payment_failures = query_count(
        pool,
        "SELECT (SELECT COUNT(*) FROM escrows WHERE status = 'failed') + (SELECT COUNT(*) FROM payment_disputes WHERE status = 'open')",
    )
    .await?;
    let document_failures = query_count(
        pool,
        "SELECT COUNT(*) FROM load_documents WHERE review_status = 'rejected' OR malware_scan_status = 'failed' OR payment_ready_blocked = TRUE",
    )
    .await?;

    Ok(vec![
        health_card(
            "queue_depth",
            "Queue Depth",
            queue_depth,
            "Queued, retrying, or processing ATMP callbacks.",
        ),
        health_card(
            "webhook_failures",
            "Webhook Failures",
            webhook_failures,
            "Inbound contract events that failed validation or processing.",
        ),
        health_card(
            "stale_postings",
            "Stale Postings",
            stale_postings,
            "Published STLOADS handoffs with no recent webhook movement.",
        ),
        health_card(
            "stale_tracking",
            "Stale Tracking",
            stale_tracking,
            "In-progress legs with no recent status movement.",
        ),
        health_card(
            "payment_failures",
            "Payment Failures",
            payment_failures,
            "Failed escrows and open payment disputes.",
        ),
        health_card(
            "document_failures",
            "Document Failures",
            document_failures,
            "Rejected, blocked, or failed document review records.",
        ),
    ])
}

fn health_card(key: &str, label: &str, value: u64, note: &str) -> OperationalHealthCard {
    let tone = if value == 0 {
        "success"
    } else if value < 5 {
        "warning"
    } else {
        "danger"
    };
    OperationalHealthCard {
        key: key.into(),
        label: label.into(),
        value,
        tone: tone.into(),
        note: note.into(),
    }
}

async fn query_count(pool: &db::DbPool, sql: &str) -> Result<u64, sqlx::Error> {
    let count = sqlx::query_scalar::<_, i64>(sql).fetch_one(pool).await?;
    Ok(count.max(0) as u64)
}

async fn list_unresolved_dead_letter_events(
    pool: &db::DbPool,
    limit: i64,
) -> Result<Vec<DeadLetterEventRow>, sqlx::Error> {
    #[derive(sqlx::FromRow)]
    struct DeadLetterRecord {
        id: i64,
        tenant_id: Option<String>,
        source_queue: String,
        event_type: String,
        error_message: String,
        retry_count: i32,
        parked_at: chrono::NaiveDateTime,
    }

    let rows = sqlx::query_as::<_, DeadLetterRecord>(
        "SELECT id, tenant_id, source_queue, event_type, error_message, retry_count, parked_at
         FROM dead_letter_events
         WHERE resolved = FALSE
         ORDER BY parked_at DESC, id DESC
         LIMIT $1",
    )
    .bind(limit.clamp(1, 50))
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| DeadLetterEventRow {
            id: row.id.max(0) as u64,
            tenant_id: row.tenant_id,
            source_queue: row.source_queue,
            event_type: row.event_type,
            error_label: row.error_message,
            retry_count: row.retry_count.max(0) as u64,
            parked_at_label: format_datetime(&row.parked_at),
        })
        .collect())
}

async fn build_stloads_reconciliation_screen(
    pool: &db::DbPool,
    action_filter: Option<String>,
) -> Result<StloadsReconciliationScreen, sqlx::Error> {
    let mismatch_counts = published_mismatch_counts(pool).await?;
    let outbound_statuses = count_atmp_outbound_events_by_status(pool).await?;
    let error_breakdown = list_unresolved_sync_error_breakdown(pool).await?;
    let normalized_filter = action_filter
        .clone()
        .filter(|value| value != "all" && !value.trim().is_empty());
    let logs =
        list_recent_reconciliation_logs_filtered(pool, normalized_filter.as_deref(), 30).await?;

    let outbound_queued = outbound_statuses
        .iter()
        .filter(|row| matches!(row.status.as_str(), "queued" | "retry" | "processing"))
        .map(|row| row.count)
        .sum::<i64>()
        .max(0) as u64;
    let outbound_delivered = outbound_statuses
        .iter()
        .filter(|row| row.status == "delivered")
        .map(|row| row.count)
        .sum::<i64>()
        .max(0) as u64;
    let outbound_dead_letter = outbound_statuses
        .iter()
        .filter(|row| row.status == "dead_letter")
        .map(|row| row.count)
        .sum::<i64>()
        .max(0) as u64;

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
        MismatchCard {
            label: "ATMP Delivered".into(),
            value: outbound_delivered,
            tone: "success".into(),
            note: "Outbound callbacks delivered to ATMP.".into(),
        },
        MismatchCard {
            label: "ATMP Queue".into(),
            value: outbound_queued,
            tone: "primary".into(),
            note: "Queued, processing, or retrying callbacks.".into(),
        },
        MismatchCard {
            label: "ATMP Dead Letter".into(),
            value: outbound_dead_letter,
            tone: "danger".into(),
            note: "Callbacks requiring replay or operator review.".into(),
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
            "Review unresolved marketplace sync issues and recovery activity.".into(),
            "ATMP callback status is shown for queue, retry, delivered, and dead-letter states."
                .into(),
        ],
        pagination: Pagination {
            page: 1,
            per_page: 30,
            total: mismatch_counts.total_published.max(0) as u64,
        },
    })
}

fn empty_load_board_screen(
    _state: &AppState,
    title: &str,
    role_label: impl Into<String>,
    notes: Vec<String>,
    primary_action: Option<(String, String)>,
    active_tab: String,
) -> LoadBoardScreen {
    LoadBoardScreen {
        title: title.into(),
        role_label: role_label.into(),
        primary_action_label: primary_action.as_ref().map(|(label, _)| label.clone()),
        primary_action_href: primary_action.as_ref().map(|(_, href)| href.clone()),
        filters: LoadBoardFilterState::default(),
        sort_options: load_board_sort_options(),
        visibility_options: load_board_visibility_options(),
        saved_searches: Vec::new(),
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

fn empty_dispatch_desk_screen(
    _state: &AppState,
    desk_key: &str,
    title: &str,
    subtitle: &str,
    notes: Vec<String>,
) -> DispatchDeskScreen {
    DispatchDeskScreen {
        desk_key: desk_key.to_string(),
        title: title.to_string(),
        subtitle: subtitle.to_string(),
        desks: DISPATCH_DESK_ORDER
            .iter()
            .map(|(key, label)| DispatchDeskLink {
                key: (*key).to_string(),
                label: (*label).to_string(),
                href: format!("/desk/{}", key),
                is_active: *key == desk_key,
            })
            .collect(),
        quick_links: dispatch_desk_quick_links(desk_key),
        status_cards: Vec::new(),
        rows: Vec::new(),
        notes,
        pagination: Pagination {
            page: 1,
            per_page: 20,
            total: 0,
        },
    }
}

fn empty_chat_workspace_screen(_state: &AppState, notes: Vec<String>) -> ChatWorkspaceScreen {
    ChatWorkspaceScreen {
        title: "Private Chat".into(),
        active_conversation_id: None,
        active_posting_id: None,
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

fn can_access_dispatch_desk(viewer: &ResolvedSession) -> bool {
    has_any_permission(
        viewer,
        &[
            "manage_dispatch_desk",
            "manage_loads",
            "access_admin_portal",
        ],
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

fn normalize_dispatch_desk_key(value: Option<&str>) -> String {
    match value.unwrap_or("quote").trim() {
        "tender" => "tender".into(),
        "facility" => "facility".into(),
        "closeout" => "closeout".into(),
        "collections" => "collections".into(),
        _ => "quote".into(),
    }
}

fn dispatch_desk_statuses(desk_key: &str) -> &'static [i16] {
    match desk_key {
        "tender" => &[1, 4],
        "facility" => &[4, 5, 6],
        "closeout" => &[9, 10],
        "collections" => &[10, 11],
        _ => &[1],
    }
}

fn desk_title(desk_key: &str) -> &'static str {
    match desk_key {
        "tender" => "Tender Desk",
        "facility" => "Facility Desk",
        "closeout" => "Closeout Desk",
        "collections" => "Collections Desk",
        _ => "Quote Desk",
    }
}

fn desk_subtitle(desk_key: &str) -> &'static str {
    match desk_key {
        "tender" => {
            "Loads at tender or booking stage, with duplicate-risk visibility for STLOADS board exposure."
        }
        "facility" => {
            "Loads at pickup and facility-readiness stage, with STLOADS and readiness signals side by side."
        }
        "closeout" => {
            "Delivered or completed loads that still need withdraw, close, or archive follow-up on STLOADS."
        }
        "collections" => {
            "Finance-stage loads that still need STLOADS archive cleanup or sync-error review."
        }
        _ => "Quote-stage loads that are still being priced and reviewed for board eligibility.",
    }
}

fn build_dispatch_desk_status_cards(
    desk_key: &str,
    rows: &[db::dispatch::DispatchDeskLegRecord],
    sync_error_count: i64,
) -> Vec<StatusCard> {
    match desk_key {
        "tender" => vec![
            StatusCard {
                key: "published".into(),
                label: "Board-Exposed".into(),
                value: rows
                    .iter()
                    .filter(|row| row.handoff_status.as_deref() == Some("published"))
                    .count() as u64,
                tone: "success".into(),
                note: Some("Tender-stage loads already visible to carriers on STLOADS.".into()),
                is_active: false,
            },
            StatusCard {
                key: "push_failed".into(),
                label: "Push Failed".into(),
                value: rows
                    .iter()
                    .filter(|row| row.handoff_status.as_deref() == Some("push_failed"))
                    .count() as u64,
                tone: "danger".into(),
                note: Some("Publish attempts that need ops attention before the board can be trusted.".into()),
                is_active: false,
            },
            StatusCard {
                key: "withdrawn".into(),
                label: "Withdrawn".into(),
                value: rows
                    .iter()
                    .filter(|row| row.handoff_status.as_deref() == Some("withdrawn"))
                    .count() as u64,
                tone: "secondary".into(),
                note: Some("Tender records already pulled off STLOADS.".into()),
                is_active: false,
            },
        ],
        "facility" => vec![
            StatusCard {
                key: "published".into(),
                label: "Published to Board".into(),
                value: rows
                    .iter()
                    .filter(|row| row.handoff_status.as_deref() == Some("published"))
                    .count() as u64,
                tone: "success".into(),
                note: Some("Facility-stage legs that still have an active board representation.".into()),
                is_active: false,
            },
            StatusCard {
                key: "no_handoff".into(),
                label: "No STLOADS Handoff".into(),
                value: rows
                    .iter()
                    .filter(|row| row.handoff_status.is_none())
                    .count() as u64,
                tone: "warning".into(),
                note: Some("Pickup-stage legs still operating without an STLOADS handoff.".into()),
                is_active: false,
            },
        ],
        "closeout" => vec![
            StatusCard {
                key: "still_live".into(),
                label: "Still Live on STLOADS".into(),
                value: rows
                    .iter()
                    .filter(|row| {
                        matches!(
                            row.handoff_status.as_deref(),
                            Some("published" | "queued" | "push_in_progress")
                        )
                    })
                    .count() as u64,
                tone: "danger".into(),
                note: Some("Delivered or completed loads that still need withdraw or close follow-up.".into()),
                is_active: false,
            },
            StatusCard {
                key: "withdrawn".into(),
                label: "Withdrawn".into(),
                value: rows
                    .iter()
                    .filter(|row| row.handoff_status.as_deref() == Some("withdrawn"))
                    .count() as u64,
                tone: "secondary".into(),
                note: Some("Closeout records already pulled from STLOADS and waiting on archive decisions.".into()),
                is_active: false,
            },
            StatusCard {
                key: "closed".into(),
                label: "Closed / Archived".into(),
                value: rows
                    .iter()
                    .filter(|row| row.handoff_status.as_deref() == Some("closed"))
                    .count() as u64,
                tone: "dark".into(),
                note: Some("Closeout records already archived downstream.".into()),
                is_active: false,
            },
        ],
        "collections" => vec![
            StatusCard {
                key: "needs_archive".into(),
                label: "Needs STLOADS Archive".into(),
                value: rows
                    .iter()
                    .filter(|row| {
                        row.handoff_status.is_some()
                            && !matches!(
                                row.handoff_status.as_deref(),
                                Some("closed" | "withdrawn")
                            )
                    })
                    .count() as u64,
                tone: "warning".into(),
                note: Some("Finance-stage loads that still look active on STLOADS.".into()),
                is_active: false,
            },
            StatusCard {
                key: "sync_errors".into(),
                label: "Delivered-Still-Open Errors".into(),
                value: sync_error_count.max(0) as u64,
                tone: "danger".into(),
                note: Some("Unresolved STLOADS sync errors with delivered loads still open.".into()),
                is_active: false,
            },
            StatusCard {
                key: "closed".into(),
                label: "Closed / Archived".into(),
                value: rows
                    .iter()
                    .filter(|row| row.handoff_status.as_deref() == Some("closed"))
                    .count() as u64,
                tone: "dark".into(),
                note: Some("Finance-stage handoffs already archived.".into()),
                is_active: false,
            },
        ],
        _ => vec![
            StatusCard {
                key: "eligible".into(),
                label: "Eligible for STLOADS".into(),
                value: rows
                    .iter()
                    .filter(|row| row.handoff_status.is_none())
                    .count() as u64,
                tone: "primary".into(),
                note: Some("Quote-stage loads with no board handoff yet.".into()),
                is_active: false,
            },
            StatusCard {
                key: "published".into(),
                label: "Published to Board".into(),
                value: rows
                    .iter()
                    .filter(|row| row.handoff_status.as_deref() == Some("published"))
                    .count() as u64,
                tone: "success".into(),
                note: Some("Quote-stage loads already visible on STLOADS.".into()),
                is_active: false,
            },
            StatusCard {
                key: "queued".into(),
                label: "Queued for Push".into(),
                value: rows
                    .iter()
                    .filter(|row| row.handoff_status.as_deref() == Some("queued"))
                    .count() as u64,
                tone: "info".into(),
                note: Some("Quote-stage loads waiting for the next board push cycle.".into()),
                is_active: false,
            },
        ],
    }
}

fn map_dispatch_desk_row(
    desk_key: &str,
    row: db::dispatch::DispatchDeskLegRecord,
) -> DispatchDeskRow {
    let (focus_label, focus_tone, focus_note) = match desk_key {
        "tender" => {
            if row.handoff_status.as_deref() == Some("published") && row.booked_carrier_id.is_some()
            {
                (
                    "Carrier Assigned".into(),
                    "warning".into(),
                    Some("Load is booked with a carrier but still published on STLOADS.".into()),
                )
            } else {
                ("-".into(), "secondary".into(), None)
            }
        }
        "facility" => match row.handoff_status.as_deref() {
            Some("published") if matches!(row.status_id, 5 | 6) => (
                "Pickup Active".into(),
                "success".into(),
                Some("Pickup has started or the carrier is already at facility.".into()),
            ),
            Some("published") => (
                "Awaiting Pickup".into(),
                "warning".into(),
                Some("Published to STLOADS, but pickup is not yet active.".into()),
            ),
            None => ("-".into(), "secondary".into(), None),
            Some(other) => (title_case_legacy_label(other), "secondary".into(), None),
        },
        "closeout" => match row.handoff_status.as_deref() {
            Some("published" | "queued" | "push_in_progress") => (
                "Needs Withdraw / Close".into(),
                "danger".into(),
                Some(
                    "Delivery is done internally but the board representation is still live."
                        .into(),
                ),
            ),
            Some("withdrawn") => (
                "Needs Archive".into(),
                "warning".into(),
                Some("Already withdrawn from STLOADS and ready for closeout cleanup.".into()),
            ),
            Some("closed") => ("Archived".into(), "dark".into(), None),
            _ => ("-".into(), "secondary".into(), None),
        },
        "collections" => match row.handoff_status.as_deref() {
            None => ("No handoff".into(), "secondary".into(), None),
            Some("closed") => ("Archived".into(), "dark".into(), None),
            Some("withdrawn") => (
                "Withdrawn - Ready to Close".into(),
                "secondary".into(),
                Some("Collections can close the downstream STLOADS trail.".into()),
            ),
            Some(_) => (
                "Still Active - Archive Required".into(),
                "danger".into(),
                Some("Finance is complete, but STLOADS still looks active.".into()),
            ),
        },
        _ => match row.handoff_status.as_deref() {
            None => ("Eligible".into(), "primary".into(), None),
            Some("published") => ("On Board".into(), "success".into(), None),
            Some(other) => (title_case_legacy_label(other), "secondary".into(), None),
        },
    };
    let (archive_guidance_label, archive_guidance_tone, archive_guidance_note) =
        dispatch_desk_archive_guidance(
            desk_key,
            row.escrow_status.as_deref(),
            row.handoff_status.as_deref(),
        );
    let (primary_action_key, primary_action_label, primary_action_enabled) =
        dispatch_desk_primary_action(
            desk_key,
            row.handoff_status.as_deref(),
            row.booked_carrier_id.is_some(),
            row.handoff_id,
        );
    let (finance_action_key, finance_action_label, finance_action_enabled) =
        dispatch_desk_finance_action(
            desk_key,
            row.status_id,
            row.booked_carrier_id.is_some(),
            row.escrow_status.as_deref(),
            row.handoff_status.as_deref(),
        );
    let payment_label = match desk_key {
        "closeout" | "collections" => Some(payment_label(
            row.escrow_status.as_deref(),
            row.booked_carrier_id.is_some() || row.status_id >= 8,
        )),
        _ => None,
    };
    let (secondary_action_label, secondary_action_href) = dispatch_desk_secondary_action(
        desk_key,
        row.leg_id,
        row.load_id,
        row.status_id,
        row.escrow_status.as_deref(),
        row.handoff_status.as_deref(),
    );

    DispatchDeskRow {
        load_id: row.load_id.max(0) as u64,
        leg_id: row.leg_id.max(0) as u64,
        handoff_id: row.handoff_id.map(|value| value.max(0) as u64),
        load_number: row.load_number,
        title: row.load_title,
        equipment_label: row.equipment_name,
        weight_label: row.weight.map(|value| format!("{:.0}", value)),
        carrier_label: row.booked_carrier_name,
        payment_label,
        leg_status_label: load_leg_status_label(row.status_id),
        leg_status_tone: load_leg_status_tone(row.status_id).into(),
        stloads_label: row
            .handoff_status
            .as_ref()
            .map(|value| title_case_legacy_label(value)),
        stloads_tone: row
            .handoff_status
            .as_deref()
            .map(|value| handoff_status_tone(value).to_string()),
        focus_label,
        focus_tone,
        focus_note,
        archive_guidance_label,
        archive_guidance_tone,
        archive_guidance_note,
        latest_activity_note: row.latest_activity_note,
        load_href: Some(format!("/loads/{}", row.load_id.max(0) as u64)),
        primary_action_key,
        primary_action_label,
        primary_action_enabled,
        finance_action_key,
        finance_action_label,
        finance_action_enabled,
        secondary_action_label,
        secondary_action_href,
    }
}

fn dispatch_desk_archive_guidance(
    desk_key: &str,
    escrow_status: Option<&str>,
    handoff_status: Option<&str>,
) -> (Option<String>, Option<String>, Option<String>) {
    match desk_key {
        "closeout" => match handoff_status {
            Some("published" | "queued" | "push_in_progress") => (
                Some("Still Live On STLOADS".into()),
                Some("danger".into()),
                Some(
                    "Withdraw or close the board listing before the load can be considered fully archived."
                        .into(),
                ),
            ),
            Some("withdrawn") if matches!(escrow_status, Some("released" | "paid_out")) => (
                Some("Ready To Archive".into()),
                Some("success".into()),
                Some(
                    "Finance is complete and the handoff is withdrawn, so downstream closeout can finish cleanly."
                        .into(),
                ),
            ),
            Some("withdrawn") => (
                Some("Archive After Finance".into()),
                Some("warning".into()),
                Some(
                    "The STLOADS listing is withdrawn, but escrow still needs a final finance step before archive is complete."
                        .into(),
                ),
            ),
            Some("closed") => (
                Some("Archived".into()),
                Some("dark".into()),
                Some("Closeout archive is already complete on the Rust side.".into()),
            ),
            None => (
                Some("No Handoff".into()),
                Some("secondary".into()),
                Some("No downstream STLOADS archive step is required for this row.".into()),
            ),
            Some(other) => (
                Some(title_case_legacy_label(other)),
                Some("secondary".into()),
                None,
            ),
        },
        "collections" => match handoff_status {
            None => (
                Some("No Handoff".into()),
                Some("secondary".into()),
                Some("Collections can close locally because no STLOADS publish trail exists.".into()),
            ),
            Some("closed") => (
                Some("Archived".into()),
                Some("dark".into()),
                Some("Finance and archive cleanup are both complete.".into()),
            ),
            Some("withdrawn") if matches!(escrow_status, Some("released" | "paid_out")) => (
                Some("Ready To Close".into()),
                Some("success".into()),
                Some(
                    "Escrow is already finished and the withdrawn handoff can be closed for final archive."
                        .into(),
                ),
            ),
            Some("withdrawn") => (
                Some("Release Then Close".into()),
                Some("warning".into()),
                Some(
                    "Collections still needs the payout step before the withdrawn handoff should be closed."
                        .into(),
                ),
            ),
            Some(_) => (
                Some("Archive Required".into()),
                Some("danger".into()),
                Some(
                    "The board posting still looks active while collections is finishing finance follow-up."
                        .into(),
                ),
            ),
        },
        _ => (None, None, None),
    }
}

fn dispatch_desk_primary_action(
    desk_key: &str,
    handoff_status: Option<&str>,
    has_booked_carrier: bool,
    handoff_id: Option<i64>,
) -> (Option<String>, Option<String>, bool) {
    let has_handoff = handoff_id.is_some();
    match desk_key {
        "tender" => match handoff_status {
            Some("push_failed" | "requeue_required") if has_handoff => {
                (Some("requeue".into()), Some("Requeue".into()), true)
            }
            Some("published") if has_handoff && has_booked_carrier => {
                (Some("withdraw".into()), Some("Withdraw".into()), true)
            }
            _ => (None, None, false),
        },
        "closeout" => match handoff_status {
            Some("published" | "queued" | "push_in_progress") if has_handoff => {
                (Some("withdraw".into()), Some("Withdraw".into()), true)
            }
            Some("withdrawn") if has_handoff => (Some("close".into()), Some("Close".into()), true),
            _ => (None, None, false),
        },
        "collections" => match handoff_status {
            Some("withdrawn") if has_handoff => (Some("close".into()), Some("Close".into()), true),
            Some("published" | "queued" | "push_in_progress") if has_handoff => {
                (Some("close".into()), Some("Force Close".into()), true)
            }
            _ => (None, None, false),
        },
        _ => (None, None, false),
    }
}

fn dispatch_desk_finance_action(
    desk_key: &str,
    status_id: i16,
    has_booked_carrier: bool,
    escrow_status: Option<&str>,
    handoff_status: Option<&str>,
) -> (Option<String>, Option<String>, bool) {
    if !matches!(desk_key, "closeout" | "collections") || !has_booked_carrier {
        return (None, None, false);
    }

    match desk_key {
        "closeout" => {
            if status_id >= 10 && !matches!(escrow_status, Some("released" | "paid_out")) {
                return match escrow_status {
                    Some("funded") => (Some("release".into()), Some("Release Escrow".into()), true),
                    Some("pending" | "hold") => {
                        (Some("hold".into()), Some("Keep On Hold".into()), true)
                    }
                    _ => (Some("fund".into()), Some("Fund Escrow".into()), true),
                };
            }

            if matches!(handoff_status, Some("withdrawn"))
                && matches!(escrow_status, Some("released" | "paid_out"))
            {
                return (Some("release".into()), Some("Review Release".into()), false);
            }

            (None, None, false)
        }
        "collections" => match escrow_status {
            Some("released" | "paid_out") => (None, None, false),
            Some("funded") => (Some("release".into()), Some("Release Escrow".into()), true),
            Some("pending" | "hold") => (Some("hold".into()), Some("Keep On Hold".into()), true),
            _ => (Some("fund".into()), Some("Fund Escrow".into()), true),
        },
        _ => (None, None, false),
    }
}

fn dispatch_desk_secondary_action(
    desk_key: &str,
    leg_id: i64,
    load_id: i64,
    status_id: i16,
    escrow_status: Option<&str>,
    handoff_status: Option<&str>,
) -> (Option<String>, Option<String>) {
    match desk_key {
        "closeout" => {
            if status_id >= 10 && !matches!(escrow_status, Some("released" | "paid_out")) {
                return (
                    Some("Open Payments".into()),
                    Some(format!(
                        "/admin/payments?leg_id={}&action=release&source=dispatch-closeout&load_id={}",
                        leg_id.max(0),
                        load_id.max(0)
                    )),
                );
            }

            if matches!(
                handoff_status,
                Some("published" | "queued" | "push_in_progress" | "withdrawn")
            ) {
                return (
                    Some("Open Reconciliation".into()),
                    Some(format!(
                        "/admin/stloads/reconciliation?action={}",
                        if matches!(handoff_status, Some("withdrawn")) {
                            "auto_archive"
                        } else {
                            "mismatch_detected"
                        }
                    )),
                );
            }

            (None, None)
        }
        "collections" => {
            if !matches!(escrow_status, Some("released" | "paid_out")) {
                return (
                    Some("Open Payments".into()),
                    Some(format!(
                        "/admin/payments?leg_id={}&action={}&source=dispatch-collections&load_id={}",
                        leg_id.max(0),
                        if matches!(escrow_status, Some("funded")) {
                            "release"
                        } else {
                            "fund"
                        },
                        load_id.max(0)
                    )),
                );
            }

            if !matches!(handoff_status, None | Some("closed")) {
                return (
                    Some("Open STLOADS Ops".into()),
                    Some("/admin/stloads/operations".into()),
                );
            }

            (None, None)
        }
        _ => (None, None),
    }
}

fn dispatch_desk_quick_links(desk_key: &str) -> Vec<DispatchDeskLink> {
    match desk_key {
        "closeout" => vec![
            DispatchDeskLink {
                key: "reconciliation".into(),
                label: "Reconciliation".into(),
                href: "/admin/stloads/reconciliation".into(),
                is_active: false,
            },
            DispatchDeskLink {
                key: "stloads_operations".into(),
                label: "STLOADS Operations".into(),
                href: "/admin/stloads/operations".into(),
                is_active: false,
            },
        ],
        "collections" => vec![
            DispatchDeskLink {
                key: "payments".into(),
                label: "Payments Console".into(),
                href: "/admin/payments".into(),
                is_active: false,
            },
            DispatchDeskLink {
                key: "reconciliation".into(),
                label: "Reconciliation".into(),
                href: "/admin/stloads/reconciliation".into(),
                is_active: false,
            },
        ],
        _ => Vec::new(),
    }
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
    let mut screen = empty_operations_screen(status_filter);
    screen.notes.push(format!(
        "Operations data is unavailable because the database is {} on {}.",
        state.database_state(),
        state.config.deployment_target
    ));
    if let Some(error) = error {
        screen.notes.push(format!("Data source error: {}", error));
    }
    screen
}

fn fallback_reconciliation_screen(
    state: &AppState,
    action_filter: Option<String>,
    error: Option<String>,
) -> StloadsReconciliationScreen {
    let mut screen = empty_reconciliation_screen(action_filter.unwrap_or_else(|| "all".into()));
    screen.callouts.push(format!(
        "Reconciliation data is unavailable because the database is {} on {}.",
        state.database_state(),
        state.config.deployment_target
    ));
    if let Some(error) = error {
        screen
            .callouts
            .push(format!("Data source error: {}", error));
    }
    screen
}

fn empty_operations_screen(status_filter: Option<String>) -> StloadsOperationsScreen {
    StloadsOperationsScreen {
        title: "STLOADS Operations".into(),
        active_filter: status_filter.clone(),
        sync_issue_summary: SyncIssueSummary {
            total: 0,
            critical: 0,
            error: 0,
            warning: 0,
        },
        health_cards: vec![
            health_card(
                "queue_depth",
                "Queue Depth",
                0,
                "No queue data is available.",
            ),
            health_card(
                "webhook_failures",
                "Webhook Failures",
                0,
                "No webhook failure data is available.",
            ),
            health_card(
                "stale_postings",
                "Stale Postings",
                0,
                "No stale posting data is available.",
            ),
            health_card(
                "stale_tracking",
                "Stale Tracking",
                0,
                "No stale tracking data is available.",
            ),
            health_card(
                "payment_failures",
                "Payment Failures",
                0,
                "No payment failure data is available.",
            ),
            health_card(
                "document_failures",
                "Document Failures",
                0,
                "No document failure data is available.",
            ),
        ],
        status_cards: STATUS_ORDER
            .iter()
            .map(|(key, label, tone, note)| StatusCard {
                key: (*key).into(),
                label: (*label).into(),
                value: 0,
                tone: (*tone).into(),
                note: Some((*note).into()),
                is_active: status_filter.as_deref() == Some(*key),
            })
            .collect(),
        recent_sync_issues: Vec::new(),
        dead_letter_events: Vec::new(),
        handoffs: Vec::new(),
        notes: vec!["No production operations records are available for this tenant.".into()],
        pagination: Pagination {
            page: 1,
            per_page: 25,
            total: 0,
        },
    }
}

fn empty_reconciliation_screen(active_action: String) -> StloadsReconciliationScreen {
    let mut action_filters = vec!["all".to_string()];
    action_filters.extend(
        reconciliation_action_descriptors()
            .iter()
            .map(|descriptor| descriptor.legacy_label.to_string()),
    );

    StloadsReconciliationScreen {
        title: "STLOADS Reconciliation".into(),
        mismatch_cards: vec![
            MismatchCard {
                label: "Published".into(),
                value: 0,
                tone: "success".into(),
                note: "No active board postings are available.".into(),
            },
            MismatchCard {
                label: "TMS Cancelled".into(),
                value: 0,
                tone: "danger".into(),
                note: "No cancelled upstream mismatches are available.".into(),
            },
            MismatchCard {
                label: "TMS Delivered".into(),
                value: 0,
                tone: "warning".into(),
                note: "No delivered upstream mismatches are available.".into(),
            },
            MismatchCard {
                label: "TMS Invoiced/Settled".into(),
                value: 0,
                tone: "info".into(),
                note: "No finance mismatch data is available.".into(),
            },
            MismatchCard {
                label: "No TMS Status".into(),
                value: 0,
                tone: "secondary".into(),
                note: "No missing upstream status data is available.".into(),
            },
            MismatchCard {
                label: "Stale 30d+".into(),
                value: 0,
                tone: "dark".into(),
                note: "No stale handoff data is available.".into(),
            },
            MismatchCard {
                label: "ATMP Delivered".into(),
                value: 0,
                tone: "success".into(),
                note: "No delivered callback data is available.".into(),
            },
            MismatchCard {
                label: "ATMP Queue".into(),
                value: 0,
                tone: "primary".into(),
                note: "No queued callback data is available.".into(),
            },
            MismatchCard {
                label: "ATMP Dead Letter".into(),
                value: 0,
                tone: "danger".into(),
                note: "No dead-letter callback data is available.".into(),
            },
        ],
        action_filters,
        active_action: Some(active_action),
        error_breakdown: Vec::new(),
        logs: Vec::new(),
        callouts: vec![
            "No production reconciliation records are available for this tenant.".into(),
        ],
        pagination: Pagination {
            page: 1,
            per_page: 30,
            total: 0,
        },
    }
}

fn normalize_load_board_tab(value: Option<&str>) -> String {
    match value.unwrap_or("all") {
        "recommended" => "recommended".into(),
        "booked" => "booked".into(),
        _ => "all".into(),
    }
}

fn search_filters_has_values(filters: &LoadBoardSearchFilters) -> bool {
    filters.origin.is_some()
        || filters.destination.is_some()
        || filters.equipment.is_some()
        || filters.mode.is_some()
        || filters.date_from.is_some()
        || filters.date_to.is_some()
        || filters.min_rate.is_some()
        || filters.max_rate.is_some()
        || filters.min_rpm.is_some()
        || filters.max_rpm.is_some()
        || filters.min_weight.is_some()
        || filters.max_weight.is_some()
        || filters.hazmat.is_some()
        || filters.temperature_controlled.is_some()
        || filters.service_level.is_some()
        || filters.visibility.is_some()
        || filters.sort.is_some()
}

fn load_board_filter_state(filters: &LoadBoardSearchFilters) -> LoadBoardFilterState {
    LoadBoardFilterState {
        origin: filters.origin.clone(),
        destination: filters.destination.clone(),
        equipment: filters.equipment.clone(),
        mode: filters.mode.clone(),
        date_from: filters
            .date_from
            .map(|value| value.format("%Y-%m-%d").to_string()),
        date_to: filters
            .date_to
            .map(|value| value.format("%Y-%m-%d").to_string()),
        min_rate: filters.min_rate.map(|value| value.to_string()),
        max_rate: filters.max_rate.map(|value| value.to_string()),
        min_rpm: filters.min_rpm.map(|value| value.to_string()),
        max_rpm: filters.max_rpm.map(|value| value.to_string()),
        min_weight: filters.min_weight.map(|value| value.to_string()),
        max_weight: filters.max_weight.map(|value| value.to_string()),
        hazmat: filters.hazmat,
        temperature_controlled: filters.temperature_controlled,
        service_level: filters.service_level.clone(),
        visibility: filters.visibility.clone(),
        sort: filters.sort.clone(),
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

#[cfg(test)]
mod tests {
    use tokio::sync::broadcast;

    use crate::{
        config::RuntimeConfig, document_storage::DocumentStorageService, email::EmailService,
        integration_auth::IntegrationAuthState, realtime_bus::RoutedRealtimeEvent, state::AppState,
        stripe::StripeService,
    };

    fn no_pool_state() -> AppState {
        let config = RuntimeConfig {
            bind_addr: "127.0.0.1".into(),
            port: 3001,
            deployment_target: "backend-test".into(),
            environment: "production".into(),
            public_base_url: Some("https://stloads.test".into()),
            cors_allowed_origins: vec!["https://stloads.test".into()],
            run_migrations: false,
            database_url: None,
            database_schema: None,
            document_storage_backend: "local".into(),
            document_storage_root: "./runtime/test-documents".into(),
            object_storage_bucket: None,
            object_storage_region: "us-south".into(),
            object_storage_endpoint: None,
            object_storage_access_key_id: None,
            object_storage_secret_access_key: None,
            object_storage_session_token: None,
            object_storage_force_path_style: false,
            object_storage_prefix: "tests".into(),
            stripe_webhook_shared_secret: None,
            stripe_webhook_connect_secret: None,
            stripe_secret_key: None,
            stripe_api_base_url: "https://api.stripe.com/v1".into(),
            stripe_connect_refresh_url: None,
            stripe_connect_return_url: None,
            stripe_live_transfers_required: false,
            atmp_outbound_base_url: None,
            atmp_integration_shared_secret: None,
            atmp_integration_require_signature: false,
            atmp_integration_replay_window_seconds: 300,
            atmp_integration_rate_limit_per_minute: 120,
            atmp_outbound_worker_enabled: false,
            atmp_outbound_interval_seconds: 30,
            atmp_outbound_batch_size: 25,
            atmp_outbound_max_attempts: 8,
            tms_shared_secret: None,
            tms_reconciliation_worker_enabled: false,
            tms_reconciliation_interval_seconds: 21_600,
            tms_retry_worker_enabled: false,
            tms_retry_interval_seconds: 300,
            tms_retry_batch_size: 10,
            tms_retry_max_attempts: 5,
            tms_stale_handoff_days: 30,
            mail_mailer: "log".into(),
            mail_host: None,
            mail_port: 587,
            mail_username: None,
            mail_password: None,
            mail_encryption: None,
            mail_from_address: "noreply@stloads.test".into(),
            mail_from_name: "STLoads".into(),
            mail_fail_open: true,
            mail_outbox_enabled: false,
            mail_outbox_worker_enabled: false,
            mail_outbox_batch_size: 25,
            mail_outbox_retry_interval_seconds: 60,
            mail_outbox_max_attempts: 5,
            portal_url: "https://stloads.test".into(),
        };
        let (realtime_tx, _) = broadcast::channel::<RoutedRealtimeEvent>(16);
        AppState {
            document_storage: DocumentStorageService::from_config(&config),
            email: EmailService::from_config_with_pool(&config, None),
            stripe: StripeService::from_config(&config),
            integration_auth: IntegrationAuthState::default(),
            config,
            pool: None,
            realtime_tx,
            security: crate::state::SecurityState::default(),
        }
    }

    #[tokio::test]
    async fn operations_fallback_uses_empty_production_state_without_sample_rows() {
        let state = no_pool_state();

        let screen = super::stloads_operations_screen(&state, Some("published".into())).await;

        assert_eq!(screen.sync_issue_summary.total, 0);
        assert!(screen.recent_sync_issues.is_empty());
        assert!(screen.dead_letter_events.is_empty());
        assert!(screen.handoffs.is_empty());
        assert_eq!(screen.pagination.total, 0);
        assert!(
            screen
                .notes
                .iter()
                .all(|note| !note.to_ascii_lowercase().contains("sample"))
        );
    }

    #[tokio::test]
    async fn reconciliation_fallback_uses_empty_production_state_without_sample_rows() {
        let state = no_pool_state();

        let screen = super::stloads_reconciliation_screen(&state, Some("all".into())).await;

        assert!(screen.logs.is_empty());
        assert!(screen.error_breakdown.is_empty());
        assert_eq!(screen.pagination.total, 0);
        assert!(
            screen
                .callouts
                .iter()
                .all(|note| !note.to_ascii_lowercase().contains("sample"))
        );
    }
}
