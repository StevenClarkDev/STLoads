use futures_util::future::AbortHandle;
use leptos::{prelude::*, tachys::view::any_view::IntoAny, task::spawn_local};
use leptos_router::{components::A, hooks::use_params_map};

use crate::{
    api, device_location, document_upload, realtime,
    session::{self, use_auth},
};
use shared::{
    ExecutionCloseoutApprovalRequest, ExecutionCustomerTrackingLinkRequest,
    ExecutionCustomerTrackingRevokeRequest, ExecutionFinanceExceptionDecisionRequest,
    ExecutionFinanceExceptionRequest, ExecutionLegActionRequest, ExecutionLegScreen,
    ExecutionRoutePlanRequest, ExecutionTelematicsConnectionRequest,
    ExecutionTrackingConsentRequest, RealtimeEventKind, RealtimeTopic,
};

use super::{
    execution_helpers::*,
    execution_workflow_helpers::*,
    shared::{FIELD_LABEL_STYLE, MUTED_TEXT_STYLE, PANEL_SCROLL_STYLE},
};
#[component]
pub fn ExecutionLegPage() -> impl IntoView {
    let auth = use_auth();
    let auth_for_admin_handoffs = auth;
    let auth_for_desk_handoffs = auth;
    let auth_for_payment_handoffs = auth;
    let params = use_params_map();
    let leg_id = Memo::new(move |_| {
        params.with(|map| {
            map.get("leg_id")
                .and_then(|value| value.parse::<u64>().ok())
        })
    });

    let screen = RwSignal::new(None::<ExecutionLegScreen>);
    let is_loading = RwSignal::new(false);
    let error_message = RwSignal::new(None::<String>);
    let action_message = RwSignal::new(None::<String>);
    let pending_action_key = RwSignal::new(None::<String>);
    let is_sending_location = RwSignal::new(false);
    let action_note = RwSignal::new(String::new());
    let refresh_nonce = RwSignal::new(0_u64);
    let ws_handle = RwSignal::new(None::<AbortHandle>);
    let ws_connected = RwSignal::new(false);
    let upload_document_name = RwSignal::new(String::new());
    let upload_document_type = RwSignal::new("delivery_pod".to_string());
    let is_uploading_document = RwSignal::new(false);
    let is_recording_consent = RwSignal::new(false);
    let is_saving_workflow = RwSignal::new(false);
    let closeout_note = RwSignal::new(String::new());
    let finance_exception_description = RwSignal::new(String::new());
    let finance_exception_amount = RwSignal::new(String::new());
    let finance_exception_decision_type = RwSignal::new("accessorial".to_string());
    let finance_exception_decision_status = RwSignal::new("approved".to_string());
    let finance_exception_resolution_note = RwSignal::new(String::new());
    let customer_tracking_expires_hours = RwSignal::new("168".to_string());
    let customer_tracking_revoke_reason = RwSignal::new(String::new());
    let telematics_provider = RwSignal::new("manual_mobile".to_string());
    let route_distance = RwSignal::new(String::new());
    let route_duration = RwSignal::new(String::new());
    let live_tracking_enabled = RwSignal::new(false);
    let is_toggling_live_tracking = RwSignal::new(false);
    let live_tracking_watcher_id = RwSignal::new(None::<i32>);
    let can_open_admin_handoffs = Signal::derive(move || {
        session::has_permission(&auth_for_admin_handoffs, "access_admin_portal")
            || session::has_permission(&auth_for_admin_handoffs, "manage_loads")
    });
    let can_open_desk_handoffs = Signal::derive(move || {
        session::has_permission(&auth_for_desk_handoffs, "access_admin_portal")
            || session::has_permission(&auth_for_desk_handoffs, "manage_dispatch_desk")
    });
    let can_open_payment_handoffs = Signal::derive(move || {
        session::has_permission(&auth_for_payment_handoffs, "access_admin_portal")
            || session::has_permission(&auth_for_payment_handoffs, "manage_payments")
    });

    on_cleanup(move || {
        if let Some(watcher_id) = live_tracking_watcher_id.get_untracked() {
            let _ = device_location::stop_live_tracking(watcher_id);
        }
    });

    Effect::new(move |_| {
        let ready = auth.session_ready.get();
        let current_session = auth.session.get();
        let leg_id = leg_id.get();
        let _refresh = refresh_nonce.get();

        if !ready {
            return;
        }

        let Some(leg_id) = leg_id else {
            screen.set(None);
            is_loading.set(false);
            error_message.set(Some(
                "The requested Rust execution URL is missing a valid leg id.".into(),
            ));
            return;
        };

        if !current_session.authenticated {
            screen.set(None);
            is_loading.set(false);
            error_message.set(Some(
                "Sign in before opening the Rust execution workspace.".into(),
            ));
            return;
        }

        is_loading.set(true);
        let auth = auth;

        spawn_local(async move {
            match api::fetch_execution_leg_screen(leg_id).await {
                Ok(next_screen) => {
                    error_message.set(None);
                    screen.set(Some(next_screen));
                    if let Ok(summary) =
                        document_upload::replay_offline_execution_submissions(leg_id).await
                        && (!summary.contains("\"replayed\":0")
                            || !summary.contains("\"pendingCount\":0"))
                    {
                        action_message.set(Some(format!(
                            "Offline execution replay checked: {}",
                            summary
                        )));
                    }
                }
                Err(error) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(
                            &auth,
                            "Your Rust session expired; sign in again.",
                        );
                    }
                    error_message.set(Some(error));
                }
            }

            is_loading.set(false);
        });
    });

    Effect::new(move |_| {
        let current_session = auth.session.get();
        if !auth.session_ready.get() || !current_session.authenticated {
            if let Some(existing_handle) = ws_handle.get_untracked() {
                existing_handle.abort();
                ws_handle.set(None);
            }
            ws_connected.set(false);
            return;
        }

        let current_leg_id = leg_id.get();
        let current_user_id = current_session.user.as_ref().map(|user| user.id);
        let auth = auth;

        if let Some(existing_handle) = ws_handle.get_untracked() {
            existing_handle.abort();
        }

        let handle = realtime::connect_realtime_listener(
            None,
            vec![RealtimeTopic::ExecutionTracking],
            move |event| match event.kind {
                RealtimeEventKind::LegExecutionUpdated | RealtimeEventKind::LegLocationUpdated => {
                    if event.leg_id == current_leg_id {
                        refresh_nonce.update(|value| *value += 1);
                        action_message.set(Some(format!("Realtime update: {}", event.summary)));
                    }
                }
                RealtimeEventKind::SessionInvalidated => {
                    if event.actor_user_id == current_user_id {
                        if let Some(existing_handle) = ws_handle.get_untracked() {
                            existing_handle.abort();
                            ws_handle.set(None);
                        }
                        session::invalidate_session(
                            &auth,
                            "The current Rust session was invalidated; sign in again.",
                        );
                        ws_connected.set(false);
                    }
                }
                _ => {}
            },
        );

        ws_connected.set(handle.is_some());
        ws_handle.set(handle);
    });

    let run_action = move |action_key: String| {
        let Some(leg_id) = leg_id.get() else {
            action_message.set(Some(
                "Missing leg id for the requested execution action.".into(),
            ));
            return;
        };

        pending_action_key.set(Some(action_key.clone()));
        action_message.set(None);
        let auth = auth;

        spawn_local(async move {
            match api::run_execution_leg_action(
                leg_id,
                &ExecutionLegActionRequest {
                    action_key: action_key.clone(),
                    note: {
                        let value = action_note.get();
                        (!value.trim().is_empty()).then_some(value)
                    },
                },
            )
            .await
            {
                Ok(response) => {
                    action_message.set(Some(response.message));
                    if response.success {
                        action_note.set(String::new());
                        refresh_nonce.update(|value| *value += 1);
                    }
                }
                Err(error) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(
                            &auth,
                            "Your Rust session expired; sign in again.",
                        );
                        action_message.set(Some(error));
                    } else {
                        let payload = format!(
                            "{{\"action_key\":{:?},\"note\":{:?}}}",
                            action_key.clone(),
                            action_note.get()
                        );
                        match document_upload::queue_offline_execution_submission(
                            leg_id,
                            "driver_action",
                            &payload,
                        ) {
                            Ok(summary) => action_message.set(Some(format!(
                                "Network failed, so the driver action was queued offline: {}",
                                summary
                            ))),
                            Err(queue_error) => action_message.set(Some(format!(
                                "{}; offline queue failed: {}",
                                error, queue_error
                            ))),
                        }
                    }
                }
            }

            pending_action_key.set(None);
        });
    };

    let send_current_location = move |_| {
        let Some(leg_id) = leg_id.get() else {
            action_message.set(Some(
                "Missing leg id for the requested location ping.".into(),
            ));
            return;
        };

        is_sending_location.set(true);
        action_message.set(None);
        let auth = auth;

        spawn_local(async move {
            let captured_position = device_location::current_position().await;
            let result = async {
                let (lat, lng) = captured_position.clone()?;
                api::send_execution_location_ping(
                    leg_id,
                    &shared::ExecutionLocationPingRequest {
                        lat,
                        lng,
                        recorded_at: None,
                    },
                )
                .await
            }
            .await;

            match result {
                Ok(response) => {
                    action_message.set(Some(response.message));
                    if response.success {
                        refresh_nonce.update(|value| *value += 1);
                    }
                }
                Err(error) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(
                            &auth,
                            "Your Rust session expired; sign in again.",
                        );
                        action_message.set(Some(error));
                    } else if let Ok((lat, lng)) = captured_position {
                        let payload = format!("{{\"lat\":{},\"lng\":{}}}", lat, lng);
                        match document_upload::queue_offline_execution_submission(
                            leg_id, "gps_ping", &payload,
                        ) {
                            Ok(summary) => action_message.set(Some(format!(
                                "Network failed, so the GPS ping was queued offline: {}",
                                summary
                            ))),
                            Err(queue_error) => action_message.set(Some(format!(
                                "{}; offline queue failed: {}",
                                error, queue_error
                            ))),
                        }
                    } else {
                        action_message.set(Some(error));
                    }
                }
            }

            is_sending_location.set(false);
        });
    };

    let upload_execution_document = move || {
        let Some(current_screen) = screen.get() else {
            action_message.set(Some(
                "Execution data is not ready yet, so document upload cannot start.".into(),
            ));
            return;
        };

        if !current_screen.can_upload_documents {
            action_message.set(Some(
                "The current Rust session cannot upload execution documents for this leg.".into(),
            ));
            return;
        }

        let document_type_value = upload_document_type.get();
        if document_type_value.trim().is_empty() {
            action_message.set(Some(
                "Choose an execution document type before uploading a file.".into(),
            ));
            return;
        }

        let document_name_value = {
            let value = upload_document_name.get();
            if value.trim().is_empty() {
                document_type_value.replace('_', " ")
            } else {
                value
            }
        };

        let input_id = document_upload::execution_upload_input_id(current_screen.leg_id);
        is_uploading_document.set(true);
        action_message.set(None);
        let auth = auth;

        spawn_local(async move {
            match document_upload::upload_execution_document(
                current_screen.leg_id,
                &document_name_value,
                &document_type_value,
                &input_id,
            )
            .await
            {
                Ok(response) => {
                    action_message.set(Some(response.message));
                    if response.success {
                        upload_document_name.set(String::new());
                        upload_document_type.set("delivery_pod".to_string());
                        refresh_nonce.update(|value| *value += 1);
                    }
                }
                Err(error) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(
                            &auth,
                            "Your Rust session expired; sign in again.",
                        );
                        action_message.set(Some(error));
                    } else {
                        match document_upload::queue_offline_execution_document_upload(
                            current_screen.leg_id,
                            &document_name_value,
                            &document_type_value,
                            &input_id,
                        )
                        .await
                        {
                            Ok(summary) => action_message.set(Some(format!(
                                "Upload failed, so the document was queued offline: {}",
                                summary
                            ))),
                            Err(queue_error) => action_message.set(Some(format!(
                                "{}; offline document queue failed: {}",
                                error, queue_error
                            ))),
                        }
                    }
                }
            }

            is_uploading_document.set(false);
        });
    };

    let record_tracking_consent = move |_| {
        let Some(current_screen) = screen.get() else {
            action_message.set(Some(
                "Execution data is not ready yet, so tracking consent cannot be recorded.".into(),
            ));
            return;
        };

        is_recording_consent.set(true);
        action_message.set(None);
        let auth = auth;

        spawn_local(async move {
            match api::capture_execution_tracking_consent(
                current_screen.leg_id,
                &ExecutionTrackingConsentRequest {
                    consent_text: current_screen.tracking_consent_text.clone(),
                },
            )
            .await
            {
                Ok(response) => {
                    action_message.set(Some(response.message));
                    if response.success {
                        refresh_nonce.update(|value| *value += 1);
                    }
                }
                Err(error) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(
                            &auth,
                            "Your Rust session expired; sign in again.",
                        );
                    }
                    action_message.set(Some(error));
                }
            }

            is_recording_consent.set(false);
        });
    };

    let queue_offline_note = move |_| {
        let Some(current_screen) = screen.get() else {
            action_message.set(Some(
                "Execution data is not ready for offline queueing.".into(),
            ));
            return;
        };

        let payload = format!(
            "{{\"note\":{:?},\"status\":\"{}\"}}",
            action_note.get(),
            current_screen.status_label
        );
        match document_upload::queue_offline_execution_submission(
            current_screen.leg_id,
            "driver_note",
            &payload,
        ) {
            Ok(summary) => action_message.set(Some(format!(
                "Offline note queued locally for later reconciliation: {}",
                summary
            ))),
            Err(error) => action_message.set(Some(error)),
        }
    };

    let save_closeout_approval = move |_| {
        let Some(current_screen) = screen.get() else {
            action_message.set(Some(
                "Execution data is not ready for closeout review.".into(),
            ));
            return;
        };

        is_saving_workflow.set(true);
        action_message.set(None);
        let auth = auth;

        spawn_local(async move {
            match api::review_execution_closeout(
                current_screen.leg_id,
                &ExecutionCloseoutApprovalRequest {
                    pod_review_status: "approved".into(),
                    export_path: Some(format!(
                        "/execution/legs/{}/closeout-package",
                        current_screen.leg_id
                    )),
                    note: {
                        let value = closeout_note.get();
                        (!value.trim().is_empty()).then_some(value)
                    },
                },
            )
            .await
            {
                Ok(response) => {
                    action_message.set(Some(response.message));
                    if response.success {
                        closeout_note.set(String::new());
                        refresh_nonce.update(|value| *value += 1);
                    }
                }
                Err(error) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(
                            &auth,
                            "Your Rust session expired; sign in again.",
                        );
                    }
                    action_message.set(Some(error));
                }
            }
            is_saving_workflow.set(false);
        });
    };

    let save_finance_exception = move |_| {
        let Some(current_screen) = screen.get() else {
            action_message.set(Some(
                "Execution data is not ready for finance exception intake.".into(),
            ));
            return;
        };

        let description = finance_exception_description.get();
        let amount_cents = finance_exception_amount
            .get()
            .trim()
            .parse::<f64>()
            .ok()
            .map(|value| (value * 100.0).round() as i64);
        is_saving_workflow.set(true);
        action_message.set(None);
        let auth = auth;

        spawn_local(async move {
            match api::create_execution_finance_exception(
                current_screen.leg_id,
                &ExecutionFinanceExceptionRequest {
                    exception_type: "accessorial".into(),
                    status: "pending".into(),
                    amount_cents,
                    visibility: "internal".into(),
                    description,
                    evidence_document_id: None,
                },
            )
            .await
            {
                Ok(response) => {
                    action_message.set(Some(response.message));
                    if response.success {
                        finance_exception_description.set(String::new());
                        finance_exception_amount.set(String::new());
                        refresh_nonce.update(|value| *value += 1);
                    }
                }
                Err(error) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(
                            &auth,
                            "Your Rust session expired; sign in again.",
                        );
                    }
                    action_message.set(Some(error));
                }
            }
            is_saving_workflow.set(false);
        });
    };

    let create_customer_tracking_link = move |_| {
        let Some(current_screen) = screen.get() else {
            action_message.set(Some(
                "Execution data is not ready for customer tracking setup.".into(),
            ));
            return;
        };

        let expires_in_hours = customer_tracking_expires_hours
            .get()
            .trim()
            .parse::<i64>()
            .ok();
        is_saving_workflow.set(true);
        action_message.set(None);
        let auth = auth;

        spawn_local(async move {
            match api::create_customer_tracking_link(
                current_screen.leg_id,
                &ExecutionCustomerTrackingLinkRequest {
                    visibility_scope: "status_eta_latest_location".into(),
                    expires_in_hours,
                    rotate_existing: true,
                },
            )
            .await
            {
                Ok(response) => {
                    action_message.set(Some(response.message));
                    if response.success {
                        refresh_nonce.update(|value| *value += 1);
                    }
                }
                Err(error) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(
                            &auth,
                            "Your Rust session expired; sign in again.",
                        );
                    }
                    action_message.set(Some(error));
                }
            }
            is_saving_workflow.set(false);
        });
    };

    let decide_finance_exception = move |_| {
        let Some(current_screen) = screen.get() else {
            action_message.set(Some(
                "Execution data is not ready for finance exception decisions.".into(),
            ));
            return;
        };

        is_saving_workflow.set(true);
        action_message.set(None);
        let exception_type = finance_exception_decision_type.get();
        let status = finance_exception_decision_status.get();
        let note = finance_exception_resolution_note.get();
        let auth = auth;

        spawn_local(async move {
            match api::decide_execution_finance_exception(
                current_screen.leg_id,
                &ExecutionFinanceExceptionDecisionRequest {
                    exception_type,
                    status,
                    resolution_note: (!note.trim().is_empty()).then_some(note),
                },
            )
            .await
            {
                Ok(response) => {
                    action_message.set(Some(response.message));
                    if response.success {
                        finance_exception_resolution_note.set(String::new());
                        refresh_nonce.update(|value| *value += 1);
                    }
                }
                Err(error) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(
                            &auth,
                            "Your Rust session expired; sign in again.",
                        );
                    }
                    action_message.set(Some(error));
                }
            }
            is_saving_workflow.set(false);
        });
    };

    let revoke_customer_tracking_link = move |_| {
        let Some(current_screen) = screen.get() else {
            action_message.set(Some(
                "Execution data is not ready for customer tracking revocation.".into(),
            ));
            return;
        };

        is_saving_workflow.set(true);
        action_message.set(None);
        let reason = customer_tracking_revoke_reason.get();
        let auth = auth;

        spawn_local(async move {
            match api::revoke_customer_tracking_links(
                current_screen.leg_id,
                &ExecutionCustomerTrackingRevokeRequest {
                    reason: (!reason.trim().is_empty()).then_some(reason),
                },
            )
            .await
            {
                Ok(response) => {
                    action_message.set(Some(response.message));
                    if response.success {
                        customer_tracking_revoke_reason.set(String::new());
                        refresh_nonce.update(|value| *value += 1);
                    }
                }
                Err(error) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(
                            &auth,
                            "Your Rust session expired; sign in again.",
                        );
                    }
                    action_message.set(Some(error));
                }
            }
            is_saving_workflow.set(false);
        });
    };

    let save_telematics = move |_| {
        let Some(current_screen) = screen.get() else {
            action_message.set(Some(
                "Execution data is not ready for telematics setup.".into(),
            ));
            return;
        };

        is_saving_workflow.set(true);
        action_message.set(None);
        let provider = telematics_provider.get();
        let auth = auth;

        spawn_local(async move {
            match api::upsert_execution_telematics(
                current_screen.leg_id,
                &ExecutionTelematicsConnectionRequest {
                    provider_key: provider,
                    status: "planned".into(),
                    fallback_behavior:
                        "Use mobile driver tracking when provider data is stale or unavailable."
                            .into(),
                },
            )
            .await
            {
                Ok(response) => {
                    action_message.set(Some(response.message));
                    if response.success {
                        refresh_nonce.update(|value| *value += 1);
                    }
                }
                Err(error) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(
                            &auth,
                            "Your Rust session expired; sign in again.",
                        );
                    }
                    action_message.set(Some(error));
                }
            }
            is_saving_workflow.set(false);
        });
    };

    let save_route_plan = move |_| {
        let Some(current_screen) = screen.get() else {
            action_message.set(Some(
                "Execution data is not ready for route planning.".into(),
            ));
            return;
        };

        is_saving_workflow.set(true);
        action_message.set(None);
        let distance = route_distance.get().trim().parse::<f64>().ok();
        let duration = route_duration.get().trim().parse::<i32>().ok();
        let auth = auth;

        spawn_local(async move {
            match api::upsert_execution_route_plan(
                current_screen.leg_id,
                &ExecutionRoutePlanRequest {
                    provider_key: "manual".into(),
                    distance_miles: distance,
                    duration_minutes: duration,
                    truck_safe: true,
                    status: "approved_manual".into(),
                    constraints: serde_json::json!({
                        "source": "operator_entered",
                        "truck_safe_review": true
                    }),
                },
            )
            .await
            {
                Ok(response) => {
                    action_message.set(Some(response.message));
                    if response.success {
                        refresh_nonce.update(|value| *value += 1);
                    }
                }
                Err(error) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(
                            &auth,
                            "Your Rust session expired; sign in again.",
                        );
                    }
                    action_message.set(Some(error));
                }
            }
            is_saving_workflow.set(false);
        });
    };

    let open_document = move |download_path: String| {
        action_message.set(None);
        spawn_local(async move {
            match document_upload::open_protected_document(&download_path).await {
                Ok(()) => {
                    action_message.set(Some(
                        "Execution document opened in a new browser tab.".into(),
                    ));
                }
                Err(error) => {
                    action_message.set(Some(error));
                }
            }
        });
    };

    let start_live_tracking = move |_| {
        let Some(leg_id) = leg_id.get() else {
            action_message.set(Some("Missing leg id for Rust live tracking.".into()));
            return;
        };

        is_toggling_live_tracking.set(true);
        action_message.set(None);
        let auth = auth;

        spawn_local(async move {
            let url = api::api_href(&format!("/execution/legs/{}/location", leg_id));
            let token = api::auth_token().unwrap_or_default();

            match device_location::start_live_tracking(&url, &token).await {
                Ok(watcher_id) => {
                    live_tracking_watcher_id.set(Some(watcher_id));
                    live_tracking_enabled.set(true);
                    action_message.set(Some(
                        "Live tracking is on. This Rust execution page will keep sending GPS updates while it stays open."
                            .into(),
                    ));
                }
                Err(error) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(
                            &auth,
                            "Your Rust session expired; sign in again.",
                        );
                    }
                    action_message.set(Some(error));
                }
            }

            is_toggling_live_tracking.set(false);
        });
    };

    let stop_live_tracking = move |_| {
        if let Some(watcher_id) = live_tracking_watcher_id.get() {
            let _ = device_location::stop_live_tracking(watcher_id);
        }
        live_tracking_watcher_id.set(None);
        live_tracking_enabled.set(false);
        action_message.set(Some(
            "Live tracking is off. You can still send one-off GPS updates manually from this Rust execution workspace."
                .into(),
        ));
    };

    view! {
        <article style="display:grid;gap:1.25rem;">
            <section style="display:flex;justify-content:space-between;gap:1rem;align-items:flex-start;flex-wrap:wrap;">
                <div style=FIELD_LABEL_STYLE>
                    <h2>{move || screen.get().map(|value| value.title).unwrap_or_else(|| "Execution Workspace".into())}</h2>
                    <p>{move || screen.get().map(|value| value.subtitle).unwrap_or_else(|| "Rust tracking and execution view".into())}</p>
                </div>
                <div style="display:flex;gap:0.75rem;flex-wrap:wrap;align-items:center;">
                    <A href="/loads" attr:style="padding:0.7rem 1rem;border-radius:0.9rem;background:#f4f4f5;color:#111827;text-decoration:none;">"Back to loads"</A>
                    <A href=move || screen.get().map(|value| format!("/loads/{}", value.load_id)).unwrap_or_else(|| "/loads".into()) attr:style="padding:0.7rem 1rem;border-radius:0.9rem;background:#111827;color:white;text-decoration:none;">"Open load profile"</A>
                </div>
            </section>

            {move || error_message.get().map(|message| view! {
                <section style="padding:0.85rem 1rem;border:1px solid #fecaca;border-radius:0.9rem;background:#fff1f2;color:#be123c;">{message}</section>
            })}

            {move || {
                if is_loading.get() && screen.get().is_none() {
                    view! {
                        <section style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#fafaf9;">
                            "Loading Rust execution data..."
                        </section>
                    }.into_any()
                } else if let Some(screen_value) = screen.get() {
                    let action_items = screen_value.action_items.clone();
                    let timeline = screen_value.timeline.clone();
                    let timeline_count = timeline.len();
                    let notes_history = screen_value.notes_history.clone();
                    let note_count = notes_history.len();
                    let tracking_points = screen_value.tracking_points.clone();
                    let tracking_point_count = tracking_points.len();
                    let tracking_embed = tracking_embed_url(&tracking_points);
                    let tracking_distance_label = tracking_distance_km(&tracking_points)
                        .map(|value| format!("{:.1} km", value))
                        .unwrap_or_else(|| "Need more GPS points".into());
                    let tracking_window_label = tracking_window_summary(&tracking_points)
                        .unwrap_or_else(|| "Waiting for the first GPS ping".into());
                    let tracking_health_label = screen_value
                        .tracking_health_label
                        .clone()
                        .unwrap_or_else(|| "Tracking health will appear here once execution data settles.".into());
                    let tracking_health_tone = screen_value.tracking_health_tone.clone();
                    let latest_tracking_point = tracking_points
                        .iter()
                        .find(|point| point.is_latest)
                        .cloned()
                        .or_else(|| tracking_points.first().cloned());
                    let earliest_tracking_point = tracking_points.last().cloned();
                    let documents = screen_value.documents.clone();
                    let required_documents = screen_value.required_documents.clone();
                    let document_count = documents.len();
                    let route_stage_key = screen_value.status_label.to_ascii_lowercase();
                    let desk_handoff = if route_stage_key.contains("delivery")
                        || route_stage_key.contains("completed")
                        || screen_value.delivery_completion_ready
                    {
                        ("/desk/closeout", "Open closeout desk")
                    } else {
                        ("/desk/facility", "Open facility desk")
                    };
                    view! {
                        <>
                            <section style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:1rem;">
                                <InfoCard label="Leg" value=screen_value.leg_code.clone() />
                                <InfoCard label="Route" value=screen_value.route_label.clone() />
                                <InfoCard label="Carrier" value=screen_value.carrier_label.clone().unwrap_or_else(|| "No carrier booked yet".into()) />
                                <div style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#ffffff;display:grid;gap:0.4rem;">
                                    <small style=MUTED_TEXT_STYLE>"Execution status"</small>
                                    <span style=tone_style(&screen_value.status_tone)>{screen_value.status_label.clone()}</span>
                                    <small>{move || if ws_connected.get() { "Realtime execution refresh connected" } else { "Realtime execution refresh idle" }}</small>
                                </div>
                            </section>

                            <section style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:1rem;">
                                <InfoCard label="Tracking Points" value=tracking_point_count.to_string() />
                                <InfoCard label="Approx. Distance" value=tracking_distance_label.clone() />
                                <InfoCard label="Timeline Events" value=timeline_count.to_string() />
                                <InfoCard label="Execution Notes" value=note_count.to_string() />
                                <InfoCard label="Operator Mode" value=screen_value.operator_mode_label.clone() />
                                <InfoCard
                                    label="Privacy"
                                    value=if screen_value.tracking_consent_granted {
                                        "Tracking consent recorded".into()
                                    } else {
                                        "Consent required".into()
                                    }
                                />
                                <InfoCard
                                    label="Closeout"
                                    value=screen_value.closeout_package_label.clone()
                                />
                                <InfoCard
                                    label="Route Plan"
                                    value=screen_value.route_plan_label.clone()
                                />
                                <InfoCard
                                    label="Delivery Readiness"
                                    value=if screen_value.delivery_completion_ready {
                                        "Ready for completion".into()
                                    } else {
                                        "Waiting on POD and note".into()
                                    }
                                />
                            </section>

                            <section style="display:flex;gap:0.5rem;flex-wrap:wrap;">
                                {screen_value.latest_map_url.clone().map(|map_url| view! {
                                    <a href=map_url target="_blank" rel="noopener noreferrer" style="padding:0.45rem 0.75rem;border-radius:0.75rem;background:#e0f2fe;color:#075985;text-decoration:none;">
                                        "Open latest point"
                                    </a>
                                })}
                                {earliest_tracking_point.clone().zip(latest_tracking_point.clone()).map(|(start, end)| view! {
                                    <a href=tracking_route_maps_url(&start, &end) target="_blank" rel="noopener noreferrer" style="padding:0.45rem 0.75rem;border-radius:0.75rem;background:#ede9fe;color:#5b21b6;text-decoration:none;">
                                        "Open route span"
                                    </a>
                                })}
                                <A href=format!("/loads/{}", screen_value.load_id) attr:style="padding:0.45rem 0.75rem;border-radius:0.75rem;background:#f1f5f9;color:#0f172a;text-decoration:none;">
                                    "Open load profile"
                                </A>
                                {can_open_admin_handoffs.get().then(|| view! {
                                    <A href=format!("/admin/loads/{}", screen_value.load_id) attr:style="padding:0.45rem 0.75rem;border-radius:0.75rem;background:#eef2ff;color:#312e81;text-decoration:none;">
                                        "Admin load profile"
                                    </A>
                                })}
                                {can_open_desk_handoffs.get().then(|| view! {
                                    <A href=desk_handoff.0 attr:style="padding:0.45rem 0.75rem;border-radius:0.75rem;background:#fff7dd;color:#92400e;text-decoration:none;">
                                        {desk_handoff.1}
                                    </A>
                                })}
                                {can_open_payment_handoffs.get().then(|| view! {
                                    <A href=format!("/admin/payments?leg_id={}&source=execution", screen_value.leg_id) attr:style="padding:0.45rem 0.75rem;border-radius:0.75rem;background:#e8fff3;color:#166534;text-decoration:none;">
                                        "Payments"
                                    </A>
                                })}
                            </section>

                            {render_execution_tracking_workspace(                                 screen_value.clone(),                                 action_items,                                 tracking_points,                                 tracking_embed,                                 latest_tracking_point,                                 earliest_tracking_point,                                 tracking_distance_label,                                 tracking_window_label,                                 tracking_health_label,                                 tracking_health_tone,                                 is_recording_consent,                                 live_tracking_enabled,                                 is_toggling_live_tracking,                                 is_sending_location,                                 pending_action_key,                                 action_note,                                 record_tracking_consent,                                 start_live_tracking,                                 stop_live_tracking,                                 send_current_location,                                 queue_offline_note,                                 run_action,                             )}

                            <section style="display:grid;grid-template-columns:repeat(auto-fit,minmax(320px,1fr));gap:1rem;align-items:start;">
                                <section style=PANEL_SCROLL_STYLE>
                                    <strong>"Timeline"</strong>
                                    {render_timeline(timeline)}
                                </section>
                                <section style=PANEL_SCROLL_STYLE>
                                    <strong>"Execution note history"</strong>
                                    {render_execution_notes(notes_history)}
                                </section>
                            </section>

                            {render_closeout_documents_customer_and_finance(                                 screen_value.clone(),                                 required_documents,                                 documents,                                 document_count,                                 upload_document_name,                                 upload_document_type,                                 is_uploading_document,                                 closeout_note,                                 is_saving_workflow,                                 customer_tracking_expires_hours,                                 customer_tracking_revoke_reason,                                 telematics_provider,                                 route_distance,                                 route_duration,                                 finance_exception_description,                                 finance_exception_amount,                                 finance_exception_decision_type,                                 finance_exception_decision_status,                                 finance_exception_resolution_note,                                 upload_execution_document,                                 open_document,                                 save_closeout_approval,                                 create_customer_tracking_link,                                 revoke_customer_tracking_link,                                 save_telematics,                                 save_route_plan,                                 save_finance_exception,                                 decide_finance_exception,                             )}

                        </>
                    }.into_any()
                } else {
                    view! {
                        <section style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#fafaf9;">
                            "No Rust execution data is available yet for this route."
                        </section>
                    }.into_any()
                }
            }}
        </article>
    }
}
