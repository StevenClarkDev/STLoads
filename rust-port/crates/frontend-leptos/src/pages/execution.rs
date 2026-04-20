use futures_util::future::AbortHandle;
use leptos::{prelude::*, tachys::view::any_view::IntoAny, task::spawn_local};
use leptos_router::{components::A, hooks::use_params_map};

use crate::{
    api, device_location, document_upload, realtime,
    session::{self, use_auth},
};
use shared::{
    ExecutionActionItem, ExecutionDocumentItem, ExecutionDocumentTypeOption,
    ExecutionLegActionRequest, ExecutionLegScreen, ExecutionNoteItem, ExecutionTimelineItem,
    ExecutionTrackingPointItem, RealtimeEventKind, RealtimeTopic,
};

fn tone_style(tone: &str) -> &'static str {
    match tone {
        "success" => {
            "background:#e8fff3;padding:0.25rem 0.55rem;border-radius:999px;color:#0f766e;"
        }
        "warning" => {
            "background:#fff7dd;padding:0.25rem 0.55rem;border-radius:999px;color:#b45309;"
        }
        "danger" => "background:#ffe4e6;padding:0.25rem 0.55rem;border-radius:999px;color:#be123c;",
        "info" => "background:#e0f2fe;padding:0.25rem 0.55rem;border-radius:999px;color:#0369a1;",
        "primary" => {
            "background:#ede9fe;padding:0.25rem 0.55rem;border-radius:999px;color:#6d28d9;"
        }
        _ => "background:#f1f5f9;padding:0.25rem 0.55rem;border-radius:999px;color:#475569;",
    }
}

fn tracking_embed_url(points: &[ExecutionTrackingPointItem]) -> Option<String> {
    let latest = points
        .iter()
        .find(|point| point.is_latest)
        .or_else(|| points.first())?;

    let (min_lat, max_lat) = points
        .iter()
        .fold((latest.lat, latest.lat), |(min, max), point| {
            (min.min(point.lat), max.max(point.lat))
        });
    let (min_lng, max_lng) = points
        .iter()
        .fold((latest.lng, latest.lng), |(min, max), point| {
            (min.min(point.lng), max.max(point.lng))
        });

    let lat_padding = ((max_lat - min_lat).abs() * 0.25).max(0.02);
    let lng_padding = ((max_lng - min_lng).abs() * 0.25).max(0.02);

    Some(format!(
        "https://www.openstreetmap.org/export/embed.html?bbox={:.6},{:.6},{:.6},{:.6}&layer=mapnik&marker={:.6},{:.6}",
        min_lng - lng_padding,
        min_lat - lat_padding,
        max_lng + lng_padding,
        max_lat + lat_padding,
        latest.lat,
        latest.lng,
    ))
}

fn tracking_point_map_url(point: &ExecutionTrackingPointItem) -> String {
    format!(
        "https://www.google.com/maps?q={:.6},{:.6}",
        point.lat, point.lng
    )
}

fn tracking_route_maps_url(
    start: &ExecutionTrackingPointItem,
    end: &ExecutionTrackingPointItem,
) -> String {
    format!(
        "https://www.google.com/maps/dir/{:.6},{:.6}/{:.6},{:.6}",
        start.lat, start.lng, end.lat, end.lng
    )
}

