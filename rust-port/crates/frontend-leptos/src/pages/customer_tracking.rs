use leptos::{prelude::*, tachys::view::any_view::IntoAny, task::spawn_local};
use leptos_router::hooks::use_params_map;
use shared::ExecutionCustomerTrackingScreen;

use crate::api;

#[component]
pub fn CustomerTrackingPage() -> impl IntoView {
    let params = use_params_map();
    let share_token = Memo::new(move |_| params.with(|map| map.get("share_token")));
    let screen = RwSignal::new(None::<ExecutionCustomerTrackingScreen>);
    let error_message = RwSignal::new(None::<String>);
    let is_loading = RwSignal::new(false);

    Effect::new(move |_| {
        let Some(token) = share_token.get() else {
            error_message.set(Some("Tracking link is missing or expired.".into()));
            return;
        };

        is_loading.set(true);
        spawn_local(async move {
            match api::fetch_customer_tracking_screen(&token).await {
                Ok(value) => {
                    screen.set(Some(value));
                    error_message.set(None);
                }
                Err(error) => error_message.set(Some(error)),
            }
            is_loading.set(false);
        });
    });

    view! {
        <article style="min-height:100vh;background:#f8fafc;padding:1rem;display:grid;place-items:center;">
            <section style="width:min(48rem,100%);display:grid;gap:1rem;">
                <div style="display:grid;gap:0.3rem;">
                    <small style="color:#64748b;">"STLoads shipment tracking"</small>
                    <h1 style="margin:0;font-size:clamp(1.6rem,4vw,2.5rem);">"Shipment progress"</h1>
                </div>
                {move || error_message.get().map(|message| view! {
                    <section style="padding:0.85rem 1rem;border:1px solid #fecaca;border-radius:0.75rem;background:#fff1f2;color:#be123c;">{message}</section>
                })}
                {move || {
                    if is_loading.get() && screen.get().is_none() {
                        view! { <section style="padding:1rem;background:white;border:1px solid #e5e7eb;border-radius:0.75rem;">"Loading tracking..."</section> }.into_any()
                    } else if let Some(value) = screen.get() {
                        view! {
                            <section style="display:grid;gap:1rem;">
                                <div style="padding:1rem;background:white;border:1px solid #e5e7eb;border-radius:0.75rem;display:grid;gap:0.5rem;">
                                    <strong>{value.leg_code}</strong>
                                    <span>{value.route_label}</span>
                                    <small style="color:#64748b;">{value.load_number.unwrap_or_else(|| "Load number pending".into())}</small>
                                </div>
                                <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:0.75rem;">
                                    <TrackingCard label="Status" value=value.status_label />
                                    <TrackingCard label="Latest update" value=value.latest_location_label.unwrap_or_else(|| "No GPS update yet".into()) />
                                    <TrackingCard label="Coordinates" value=value.latest_coordinate_label.unwrap_or_else(|| "Hidden until available".into()) />
                                    <TrackingCard label="Expires" value=value.expires_at_label />
                                </div>
                                <div style="padding:1rem;background:white;border:1px solid #e5e7eb;border-radius:0.75rem;display:grid;gap:0.6rem;">
                                    <strong>"Tracking signals"</strong>
                                    <small style="color:#64748b;">{value.tracking_health_label.unwrap_or_else(|| "Tracking health pending.".into())}</small>
                                    <small style="color:#64748b;">{value.geofence_status_label.unwrap_or_else(|| "Geofence signal pending.".into())}</small>
                                    <small style="color:#64748b;">{value.eta_risk_label.unwrap_or_else(|| "ETA signal pending.".into())}</small>
                                    <small style="color:#64748b;">{format!("Visibility: {}", value.visibility_scope_label)}</small>
                                </div>
                            </section>
                        }.into_any()
                    } else {
                        view! { <section style="padding:1rem;background:white;border:1px solid #e5e7eb;border-radius:0.75rem;">"Tracking is not available."</section> }.into_any()
                    }
                }}
            </section>
        </article>
    }
}

#[component]
fn TrackingCard(label: &'static str, value: String) -> impl IntoView {
    view! {
        <div style="padding:1rem;background:white;border:1px solid #e5e7eb;border-radius:0.75rem;display:grid;gap:0.3rem;">
            <small style="color:#64748b;">{label}</small>
            <strong>{value}</strong>
        </div>
    }
}
