use leptos::{prelude::*, tachys::view::any_view::IntoAny};

use shared::{
    ExecutionActionItem, ExecutionDocumentItem, ExecutionDocumentTypeOption, ExecutionNoteItem,
    ExecutionStatusItem, ExecutionTimelineItem, ExecutionTrackingPointItem,
};

pub(super) fn tone_style(tone: &str) -> &'static str {
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

pub(super) fn tracking_embed_url(points: &[ExecutionTrackingPointItem]) -> Option<String> {
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

pub(super) fn tracking_point_map_url(point: &ExecutionTrackingPointItem) -> String {
    format!(
        "https://www.google.com/maps?q={:.6},{:.6}",
        point.lat, point.lng
    )
}

pub(super) fn tracking_route_maps_url(
    start: &ExecutionTrackingPointItem,
    end: &ExecutionTrackingPointItem,
) -> String {
    format!(
        "https://www.google.com/maps/dir/{:.6},{:.6}/{:.6},{:.6}",
        start.lat, start.lng, end.lat, end.lng
    )
}

pub(super) fn haversine_km(
    first_lat: f64,
    first_lng: f64,
    second_lat: f64,
    second_lng: f64,
) -> f64 {
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

pub(super) fn tracking_distance_km(points: &[ExecutionTrackingPointItem]) -> Option<f64> {
    if points.len() < 2 {
        return None;
    }

    let total = points
        .windows(2)
        .map(|window| haversine_km(window[0].lat, window[0].lng, window[1].lat, window[1].lng))
        .sum::<f64>();

    Some(total)
}

pub(super) fn tracking_window_summary(points: &[ExecutionTrackingPointItem]) -> Option<String> {
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

pub(super) fn event_tone(event_type_key: &str) -> &'static str {
    match event_type_key {
        "pickup_started" | "pickup_arrived" => "primary",
        "pickup_departed" | "in_transit" => "warning",
        "delivery_arrived" | "delivery_completed" => "success",
        "location_ping" => "info",
        _ => "info",
    }
}

pub(super) fn render_action_item(
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

pub(super) fn render_tracking_points(points: Vec<ExecutionTrackingPointItem>) -> AnyView {
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

pub(super) fn render_timeline(items: Vec<ExecutionTimelineItem>) -> AnyView {
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

pub(super) fn render_documents(
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
                        <small>{format!("{} | {}", item.version_history_label, item.created_at_label)}</small>
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

pub(super) fn render_execution_notes(items: Vec<ExecutionNoteItem>) -> AnyView {
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

pub(super) fn render_status_items(items: Vec<ExecutionStatusItem>) -> AnyView {
    view! {
        <div style="display:grid;gap:0.7rem;">
            {items.into_iter().map(|item| view! {
                <div style="padding:0.8rem 0.95rem;border:1px solid #e5e7eb;border-radius:0.9rem;background:#fcfcfb;display:grid;gap:0.2rem;">
                    <div style="display:flex;justify-content:space-between;gap:0.75rem;align-items:center;flex-wrap:wrap;">
                        <strong>{item.label}</strong>
                        <span style=tone_style(&item.status_tone)>{item.status_label}</span>
                    </div>
                    <small style="color:#64748b;">{item.detail}</small>
                </div>
            }).collect_view()}
        </div>
    }.into_any()
}

pub(super) fn render_document_type_option(option: ExecutionDocumentTypeOption) -> impl IntoView {
    view! {
        <option value=option.key>{format!("{} - {} {}", option.label, option.description, option.mobile_capture_hint)}</option>
    }
}

#[allow(clippy::too_many_arguments)]
#[component]
pub(super) fn InfoCard(label: &'static str, value: String) -> impl IntoView {
    view! {
        <div style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#ffffff;display:grid;gap:0.4rem;">
            <small style="color:#64748b;">{label}</small>
            <strong>{value}</strong>
        </div>
    }
}