fn haversine_km(first_lat: f64, first_lng: f64, second_lat: f64, second_lng: f64) -> f64 {
    let earth_radius_km = 6371.0_f64;
    let d_lat = (second_lat - first_lat).to_radians();
    let d_lng = (second_lng - first_lng).to_radians();
    let first_lat = first_lat.to_radians();
    let second_lat = second_lat.to_radians();

    let a = (d_lat / 2.0).sin().powi(2)
        + first_lat.cos() * second_lat.cos() * (d_lng / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    earth_radius_km * c
}

fn tracking_distance_km(points: &[ExecutionTrackingPointItem]) -> Option<f64> {
    if points.len() < 2 {
        return None;
    }

    let total = points
        .windows(2)
        .map(|window| haversine_km(window[0].lat, window[0].lng, window[1].lat, window[1].lng))
        .sum::<f64>();

    Some(total)
}

fn tracking_window_summary(points: &[ExecutionTrackingPointItem]) -> Option<String> {
    if points.is_empty() {
        return None;
    }

    let latest = points
        .iter()
        .find(|point| point.is_latest)
        .or_else(|| points.first())?;
    let earliest = points.last().unwrap_or(latest);

    Some(format!(
        "Window: {} -> {}",
        earliest.recorded_at_label, latest.recorded_at_label
    ))
}

fn tracking_guidance_items(
    point_count: usize,
    tracking_health_tone: &str,
    can_send_location_ping: bool,
    live_tracking_available: bool,
    delivery_completion_ready: bool,
) -> Vec<String> {
    let mut items = Vec::new();

    if point_count == 0 {
        items.push(
            "No GPS points are recorded yet. Send the first ping now so operations can frame the leg on the map."
                .to_string(),
        );
    } else if point_count == 1 {
        items.push(
            "Only one GPS point is recorded. Send one more update so the route span becomes easier to validate."
                .to_string(),
        );
    }

    if matches!(tracking_health_tone, "warning" | "danger") {
        items.push(
            "Tracking health is degraded. Send a fresh GPS ping or restart live tracking before moving the leg forward."
                .to_string(),
        );
    }

    if live_tracking_available
        && point_count > 0
        && !matches!(tracking_health_tone, "warning" | "danger")
    {
        items.push(
            "Live tracking is available here. Keep it running while the truck is moving so dispatch does not lose route continuity."
                .to_string(),
        );
    }

    if !delivery_completion_ready {
        items.push(
            "Delivery completion still needs both a POD upload and an execution note.".to_string(),
        );
    }

    if !can_send_location_ping {
        items.push(
            "This leg is currently view-only for GPS updates, so coordinate with the booked carrier or operations team for the next location refresh."
                .to_string(),
        );
    }

    items
}

fn gps_coverage_label(point_count: usize, tracking_health_tone: &str) -> (&'static str, String) {
    if point_count == 0 {
        ("danger", "No route trace yet".into())
    } else if point_count == 1 {
        ("warning", "Single-point route".into())
    } else if matches!(tracking_health_tone, "warning" | "danger") {
        ("warning", "Route trace is stale".into())
    } else {
        ("success", format!("{} plotted point(s)", point_count))
    }
}

fn document_readiness_label(
    document_count: usize,
    delivery_completion_ready: bool,
) -> (&'static str, String) {
    if delivery_completion_ready {
        ("success", "POD and note ready".into())
    } else if document_count == 0 {
        ("danger", "No execution docs yet".into())
    } else {
        ("warning", format!("{} doc(s) attached", document_count))
    }
}

fn count_execution_documents_by_type(
    documents: &[ExecutionDocumentItem],
    document_type_key: &str,
) -> usize {
    documents
        .iter()
        .filter(|document| document.document_type_key == document_type_key)
        .count()
}

fn execution_blocker_items(
    point_count: usize,
    document_count: usize,
    note_count: usize,
    tracking_health_tone: &str,
    can_send_location_ping: bool,
    live_tracking_available: bool,
    delivery_completion_ready: bool,
) -> Vec<String> {
    let mut items = Vec::new();

    if point_count == 0 {
        items.push(
            "GPS coverage has not started yet, so dispatch still cannot see a live route."
                .to_string(),
        );
    } else if point_count == 1 {
        items.push(
            "Only one GPS point is on record, so the route span is still thin for operator review."
                .to_string(),
        );
    }

    if matches!(tracking_health_tone, "warning" | "danger") {
        items.push(
            "Tracking health is degraded. Send a fresh ping or restart live tracking before the next leg transition."
                .to_string(),
        );
    }

    if document_count == 0 {
        items.push(
            "No execution paperwork is attached yet, so proof and exception handling are still weak."
                .to_string(),
        );
    }

    if note_count == 0 {
        items.push(
            "No execution notes have been captured yet, so operator context is still missing from the timeline."
                .to_string(),
        );
    }

    if !delivery_completion_ready {
        items.push(
            "Delivery closeout is still blocked until both a POD and an execution note are present."
                .to_string(),
        );
    }

    if !can_send_location_ping {
        items.push(
            "This session cannot send GPS directly, so the next location refresh depends on the booked carrier or ops."
                .to_string(),
        );
    } else if live_tracking_available
        && point_count > 0
        && !matches!(tracking_health_tone, "warning" | "danger")
    {
        items.push(
            "Live tracking is available here and should stay on while the truck is moving."
                .to_string(),
        );
    }

    if items.is_empty() {
        items.push(
            "No obvious operator blockers are open right now; this leg looks healthy from the Rust execution side."
                .to_string(),
        );
    }

    items
}

fn event_tone(event_type_key: &str) -> &'static str {
    match event_type_key {
        "pickup_started" | "pickup_arrived" => "primary",
        "pickup_departed" | "in_transit" => "warning",
        "delivery_arrived" | "delivery_completed" => "success",
        "location_ping" => "info",
        _ => "info",
    }
}

#[component]
pub fn ExecutionLegPage() -> impl IntoView {
    let auth = use_auth();
    let auth_for_admin_handoffs = auth.clone();
    let auth_for_desk_handoffs = auth.clone();
    let auth_for_payment_handoffs = auth.clone();
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
        let auth = auth.clone();

        spawn_local(async move {
            match api::fetch_execution_leg_screen(leg_id).await {
                Ok(next_screen) => {
                    error_message.set(None);
                    screen.set(Some(next_screen));
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
        let auth = auth.clone();

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
        let auth = auth.clone();

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
                    }
                    action_message.set(Some(error));
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
        let auth = auth.clone();

        spawn_local(async move {
            let result = async {
                let (lat, lng) = device_location::current_position().await?;
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
                    }
                    action_message.set(Some(error));
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
        let auth = auth.clone();

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
                    }
                    action_message.set(Some(error));
                }
            }

            is_uploading_document.set(false);
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
        let auth = auth.clone();

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
                <div style="display:grid;gap:0.35rem;">
                    <h2>{move || screen.get().map(|value| value.title).unwrap_or_else(|| "Execution Workspace".into())}</h2>
                    <p>{move || screen.get().map(|value| value.subtitle).unwrap_or_else(|| "Rust tracking and execution view".into())}</p>
                </div>
                <div style="display:flex;gap:0.75rem;flex-wrap:wrap;align-items:center;">
                    <A href="/loads" attr:style="padding:0.7rem 1rem;border-radius:0.9rem;background:#f4f4f5;color:#111827;text-decoration:none;">"Back to loads"</A>
                    <A href=move || screen.get().map(|value| format!("/loads/{}", value.load_id)).unwrap_or_else(|| "/loads".into()) attr:style="padding:0.7rem 1rem;border-radius:0.9rem;background:#111827;color:white;text-decoration:none;">"Open load profile"</A>
                </div>
            </section>

            {move || auth.session.get().user.map(|user| view! {
                <section style="padding:0.85rem 1rem;border:1px solid #dcfce7;border-radius:0.9rem;background:#f0fdf4;color:#166534;">
                    {format!("Authenticated as {} ({})", user.name, user.role_label)}
                </section>
            })}

            {move || action_message.get().map(|message| view! {
                <section style="padding:0.85rem 1rem;border:1px solid #dbeafe;border-radius:0.9rem;background:#eff6ff;color:#1d4ed8;">{message}</section>
            })}

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
                    let tracking_health_tone_for_hint = tracking_health_tone.clone();
                    let tracking_guidance = tracking_guidance_items(
                        tracking_point_count,
                        &tracking_health_tone,
                        screen_value.can_send_location_ping,
                        screen_value.live_tracking_available,
                        screen_value.delivery_completion_ready,
                    );
                    let next_action_label = screen_value
                        .next_action_label
                        .clone()
                        .unwrap_or_else(|| "Use the execution controls below to keep this leg moving.".into());
                    let latest_tracking_point = tracking_points
                        .iter()
                        .find(|point| point.is_latest)
                        .cloned()
                        .or_else(|| tracking_points.first().cloned());
                    let earliest_tracking_point = tracking_points.last().cloned();
                    let documents = screen_value.documents.clone();
                    let document_count = documents.len();
                    let (gps_focus_tone, gps_focus_label) =
                        gps_coverage_label(tracking_point_count, &tracking_health_tone);
                    let (document_focus_tone, document_focus_label) = document_readiness_label(
                        document_count,
                        screen_value.delivery_completion_ready,
                    );
                    let execution_blockers = execution_blocker_items(
                        tracking_point_count,
                        document_count,
                        note_count,
                        &tracking_health_tone,
                        screen_value.can_send_location_ping,
                        screen_value.live_tracking_available,
                        screen_value.delivery_completion_ready,
                    );
                    let document_type_options = screen_value.document_type_options.clone();
                    let pickup_bol_count =
                        count_execution_documents_by_type(&documents, "pickup_bol");
                    let pickup_photo_count =
                        count_execution_documents_by_type(&documents, "pickup_photo");
                    let delivery_pod_count =
                        count_execution_documents_by_type(&documents, "delivery_pod");
                    let delivery_photo_count =
                        count_execution_documents_by_type(&documents, "delivery_photo");
                    let other_document_count =
                        count_execution_documents_by_type(&documents, "other");
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
                                    <small style="color:#64748b;">"Execution status"</small>
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
                                    label="Delivery Readiness"
                                    value=if screen_value.delivery_completion_ready {
                                        "Ready for completion".into()
                                    } else {
                                        "Waiting on POD and note".into()
                                    }
                                />
                            </section>

                            <section style="display:grid;grid-template-columns:repeat(auto-fit,minmax(260px,1fr));gap:1rem;align-items:start;">
                                <section style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#ffffff;display:grid;gap:0.75rem;">
                                    <div style="display:flex;justify-content:space-between;gap:0.75rem;align-items:center;flex-wrap:wrap;">
                                        <strong>"Operator readiness"</strong>
                                        <span style=tone_style(gps_focus_tone)>{gps_focus_label.clone()}</span>
                                    </div>
                                    <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(160px,1fr));gap:0.75rem;">
                                        <div style="padding:0.8rem 0.9rem;border:1px solid #e5e7eb;border-radius:0.9rem;background:#fcfcfb;display:grid;gap:0.15rem;">
                                            <strong>"GPS coverage"</strong>
                                            <small style="color:#64748b;">{gps_focus_label.clone()}</small>
                                        </div>
                                        <div style="padding:0.8rem 0.9rem;border:1px solid #e5e7eb;border-radius:0.9rem;background:#fcfcfb;display:grid;gap:0.15rem;">
                                            <strong>"Document pack"</strong>
                                            <span style=tone_style(document_focus_tone)>{document_focus_label.clone()}</span>
                                        </div>
                                        <div style="padding:0.8rem 0.9rem;border:1px solid #e5e7eb;border-radius:0.9rem;background:#fcfcfb;display:grid;gap:0.15rem;">
                                            <strong>"Realtime path"</strong>
                                            <small style="color:#64748b;">
                                                {if screen_value.live_tracking_available {
                                                    "This role can keep live route updates flowing from this page."
                                                } else {
                                                    "This leg is view-first here, so location refresh depends on the booked carrier or ops."
                                                }}
                                            </small>
                                        </div>
                                        <div style="padding:0.8rem 0.9rem;border:1px solid #e5e7eb;border-radius:0.9rem;background:#fcfcfb;display:grid;gap:0.15rem;">
                                            <strong>"Closeout gate"</strong>
                                            <small style="color:#64748b;">{if screen_value.delivery_completion_ready {
                                                "Delivery closeout prerequisites are ready."
                                            } else {
                                                "POD and execution note are still required for clean closeout."
                                            }}</small>
                                        </div>
                                    </div>
                                </section>

                                <section style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#ffffff;display:grid;gap:0.6rem;">
                                    <div style="display:flex;justify-content:space-between;gap:0.75rem;align-items:center;flex-wrap:wrap;">
                                        <strong>"Operator blocker checklist"</strong>
                                        <span style=tone_style(if execution_blockers.len() > 1 || !execution_blockers.first().is_some_and(|item| item.contains("No obvious operator blockers")) { "warning" } else { "success" })>
                                            {if execution_blockers.first().is_some_and(|item| item.contains("No obvious operator blockers")) {
                                                "Clear"
                                            } else {
                                                "Needs attention"
                                            }}
                                        </span>
                                    </div>
                                    <ul style="margin:0;padding-left:1.1rem;display:grid;gap:0.35rem;color:#475569;">
                                        {execution_blockers.into_iter().map(|item| view! { <li>{item}</li> }).collect_view()}
                                    </ul>
                                    <div style="display:flex;gap:0.5rem;flex-wrap:wrap;">
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
                                    </div>
                                </section>
                            </section>

                            <section style="display:grid;grid-template-columns:repeat(auto-fit,minmax(260px,1fr));gap:1rem;align-items:start;">
                                <section style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#ffffff;display:grid;gap:0.75rem;">
                                    <div style="display:flex;justify-content:space-between;gap:0.75rem;align-items:center;flex-wrap:wrap;">
                                        <strong>"Execution closeout checklist"</strong>
                                        <span style=tone_style(if screen_value.delivery_completion_ready { "success" } else { "warning" })>
                                            {if screen_value.delivery_completion_ready {
                                                "Ready"
                                            } else {
                                                "Still collecting"
                                            }}
                                        </span>
                                    </div>
                                    <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(150px,1fr));gap:0.75rem;">
                                        <div style="padding:0.8rem 0.9rem;border:1px solid #e5e7eb;border-radius:0.9rem;background:#fcfcfb;display:grid;gap:0.2rem;">
                                            <strong>"Pickup docs"</strong>
                                            <span style=tone_style(if pickup_bol_count + pickup_photo_count > 0 { "success" } else { "warning" })>
                                                {format!("{} file(s)", pickup_bol_count + pickup_photo_count)}
                                            </span>
                                            <small style="color:#64748b;">"Pickup BOL and pickup photos stay visible here for exception handling."</small>
                                        </div>
                                        <div style="padding:0.8rem 0.9rem;border:1px solid #e5e7eb;border-radius:0.9rem;background:#fcfcfb;display:grid;gap:0.2rem;">
                                            <strong>"Delivery POD"</strong>
                                            <span style=tone_style(if delivery_pod_count > 0 { "success" } else { "danger" })>
                                                {format!("{} file(s)", delivery_pod_count)}
                                            </span>
                                            <small style="color:#64748b;">"At least one POD is required before delivery completion can close cleanly."</small>
                                        </div>
                                        <div style="padding:0.8rem 0.9rem;border:1px solid #e5e7eb;border-radius:0.9rem;background:#fcfcfb;display:grid;gap:0.2rem;">
                                            <strong>"Delivery photos"</strong>
                                            <span style=tone_style(if delivery_photo_count > 0 { "success" } else { "info" })>
                                                {format!("{} file(s)", delivery_photo_count)}
                                            </span>
                                            <small style="color:#64748b;">"Photos are optional, but they help when closeout or claims review gets noisy."</small>
                                        </div>
                                        <div style="padding:0.8rem 0.9rem;border:1px solid #e5e7eb;border-radius:0.9rem;background:#fcfcfb;display:grid;gap:0.2rem;">
                                            <strong>"Operator notes"</strong>
                                            <span style=tone_style(if note_count > 0 { "success" } else { "warning" })>
                                                {format!("{} note(s)", note_count)}
                                            </span>
                                            <small style="color:#64748b;">"A clear note trail makes admin closeout and carrier follow-up much safer."</small>
                                        </div>
                                        <div style="padding:0.8rem 0.9rem;border:1px solid #e5e7eb;border-radius:0.9rem;background:#fcfcfb;display:grid;gap:0.2rem;">
                                            <strong>"Route trace"</strong>
                                            <span style=tone_style(gps_focus_tone)>{gps_focus_label.clone()}</span>
                                            <small style="color:#64748b;">{tracking_window_label.clone()}</small>
                                        </div>
                                        <div style="padding:0.8rem 0.9rem;border:1px solid #e5e7eb;border-radius:0.9rem;background:#fcfcfb;display:grid;gap:0.2rem;">
                                            <strong>"Other docs"</strong>
                                            <span style=tone_style(if other_document_count > 0 { "info" } else { "dark" })>
                                                {format!("{} file(s)", other_document_count)}
                                            </span>
                                            <small style="color:#64748b;">"Use this lane for extra paperwork that does not fit the pickup or delivery buckets."</small>
                                        </div>
                                    </div>
                                </section>

                                <section style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#ffffff;display:grid;gap:0.75rem;">
                                    <div style="display:flex;justify-content:space-between;gap:0.75rem;align-items:center;flex-wrap:wrap;">
                                        <strong>"Workspace handoff"</strong>
                                        <span style=tone_style("info")>{screen_value.operator_mode_label.clone()}</span>
                                    </div>
                                    <small style="color:#64748b;">
                                        "This keeps the Rust execution page connected to the operator boards that normally take over after tracking, proof, or closeout starts to matter."
                                    </small>
                                    <div style="display:flex;gap:0.5rem;flex-wrap:wrap;">
                                        <A href=format!("/loads/{}", screen_value.load_id) attr:style="padding:0.5rem 0.8rem;border-radius:0.8rem;background:#111827;color:white;text-decoration:none;">
                                            "User load profile"
                                        </A>
                                        {can_open_admin_handoffs.get().then(|| view! {
                                            <A href=format!("/admin/loads/{}", screen_value.load_id) attr:style="padding:0.5rem 0.8rem;border-radius:0.8rem;background:#eef2ff;color:#312e81;text-decoration:none;">
                                                "Admin load profile"
                                            </A>
                                        })}
                                        {can_open_desk_handoffs.get().then(|| view! {
                                            <A href=desk_handoff.0 attr:style="padding:0.5rem 0.8rem;border-radius:0.8rem;background:#fff7dd;color:#92400e;text-decoration:none;">
                                                {desk_handoff.1}
                                            </A>
                                        })}
                                        {can_open_payment_handoffs.get().then(|| view! {
                                            <A href=format!("/admin/payments?leg_id={}&source=execution", screen_value.leg_id) attr:style="padding:0.5rem 0.8rem;border-radius:0.8rem;background:#e8fff3;color:#166534;text-decoration:none;">
                                                "Payments console"
                                            </A>
                                        })}
                                    </div>
                                    <small style="color:#64748b;">
                                        {if screen_value.delivery_completion_ready {
                                            "Proof is in place, so closeout and finance follow-up can start without waiting on another document pass."
                                        } else if delivery_pod_count == 0 {
                                            "Stay in execution until POD is uploaded, then hand the leg off to closeout and finance."
                                        } else {
                                            "Execution still owns the active follow-up, but the next workspace is already linked here."
                                        }}
                                    </small>
                                </section>
                            </section>

                            <section style="display:grid;grid-template-columns:repeat(auto-fit,minmax(320px,1fr));gap:1rem;align-items:start;">
                                <section style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#ffffff;display:grid;gap:0.85rem;">
                                    <div style="padding:0.85rem 1rem;border:1px solid #dbeafe;border-radius:0.9rem;background:#eff6ff;display:grid;gap:0.35rem;">
                                        <div style="display:flex;justify-content:space-between;gap:1rem;align-items:center;flex-wrap:wrap;">
                                            <div style="display:grid;gap:0.2rem;">
                                                <strong>{screen_value.operator_mode_label.clone()}</strong>
                                                {screen_value.live_tracking_note.clone().map(|note| view! {
                                                    <small style="color:#1d4ed8;">{note}</small>
                                                })}
                                            </div>
                                            <span style=move || tone_style(if live_tracking_enabled.get() { "success" } else if screen_value.live_tracking_available { "warning" } else { "info" })>
                                                {move || if live_tracking_enabled.get() {
                                                    "Tracking: ON".to_string()
                                                } else if screen_value.live_tracking_available {
                                                    "Tracking: Ready".to_string()
                                                } else {
                                                    "Tracking: View only".to_string()
                                                }}
                                            </span>
                                        </div>
                                        {screen_value.live_tracking_available.then(|| view! {
                                            <div style="display:flex;gap:0.5rem;flex-wrap:wrap;">
                                                <button
                                                    type="button"
                                                    style="padding:0.55rem 0.85rem;border-radius:0.8rem;border:none;background:#166534;color:white;cursor:pointer;"
                                                    disabled=move || is_toggling_live_tracking.get() || live_tracking_enabled.get()
                                                    on:click=start_live_tracking
                                                >
                                                    {move || if is_toggling_live_tracking.get() && !live_tracking_enabled.get() { "Starting..." } else { "Start live tracking" }}
                                                </button>
                                                <button
                                                    type="button"
                                                    style="padding:0.55rem 0.85rem;border-radius:0.8rem;border:none;background:#475569;color:white;cursor:pointer;"
                                                    disabled=move || is_toggling_live_tracking.get() || !live_tracking_enabled.get()
                                                    on:click=stop_live_tracking
                                                >
                                                    {move || if is_toggling_live_tracking.get() && live_tracking_enabled.get() { "Stopping..." } else { "Stop live tracking" }}
                                                </button>
                                            </div>
                                        })}
                                    </div>

                                    <div style="display:flex;justify-content:space-between;gap:1rem;align-items:flex-start;flex-wrap:wrap;">
                                        <div style="display:grid;gap:0.3rem;">
                                            <strong>"Execution actions"</strong>
                                            <small style="color:#64748b;">"The action list follows the same pickup-to-delivery sequence the Laravel tracking page uses."</small>
                                        </div>
                                        <button
                                            type="button"
                                            style="padding:0.55rem 0.85rem;border-radius:0.8rem;border:none;background:#0f766e;color:white;cursor:pointer;"
                                            disabled=move || is_sending_location.get() || !screen_value.can_send_location_ping
                                            on:click=send_current_location
                                        >
                                            {move || if is_sending_location.get() { "Sending GPS..." } else if screen_value.can_send_location_ping { "Send current GPS" } else { "GPS locked" }}
                                        </button>
                                    </div>
                                    <div style="display:grid;gap:0.35rem;padding:0.85rem 1rem;border:1px solid #e5e7eb;border-radius:0.9rem;background:#fcfcfb;">
                                        <strong>"Latest location"</strong>
                                        <span>{screen_value.latest_location_label.clone().unwrap_or_else(|| "No location ping yet".into())}</span>
                                        <small>{screen_value.latest_coordinate_label.clone().unwrap_or_else(|| "Waiting for the first GPS update".into())}</small>
                                        {screen_value.tracking_summary_label.clone().map(|summary| view! {
                                            <small style="color:#64748b;">{summary}</small>
                                        })}
                                        <small style="color:#64748b;">{tracking_window_label.clone()}</small>
                                        <span style=tone_style(&tracking_health_tone)>{screen_value.tracking_health_tone.replace('_', " ")}</span>
                                        <small style="color:#64748b;">{tracking_health_label.clone()}</small>
                                        {move || {
                                            if screen_value.can_send_location_ping
                                                && matches!(tracking_health_tone_for_hint.as_str(), "warning" | "danger")
                                            {
                                                view! {
                                                    <small style="color:#92400e;">
                                                        "If the route has gone quiet, send a fresh GPS update now or restart live tracking before moving to the next execution step."
                                                    </small>
                                                }.into_any()
                                            } else {
                                                view! { <></> }.into_any()
                                            }
                                        }}
                                        {screen_value.latest_map_url.clone().map(|map_url| view! {
                                            <a href=map_url target="_blank" rel="noopener noreferrer" style="justify-self:start;padding:0.45rem 0.75rem;border-radius:0.75rem;background:#e0f2fe;color:#075985;text-decoration:none;">
                                                "Open latest point in Google Maps"
                                            </a>
                                        })}
                                    </div>
                                    <div style="display:grid;gap:0.35rem;padding:0.85rem 1rem;border:1px solid #e5e7eb;border-radius:0.9rem;background:#fcfcfb;">
                                        <strong>"Recommended next step"</strong>
                                        <small style="color:#64748b;">{next_action_label.clone()}</small>
                                    </div>
                                    <div style="display:grid;gap:0.4rem;padding:0.85rem 1rem;border:1px solid #e5e7eb;border-radius:0.9rem;background:#fcfcfb;">
                                        <strong>"Execution note"</strong>
                                        <textarea
                                            rows="3"
                                            placeholder="Add carrier or operator context for the next execution step"
                                            prop:value=move || action_note.get()
                                            on:input=move |ev| action_note.set(event_target_value(&ev))
                                            disabled=move || pending_action_key.get().is_some()
                                        />
                                        {screen_value.delivery_completion_note.clone().map(|note| view! {
                                            <small style=move || if screen_value.delivery_completion_ready { "color:#166534;" } else { "color:#b45309;" }>{note}</small>
                                        })}
                                    </div>
                                    <div style="display:grid;gap:0.75rem;">
                                        {action_items.into_iter().map(|item| render_action_item(item, pending_action_key, run_action)).collect_view()}
                                    </div>
                                </section>

                                <section style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#ffffff;display:grid;gap:0.85rem;overflow:auto;">
                                    <div style="display:grid;gap:0.3rem;">
                                        <strong>"Live location map"</strong>
                                        <small style="color:#64748b;">"The Rust execution view now keeps the map context inline so operators do not have to bounce back to the Blade tracking page for every GPS check."</small>
                                    </div>
                                    {tracking_embed.clone().map(|embed_url| view! {
                                        <iframe
                                            src=embed_url
                                            style="width:100%;min-height:320px;border:1px solid #e5e7eb;border-radius:0.95rem;background:#f8fafc;"
                                        ></iframe>
                                    }.into_any()).unwrap_or_else(|| view! {
                                        <section style="padding:1rem;border:1px dashed #cbd5e1;border-radius:0.95rem;background:#f8fafc;display:grid;gap:0.35rem;">
                                            <strong>"No live map yet"</strong>
                                            <small style="color:#64748b;">"Once the driver sends the first GPS ping, the Rust execution workspace will frame the active leg here."</small>
                                        </section>
                                    }.into_any())}
                                    {latest_tracking_point.clone().map(|point| view! {
                                        <div style="display:flex;justify-content:space-between;gap:0.75rem;align-items:center;flex-wrap:wrap;padding:0.75rem 0.9rem;border:1px solid #e5e7eb;border-radius:0.9rem;background:#fcfcfb;">
                                            <div style="display:grid;gap:0.2rem;">
                                                <strong>"Latest plotted point"</strong>
                                                <small>{point.recorded_at_label.clone()}</small>
                                                <small style="color:#64748b;">{format!("{:.5}, {:.5}", point.lat, point.lng)}</small>
                                            </div>
                                            <a href=tracking_point_map_url(&point) target="_blank" rel="noopener noreferrer" style="padding:0.45rem 0.75rem;border-radius:0.75rem;background:#e0f2fe;color:#075985;text-decoration:none;">
                                                "Open exact point"
                                            </a>
                                        </div>
                                    })}
                                    {earliest_tracking_point.clone().map(|point| view! {
                                        <div style="display:flex;justify-content:space-between;gap:0.75rem;align-items:center;flex-wrap:wrap;padding:0.75rem 0.9rem;border:1px solid #e5e7eb;border-radius:0.9rem;background:#fcfcfb;">
                                            <div style="display:grid;gap:0.2rem;">
                                                <strong>"First plotted point"</strong>
                                                <small>{point.recorded_at_label.clone()}</small>
                                                <small style="color:#64748b;">{format!("{:.5}, {:.5}", point.lat, point.lng)}</small>
                                            </div>
                                            <a href=tracking_point_map_url(&point) target="_blank" rel="noopener noreferrer" style="padding:0.45rem 0.75rem;border-radius:0.75rem;background:#f1f5f9;color:#0f172a;text-decoration:none;">
                                                "Open starting point"
                                            </a>
                                        </div>
                                    })}
                                    {earliest_tracking_point.clone().zip(latest_tracking_point.clone()).map(|(start, end)| view! {
                                        <div style="display:flex;justify-content:space-between;gap:0.75rem;align-items:center;flex-wrap:wrap;padding:0.75rem 0.9rem;border:1px solid #e5e7eb;border-radius:0.9rem;background:#fcfcfb;">
                                            <div style="display:grid;gap:0.2rem;">
                                                <strong>"Route handoff"</strong>
                                                <small style="color:#64748b;">"Open the first-to-latest GPS span in Google Maps for a quick operator route check."</small>
                                            </div>
                                            <a href=tracking_route_maps_url(&start, &end) target="_blank" rel="noopener noreferrer" style="padding:0.45rem 0.75rem;border-radius:0.75rem;background:#ede9fe;color:#5b21b6;text-decoration:none;">
                                                "Open route span"
                                            </a>
                                        </div>
                                    })}
                                    <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:0.75rem;">
                                        <div style="padding:0.8rem 0.9rem;border:1px solid #e5e7eb;border-radius:0.9rem;background:#fcfcfb;display:grid;gap:0.15rem;">
                                            <strong>"Tracking session"</strong>
                                            <small style="color:#64748b;">{tracking_window_label.clone()}</small>
                                        </div>
                                        <div style="padding:0.8rem 0.9rem;border:1px solid #e5e7eb;border-radius:0.9rem;background:#fcfcfb;display:grid;gap:0.15rem;">
                                            <strong>"Approx. distance"</strong>
                                            <small style="color:#64748b;">{tracking_distance_label.clone()}</small>
                                        </div>
                                        <div style="padding:0.8rem 0.9rem;border:1px solid #e5e7eb;border-radius:0.9rem;background:#fcfcfb;display:grid;gap:0.15rem;">
                                            <strong>"Tracking health"</strong>
                                            <span style=tone_style(&tracking_health_tone)>{screen_value.tracking_health_tone.replace('_', " ")}</span>
                                        </div>
                                        <div style="padding:0.8rem 0.9rem;border:1px solid #e5e7eb;border-radius:0.9rem;background:#fcfcfb;display:grid;gap:0.15rem;">
                                            <strong>"Next step"</strong>
                                            <small style="color:#64748b;">{next_action_label.clone()}</small>
                                        </div>
                                    </div>
                                    {(!tracking_guidance.is_empty()).then(|| view! {
                                        <div style="display:grid;gap:0.45rem;padding:0.85rem 1rem;border:1px solid #dbeafe;border-radius:0.9rem;background:#eff6ff;">
                                            <strong>"Tracking guidance"</strong>
                                            <ul style="margin:0;padding-left:1.1rem;display:grid;gap:0.3rem;color:#1d4ed8;">
                                                {tracking_guidance.into_iter().map(|item| view! { <li>{item}</li> }).collect_view()}
                                            </ul>
                                        </div>
                                    })}
                                    <strong>"Tracking points"</strong>
                                    {render_tracking_points(tracking_points)}
                                </section>
                            </section>

                            <section style="display:grid;grid-template-columns:repeat(auto-fit,minmax(320px,1fr));gap:1rem;align-items:start;">
                                <section style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#ffffff;display:grid;gap:0.85rem;overflow:auto;">
                                    <strong>"Timeline"</strong>
                                    {render_timeline(timeline)}
                                </section>
                                <section style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#ffffff;display:grid;gap:0.85rem;overflow:auto;">
                                    <strong>"Execution note history"</strong>
                                    {render_execution_notes(notes_history)}
                                </section>
                            </section>

                            <section style="display:grid;grid-template-columns:repeat(auto-fit,minmax(320px,1fr));gap:1rem;align-items:start;">
                                <section style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#ffffff;display:grid;gap:0.85rem;overflow:auto;">
                                    <div style="display:grid;gap:0.3rem;">
                                        <strong>"Execution documents"</strong>
                                        <small style="color:#64748b;">"Attach pickup and delivery paperwork directly to the leg so the carrier, shipper, and admin workflow stays off the old Laravel page."</small>
                                        <small style="color:#64748b;">{format!("{} execution document(s) currently attached.", document_count)}</small>
                                    </div>
                                    <div style="display:grid;gap:0.65rem;padding:0.85rem 1rem;border:1px solid #e5e7eb;border-radius:0.9rem;background:#fcfcfb;">
                                        <strong>"Upload document"</strong>
                                        <input
                                            type="text"
                                            placeholder="Optional custom document name"
                                            prop:value=move || upload_document_name.get()
                                            on:input=move |ev| upload_document_name.set(event_target_value(&ev))
                                            disabled=move || !screen_value.can_upload_documents || is_uploading_document.get()
                                        />
                                        <select
                                            prop:value=move || upload_document_type.get()
                                            on:change=move |ev| upload_document_type.set(event_target_value(&ev))
                                            disabled=move || !screen_value.can_upload_documents || is_uploading_document.get()
                                        >
                                            {document_type_options.into_iter().map(render_document_type_option).collect_view()}
                                        </select>
                                        <input
                                            id=document_upload::execution_upload_input_id(screen_value.leg_id)
                                            type="file"
                                            disabled=move || !screen_value.can_upload_documents || is_uploading_document.get()
                                        />
                                        <button
                                            type="button"
                                            style="padding:0.55rem 0.85rem;border-radius:0.8rem;border:none;background:#111827;color:white;cursor:pointer;justify-self:start;"
                                            disabled=move || !screen_value.can_upload_documents || is_uploading_document.get()
                                            on:click=move |_| upload_execution_document()
                                        >
                                            {move || if is_uploading_document.get() { "Uploading..." } else if screen_value.can_upload_documents { "Upload execution document" } else { "Upload locked" }}
                                        </button>
                                    </div>
                                    {render_documents(documents, open_document)}
                                </section>
                            </section>

                            <section style="display:grid;gap:0.35rem;">
                                {screen_value.notes.into_iter().map(|note| view! { <p style="margin:0;">{note}</p> }).collect_view()}
                            </section>
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

fn render_action_item(
    item: ExecutionActionItem,
    pending_action_key: RwSignal<Option<String>>,
    run_action: impl Fn(String) + Copy + 'static,
) -> impl IntoView {
    let item_key = item.key.clone();
    let is_pending = Signal::derive(move || pending_action_key.get() == Some(item_key.clone()));
    let button_label = item.label.clone();
    let action_key = item.key.clone();
    let description = item.description.clone();
    let is_enabled = item.is_enabled;

    view! {
        <div style="padding:0.85rem 1rem;border:1px solid #e5e7eb;border-radius:0.95rem;background:#fcfcfb;display:grid;gap:0.45rem;">
            <strong>{button_label.clone()}</strong>
            <small style="color:#64748b;">{description}</small>
            <button
                type="button"
                style="padding:0.55rem 0.85rem;border-radius:0.8rem;border:none;background:#111827;color:white;cursor:pointer;justify-self:start;"
                disabled=move || is_pending.get() || !is_enabled
                on:click=move |_| run_action(action_key.clone())
            >
                {move || if is_pending.get() { "Working...".to_string() } else if is_enabled { button_label.clone() } else { "Unavailable".to_string() }}
            </button>
        </div>
    }
}

fn render_tracking_points(points: Vec<ExecutionTrackingPointItem>) -> AnyView {
    if points.is_empty() {
        view! { <p style="margin:0;">"No GPS points have been stored for this leg yet."</p> }
            .into_any()
    } else {
        view! {
            <table style="width:100%;border-collapse:collapse;min-width:420px;">
                <thead style="background:#f8fafc;">
                    <tr>
                        <th style="text-align:left;padding:0.75rem;">"Recorded"</th>
                        <th style="text-align:left;padding:0.75rem;">"Coordinates"</th>
                        <th style="text-align:left;padding:0.75rem;">"State"</th>
                    </tr>
                </thead>
                <tbody>
                    {points.into_iter().map(|point| view! {
                        <tr style="border-top:1px solid #f1f5f9;vertical-align:top;">
                            <td style="padding:0.75rem;">{point.recorded_at_label.clone()}</td>
                            <td style="padding:0.75rem;">{format!("{:.5}, {:.5}", point.lat, point.lng)}</td>
                            <td style="padding:0.75rem;display:grid;gap:0.35rem;">
                                <span>{if point.is_latest { "Latest" } else { "Historical" }}</span>
                                <a href=tracking_point_map_url(&point) target="_blank" rel="noopener noreferrer" style="color:#0f766e;text-decoration:none;">
                                    "Open point"
                                </a>
                            </td>
                        </tr>
                    }).collect_view()}
                </tbody>
            </table>
        }.into_any()
    }
}

fn render_timeline(items: Vec<ExecutionTimelineItem>) -> AnyView {
    if items.is_empty() {
        view! { <p style="margin:0;">"No execution events are recorded yet."</p> }.into_any()
    } else {
        view! {
            <div style="display:grid;gap:0.7rem;">
                {items.into_iter().map(|item| view! {
                    <div style="padding:0.8rem 0.95rem;border:1px solid #e5e7eb;border-radius:0.9rem;background:#fcfcfb;display:grid;gap:0.15rem;">
                        <div style="display:flex;justify-content:space-between;gap:0.75rem;align-items:center;flex-wrap:wrap;">
                            <strong>{item.event_type_label}</strong>
                            <span style=tone_style(event_tone(&item.event_type_key))>{item.event_type_key.replace('_', " ")}</span>
                        </div>
                        <small>{item.created_at_label}</small>
                    </div>
                }).collect_view()}
            </div>
        }.into_any()
    }
}

fn render_documents(
    items: Vec<ExecutionDocumentItem>,
    open_document: impl Fn(String) + Copy + 'static,
) -> AnyView {
    if items.is_empty() {
        view! { <p style="margin:0;">"No execution-stage documents have been attached to this leg yet."</p> }.into_any()
    } else {
        view! {
            <div style="display:grid;gap:0.7rem;">
                {items.into_iter().map(|item| view! {
                    <div style="padding:0.8rem 0.95rem;border:1px solid #e5e7eb;border-radius:0.9rem;background:#fcfcfb;display:grid;gap:0.15rem;">
                        <strong>{item.document_type_label}</strong>
                        <small>{item.file_label}</small>
                        <small style="color:#64748b;">{item.uploaded_by_label.unwrap_or_else(|| "Uploader not recorded".into())}</small>
                        <small>{item.created_at_label}</small>
                        {item.download_path.clone().map(|path| {
                            if item.can_view_file {
                                view! {
                                    <button
                                        type="button"
                                        style="padding:0.45rem 0.75rem;border-radius:0.75rem;border:none;background:#0f766e;color:white;cursor:pointer;justify-self:start;"
                                        on:click=move |_| open_document(path.clone())
                                    >
                                        "View file"
                                    </button>
                                }.into_any()
                            } else {
                                view! { <small>"File binary is restricted to admin and the uploader."</small> }.into_any()
                            }
                        }).unwrap_or_else(|| view! { <small>"File binary is restricted to admin and the uploader."</small> }.into_any())}
                    </div>
                }).collect_view()}
            </div>
        }.into_any()
    }
}

fn render_execution_notes(items: Vec<ExecutionNoteItem>) -> AnyView {
    if items.is_empty() {
        view! { <p style="margin:0;">"No execution notes have been captured yet for this leg."</p> }
            .into_any()
    } else {
        view! {
            <div style="display:grid;gap:0.7rem;">
                {items.into_iter().map(|item| view! {
                    <div style="padding:0.8rem 0.95rem;border:1px solid #e5e7eb;border-radius:0.9rem;background:#fcfcfb;display:grid;gap:0.2rem;">
                        <strong>{item.actor_label}</strong>
                        <small style="color:#64748b;">{item.status_label}</small>
                        <span>{item.remarks_label}</span>
                        <small>{item.created_at_label}</small>
                    </div>
                }).collect_view()}
            </div>
        }.into_any()
    }
}

fn render_document_type_option(option: ExecutionDocumentTypeOption) -> impl IntoView {
    view! {
        <option value=option.key>{format!("{} - {}", option.label, option.description)}</option>
    }
}

#[component]
fn InfoCard(label: &'static str, value: String) -> impl IntoView {
    view! {
        <div style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#ffffff;display:grid;gap:0.4rem;">
            <small style="color:#64748b;">{label}</small>
            <strong>{value}</strong>
        </div>
    }
}
