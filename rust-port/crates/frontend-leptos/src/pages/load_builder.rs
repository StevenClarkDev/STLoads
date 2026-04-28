use leptos::{ev, prelude::*, tachys::view::any_view::IntoAny, task::spawn_local};
use leptos_router::{
    components::A,
    hooks::{use_navigate, use_params_map},
};

use crate::{
    api, google_places, runtime_config,
    session::{self, use_auth},
};
use shared::{CreateLoadLegRequest, CreateLoadRequest, LoadBuilderScreen};

#[derive(Debug, Clone, PartialEq)]
struct LoadLegDraft {
    pickup_location_address: String,
    pickup_city: String,
    pickup_country: String,
    pickup_place_id: String,
    pickup_latitude: String,
    pickup_longitude: String,
    delivery_location_address: String,
    delivery_city: String,
    delivery_country: String,
    delivery_place_id: String,
    delivery_latitude: String,
    delivery_longitude: String,
    pickup_date: String,
    delivery_date: String,
    bid_status: String,
    price: String,
}

fn default_bid_status(options: &[String]) -> String {
    options.first().cloned().unwrap_or_else(|| "Fixed".into())
}

fn default_weight_unit(options: &[String]) -> String {
    options.first().cloned().unwrap_or_else(|| "LBS".into())
}

fn default_leg(screen: &LoadBuilderScreen) -> LoadLegDraft {
    LoadLegDraft {
        pickup_location_address: String::new(),
        pickup_city: String::new(),
        pickup_country: String::new(),
        pickup_place_id: String::new(),
        pickup_latitude: String::new(),
        pickup_longitude: String::new(),
        delivery_location_address: String::new(),
        delivery_city: String::new(),
        delivery_country: String::new(),
        delivery_place_id: String::new(),
        delivery_latitude: String::new(),
        delivery_longitude: String::new(),
        pickup_date: String::new(),
        delivery_date: String::new(),
        bid_status: default_bid_status(&screen.bid_status_options),
        price: String::new(),
    }
}

fn first_option_id<T>(items: &[T], id: impl Fn(&T) -> u64) -> String {
    items
        .first()
        .map(|item| id(item).to_string())
        .unwrap_or_default()
}

fn parse_required_u64(label: &str, value: &str) -> Result<u64, String> {
    value
        .trim()
        .parse::<u64>()
        .map_err(|_| format!("Select a valid {} before saving the load.", label))
}

fn parse_positive_f64(label: &str, value: &str) -> Result<f64, String> {
    let parsed = value
        .trim()
        .parse::<f64>()
        .map_err(|_| format!("Enter a valid {}.", label))?;
    if parsed > 0.0 {
        Ok(parsed)
    } else {
        Err(format!("{} must be greater than zero.", label))
    }
}

fn parse_non_negative_f64(label: &str, value: &str) -> Result<f64, String> {
    let parsed = value
        .trim()
        .parse::<f64>()
        .map_err(|_| format!("Enter a valid {}.", label))?;
    if parsed >= 0.0 {
        Ok(parsed)
    } else {
        Err(format!("{} cannot be negative.", label))
    }
}

fn parse_optional_f64(value: &str) -> Result<Option<f64>, String> {
    if value.trim().is_empty() {
        Ok(None)
    } else {
        value
            .trim()
            .parse::<f64>()
            .map(Some)
            .map_err(|_| "Google location coordinates could not be parsed.".into())
    }
}

fn build_create_load_request(
    title: &str,
    load_type_id: &str,
    equipment_id: &str,
    commodity_type_id: &str,
    weight_unit: &str,
    weight: &str,
    special_instructions: &str,
    is_hazardous: bool,
    is_temperature_controlled: bool,
    legs: &[LoadLegDraft],
) -> Result<CreateLoadRequest, String> {
    let title = title.trim().to_string();
    if title.is_empty() {
        return Err("Enter a load title before saving.".into());
    }

    if legs.is_empty() {
        return Err("Add at least one leg before saving the load.".into());
    }

    let legs = legs
        .iter()
        .enumerate()
        .map(|(index, leg)| {
            let pickup_location_address = leg.pickup_location_address.trim().to_string();
            if pickup_location_address.is_empty() {
                return Err(format!(
                    "Use Google autocomplete or enter a pickup address for leg {}.",
                    index + 1
                ));
            }

            let delivery_location_address = leg.delivery_location_address.trim().to_string();
            if delivery_location_address.is_empty() {
                return Err(format!(
                    "Use Google autocomplete or enter a delivery address for leg {}.",
                    index + 1
                ));
            }

            if pickup_location_address.eq_ignore_ascii_case(&delivery_location_address) {
                return Err(format!(
                    "Leg {} must use different pickup and delivery addresses.",
                    index + 1
                ));
            }

            let pickup_date = leg.pickup_date.trim().to_string();
            if pickup_date.is_empty() {
                return Err(format!("Enter a pickup date for leg {}.", index + 1));
            }

            let delivery_date = leg.delivery_date.trim().to_string();
            if delivery_date.is_empty() {
                return Err(format!("Enter a delivery date for leg {}.", index + 1));
            }

            let bid_status = leg.bid_status.trim().to_string();
            if !matches!(bid_status.as_str(), "Fixed" | "Open") {
                return Err(format!(
                    "Leg {} must use Fixed or Open bid status.",
                    index + 1
                ));
            }

            Ok(CreateLoadLegRequest {
                pickup_location_id: None,
                pickup_location_address: Some(pickup_location_address),
                pickup_city: (!leg.pickup_city.trim().is_empty())
                    .then_some(leg.pickup_city.trim().to_string()),
                pickup_country: (!leg.pickup_country.trim().is_empty())
                    .then_some(leg.pickup_country.trim().to_string()),
                pickup_place_id: (!leg.pickup_place_id.trim().is_empty())
                    .then_some(leg.pickup_place_id.trim().to_string()),
                pickup_latitude: parse_optional_f64(&leg.pickup_latitude)?,
                pickup_longitude: parse_optional_f64(&leg.pickup_longitude)?,
                delivery_location_id: None,
                delivery_location_address: Some(delivery_location_address),
                delivery_city: (!leg.delivery_city.trim().is_empty())
                    .then_some(leg.delivery_city.trim().to_string()),
                delivery_country: (!leg.delivery_country.trim().is_empty())
                    .then_some(leg.delivery_country.trim().to_string()),
                delivery_place_id: (!leg.delivery_place_id.trim().is_empty())
                    .then_some(leg.delivery_place_id.trim().to_string()),
                delivery_latitude: parse_optional_f64(&leg.delivery_latitude)?,
                delivery_longitude: parse_optional_f64(&leg.delivery_longitude)?,
                pickup_date,
                delivery_date,
                bid_status,
                price: parse_non_negative_f64(&format!("price for leg {}", index + 1), &leg.price)?,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(CreateLoadRequest {
        title,
        load_type_id: parse_required_u64("load type", load_type_id)?,
        equipment_id: parse_required_u64("equipment", equipment_id)?,
        commodity_type_id: parse_required_u64("commodity type", commodity_type_id)?,
        weight_unit: weight_unit.trim().to_string(),
        weight: parse_positive_f64("weight", weight)?,
        special_instructions: if special_instructions.trim().is_empty() {
            None
        } else {
            Some(special_instructions.trim().to_string())
        },
        is_hazardous,
        is_temperature_controlled,
        legs,
    })
}

fn initialize_form(
    screen: &LoadBuilderScreen,
    title: RwSignal<String>,
    load_type_id: RwSignal<String>,
    equipment_id: RwSignal<String>,
    commodity_type_id: RwSignal<String>,
    weight_unit: RwSignal<String>,
    weight: RwSignal<String>,
    special_instructions: RwSignal<String>,
    is_hazardous: RwSignal<bool>,
    is_temperature_controlled: RwSignal<bool>,
    legs: RwSignal<Vec<LoadLegDraft>>,
) {
    if let Some(draft) = screen.draft.as_ref() {
        title.set(draft.title.clone());
        load_type_id.set(
            draft
                .load_type_id
                .map(|value| value.to_string())
                .unwrap_or_else(|| first_option_id(&screen.load_type_options, |option| option.id)),
        );
        equipment_id.set(
            draft
                .equipment_id
                .map(|value| value.to_string())
                .unwrap_or_else(|| first_option_id(&screen.equipment_options, |option| option.id)),
        );
        commodity_type_id.set(
            draft
                .commodity_type_id
                .map(|value| value.to_string())
                .unwrap_or_else(|| {
                    first_option_id(&screen.commodity_type_options, |option| option.id)
                }),
        );
        weight_unit.set(
            draft
                .weight_unit
                .clone()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| default_weight_unit(&screen.weight_units)),
        );
        weight.set(
            draft
                .weight
                .map(|value| format!("{:.2}", value))
                .unwrap_or_default(),
        );
        special_instructions.set(draft.special_instructions.clone().unwrap_or_default());
        is_hazardous.set(draft.is_hazardous);
        is_temperature_controlled.set(draft.is_temperature_controlled);
        legs.set(if draft.legs.is_empty() {
            vec![default_leg(screen)]
        } else {
            draft
                .legs
                .iter()
                .map(|leg| LoadLegDraft {
                    pickup_location_address: leg.pickup_location_address.clone(),
                    pickup_city: leg.pickup_city.clone().unwrap_or_default(),
                    pickup_country: leg.pickup_country.clone().unwrap_or_default(),
                    pickup_place_id: leg.pickup_place_id.clone().unwrap_or_default(),
                    pickup_latitude: leg
                        .pickup_latitude
                        .map(|value| value.to_string())
                        .unwrap_or_default(),
                    pickup_longitude: leg
                        .pickup_longitude
                        .map(|value| value.to_string())
                        .unwrap_or_default(),
                    delivery_location_address: leg.delivery_location_address.clone(),
                    delivery_city: leg.delivery_city.clone().unwrap_or_default(),
                    delivery_country: leg.delivery_country.clone().unwrap_or_default(),
                    delivery_place_id: leg.delivery_place_id.clone().unwrap_or_default(),
                    delivery_latitude: leg
                        .delivery_latitude
                        .map(|value| value.to_string())
                        .unwrap_or_default(),
                    delivery_longitude: leg
                        .delivery_longitude
                        .map(|value| value.to_string())
                        .unwrap_or_default(),
                    pickup_date: leg.pickup_date.clone(),
                    delivery_date: leg.delivery_date.clone(),
                    bid_status: if leg.bid_status.trim().is_empty() {
                        default_bid_status(&screen.bid_status_options)
                    } else {
                        leg.bid_status.clone()
                    },
                    price: leg
                        .price
                        .map(|value| format!("{:.2}", value))
                        .unwrap_or_default(),
                })
                .collect()
        });
        return;
    }

    title.set(String::new());
    load_type_id.set(first_option_id(&screen.load_type_options, |option| {
        option.id
    }));
    equipment_id.set(first_option_id(&screen.equipment_options, |option| {
        option.id
    }));
    commodity_type_id.set(first_option_id(&screen.commodity_type_options, |option| {
        option.id
    }));
    weight_unit.set(default_weight_unit(&screen.weight_units));
    weight.set(String::new());
    special_instructions.set(String::new());
    is_hazardous.set(false);
    is_temperature_controlled.set(false);
    legs.set(vec![default_leg(screen)]);
}

fn google_maps_api_key() -> Option<String> {
    runtime_config::google_maps_api_key().or_else(|| {
        option_env!("GOOGLE_MAPS_API_KEY")
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    })
}

fn pickup_address_input_id(index: usize) -> String {
    format!("load-leg-{}-pickup-address", index)
}
fn pickup_city_input_id(index: usize) -> String {
    format!("load-leg-{}-pickup-city", index)
}
fn pickup_country_input_id(index: usize) -> String {
    format!("load-leg-{}-pickup-country", index)
}
fn pickup_place_id_input_id(index: usize) -> String {
    format!("load-leg-{}-pickup-place-id", index)
}
fn pickup_latitude_input_id(index: usize) -> String {
    format!("load-leg-{}-pickup-latitude", index)
}
fn pickup_longitude_input_id(index: usize) -> String {
    format!("load-leg-{}-pickup-longitude", index)
}
fn delivery_address_input_id(index: usize) -> String {
    format!("load-leg-{}-delivery-address", index)
}
fn delivery_city_input_id(index: usize) -> String {
    format!("load-leg-{}-delivery-city", index)
}
fn delivery_country_input_id(index: usize) -> String {
    format!("load-leg-{}-delivery-country", index)
}
fn delivery_place_id_input_id(index: usize) -> String {
    format!("load-leg-{}-delivery-place-id", index)
}
fn delivery_latitude_input_id(index: usize) -> String {
    format!("load-leg-{}-delivery-latitude", index)
}
fn delivery_longitude_input_id(index: usize) -> String {
    format!("load-leg-{}-delivery-longitude", index)
}

#[component]
pub fn LoadBuilderPage() -> impl IntoView {
    let navigate = use_navigate();
    let params = use_params_map();
    let auth = use_auth();
    let screen = RwSignal::new(None::<LoadBuilderScreen>);
    let is_loading = RwSignal::new(false);
    let is_submitting = RwSignal::new(false);
    let error_message = RwSignal::new(None::<String>);
    let action_message = RwSignal::new(None::<String>);
    let google_status = RwSignal::new(None::<String>);
    let google_ready = RwSignal::new(false);

    let title = RwSignal::new(String::new());
    let load_type_id = RwSignal::new(String::new());
    let equipment_id = RwSignal::new(String::new());
    let commodity_type_id = RwSignal::new(String::new());
    let weight_unit = RwSignal::new("LBS".to_string());
    let weight = RwSignal::new(String::new());
    let special_instructions = RwSignal::new(String::new());
    let is_hazardous = RwSignal::new(false);
    let is_temperature_controlled = RwSignal::new(false);
    let legs = RwSignal::new(Vec::<LoadLegDraft>::new());

    let editing_load_id = Memo::new(move |_| {
        params.with(|map| {
            map.get("load_id")
                .and_then(|value| value.parse::<u64>().ok())
        })
    });

    let can_manage_loads = Signal::derive(move || session::has_permission(&auth, "manage_loads"));

    Effect::new(move |_| {
        let ready = auth.session_ready.get();
        let current_session = auth.session.get();
        let editing_load_id = editing_load_id.get();

        if !ready {
            return;
        }

        if !current_session.authenticated {
            screen.set(None);
            error_message.set(Some(if editing_load_id.is_some() {
                "Sign in before editing loads from the Rust builder.".into()
            } else {
                "Sign in before creating loads from the Rust builder.".into()
            }));
            is_loading.set(false);
            return;
        }

        if !can_manage_loads.get() {
            screen.set(None);
            error_message.set(Some(if editing_load_id.is_some() {
                "The authenticated session does not have load update access in this Rust slice."
                    .into()
            } else {
                "The authenticated session does not have load creation access in this Rust slice."
                    .into()
            }));
            is_loading.set(false);
            return;
        }

        is_loading.set(true);
        let auth = auth.clone();

        spawn_local(async move {
            match api::fetch_load_builder_screen(editing_load_id).await {
                Ok(next_screen) => {
                    error_message.set(None);
                    initialize_form(
                        &next_screen,
                        title,
                        load_type_id,
                        equipment_id,
                        commodity_type_id,
                        weight_unit,
                        weight,
                        special_instructions,
                        is_hazardous,
                        is_temperature_controlled,
                        legs,
                    );
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
        let Some(_screen) = screen.get() else {
            return;
        };

        let leg_count = legs.with(|items| items.len());
        if leg_count == 0 {
            return;
        }

        let Some(api_key) = google_maps_api_key() else {
            google_ready.set(false);
            google_status.set(Some("Google address autocomplete is waiting for GOOGLE_MAPS_API_KEY in frontend runtime config. Manual typing still works, but IBM deployment should provide the browser key with domain restrictions.".into()));
            return;
        };

        spawn_local(async move {
            match google_places::ensure_loaded(&api_key).await {
                Ok(()) => {
                    google_ready.set(true);
                    google_status.set(Some("Google address autocomplete is active. If device GPS is available, predictions are biased toward the current area; if GPS is off, Google returns general suggestions instead.".into()));

                    for index in 0..leg_count {
                        let _ = google_places::attach_address_autocomplete(
                            &pickup_address_input_id(index),
                            &pickup_city_input_id(index),
                            &pickup_country_input_id(index),
                            &pickup_place_id_input_id(index),
                            &pickup_latitude_input_id(index),
                            &pickup_longitude_input_id(index),
                        )
                        .await;

                        let _ = google_places::attach_address_autocomplete(
                            &delivery_address_input_id(index),
                            &delivery_city_input_id(index),
                            &delivery_country_input_id(index),
                            &delivery_place_id_input_id(index),
                            &delivery_latitude_input_id(index),
                            &delivery_longitude_input_id(index),
                        )
                        .await;
                    }
                }
                Err(error) => {
                    google_ready.set(false);
                    google_status.set(Some(error));
                }
            }
        });
    });

    let add_leg = move |_| {
        let Some(screen_value) = screen.get() else {
            action_message.set(Some(
                "The load builder master data has not loaded yet, so a new leg cannot be added."
                    .into(),
            ));
            return;
        };

        legs.update(|items| items.push(default_leg(&screen_value)));
        action_message.set(None);
    };

    let submit = Callback::new(move |ev: ev::SubmitEvent| {
        ev.prevent_default();

        let editing_load_id = editing_load_id.get();

        if !can_manage_loads.get() {
            action_message.set(Some(if editing_load_id.is_some() {
                "The authenticated session does not have load update access in this Rust slice."
                    .into()
            } else {
                "The authenticated session does not have load creation access in this Rust slice."
                    .into()
            }));
            return;
        }

        let payload = match build_create_load_request(
            &title.get(),
            &load_type_id.get(),
            &equipment_id.get(),
            &commodity_type_id.get(),
            &weight_unit.get(),
            &weight.get(),
            &special_instructions.get(),
            is_hazardous.get(),
            is_temperature_controlled.get(),
            &legs.get(),
        ) {
            Ok(payload) => payload,
            Err(message) => {
                action_message.set(Some(message));
                return;
            }
        };

        is_submitting.set(true);
        action_message.set(None);
        let auth = auth.clone();
        let navigate = navigate.clone();

        spawn_local(async move {
            let result = match editing_load_id {
                Some(load_id) => api::update_load(load_id, &payload).await,
                None => api::create_load(&payload).await,
            };

            match result {
                Ok(response) => {
                    action_message.set(Some(response.message.clone()));
                    if response.success {
                        if let Some(load_id) = response.load_id {
                            navigate(&format!("/loads/{}", load_id), Default::default());
                        } else if let Some(screen_value) = screen.get_untracked() {
                            initialize_form(
                                &screen_value,
                                title,
                                load_type_id,
                                equipment_id,
                                commodity_type_id,
                                weight_unit,
                                weight,
                                special_instructions,
                                is_hazardous,
                                is_temperature_controlled,
                                legs,
                            );
                        }
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

            is_submitting.set(false);
        });
    });

    view! {
        <article style="display:grid;gap:1.25rem;max-width:1100px;">
            <section style="display:flex;justify-content:space-between;gap:1rem;align-items:flex-start;flex-wrap:wrap;">
                <div style="display:grid;gap:0.35rem;">
                    <h2>{move || screen.get().map(|value| value.title).unwrap_or_else(|| "Create Load".into())}</h2>
                    <p>{move || screen.get().map(|value| value.subtitle).unwrap_or_else(|| "Rust builder for Google-address load creation.".into())}</p>
                </div>
                <div style="display:flex;gap:0.75rem;flex-wrap:wrap;align-items:center;">
                    <A href="/loads" attr:style="padding:0.7rem 1rem;border-radius:0.9rem;background:#f4f4f5;color:#111827;text-decoration:none;">"Back to load board"</A>
                    <span style="padding:0.35rem 0.65rem;border-radius:999px;background:#eff6ff;color:#1d4ed8;">
                        {move || if is_submitting.get() { "Saving..." } else if google_ready.get() { "Google address mode" } else { "Address setup pending" }}
                    </span>
                </div>
            </section>

            {move || error_message.get().map(|message| view! {
                <section style="padding:0.85rem 1rem;border:1px solid #fecaca;border-radius:0.9rem;background:#fff1f2;color:#be123c;">
                    {message}
                </section>
            })}

            {move || {
                if is_loading.get() && screen.get().is_none() {
                    view! {
                        <section style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#fafaf9;">"Loading Rust load-builder master data..."</section>
                    }.into_any()
                } else if screen.get().is_none() {
                    view! {
                        <section style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#fafaf9;">"The load builder is waiting for a valid authenticated Rust session."</section>
                    }.into_any()
                } else {
                    view! {
                        <form on:submit=move |ev| submit.run(ev) style="display:grid;gap:1.25rem;">
                            <section style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:1rem;align-items:start;">
                                <label style="display:grid;gap:0.35rem;grid-column:span 2;">
                                    <span>"Load title"</span>
                                    <input type="text" prop:value=move || title.get() on:input=move |ev| title.set(event_target_value(&ev)) placeholder="Dallas to Joliet produce reload" style="padding:0.8rem 1rem;border:1px solid #d1d5db;border-radius:0.85rem;" />
                                </label>

                                <label style="display:grid;gap:0.35rem;">
                                    <span>"Load type"</span>
                                    <select prop:value=move || load_type_id.get() on:change=move |ev| load_type_id.set(event_target_value(&ev)) style="padding:0.8rem 1rem;border:1px solid #d1d5db;border-radius:0.85rem;background:white;">
                                        <option value="">"Select load type"</option>
                                        {move || screen.get().map(|value| value.load_type_options.into_iter().map(|option| view! { <option value={option.id.to_string()}>{option.label}</option> }).collect_view())}
                                    </select>
                                </label>

                                <label style="display:grid;gap:0.35rem;">
                                    <span>"Equipment"</span>
                                    <select prop:value=move || equipment_id.get() on:change=move |ev| equipment_id.set(event_target_value(&ev)) style="padding:0.8rem 1rem;border:1px solid #d1d5db;border-radius:0.85rem;background:white;">
                                        <option value="">"Select equipment"</option>
                                        {move || screen.get().map(|value| value.equipment_options.into_iter().map(|option| view! { <option value={option.id.to_string()}>{option.label}</option> }).collect_view())}
                                    </select>
                                </label>

                                <label style="display:grid;gap:0.35rem;">
                                    <span>"Commodity type"</span>
                                    <select prop:value=move || commodity_type_id.get() on:change=move |ev| commodity_type_id.set(event_target_value(&ev)) style="padding:0.8rem 1rem;border:1px solid #d1d5db;border-radius:0.85rem;background:white;">
                                        <option value="">"Select commodity type"</option>
                                        {move || screen.get().map(|value| value.commodity_type_options.into_iter().map(|option| view! { <option value={option.id.to_string()}>{option.label}</option> }).collect_view())}
                                    </select>
                                </label>

                                <label style="display:grid;gap:0.35rem;">
                                    <span>"Weight unit"</span>
                                    <select prop:value=move || weight_unit.get() on:change=move |ev| weight_unit.set(event_target_value(&ev)) style="padding:0.8rem 1rem;border:1px solid #d1d5db;border-radius:0.85rem;background:white;">
                                        {move || screen.get().map(|value| value.weight_units.into_iter().map(|unit| view! { <option value={unit.clone()}>{unit.clone()}</option> }).collect_view())}
                                    </select>
                                </label>

                                <label style="display:grid;gap:0.35rem;">
                                    <span>"Weight"</span>
                                    <input type="number" step="0.01" min="0" prop:value=move || weight.get() on:input=move |ev| weight.set(event_target_value(&ev)) placeholder="42000" style="padding:0.8rem 1rem;border:1px solid #d1d5db;border-radius:0.85rem;" />
                                </label>
                            </section>

                            <section style="display:grid;gap:0.85rem;border:1px solid #e5e7eb;border-radius:1rem;background:#ffffff;padding:1rem;">
                                <strong>"Special handling"</strong>
                                <div style="display:flex;gap:1rem;flex-wrap:wrap;">
                                    <label style="display:flex;gap:0.5rem;align-items:center;">
                                        <input type="checkbox" prop:checked=move || is_hazardous.get() on:change=move |ev| is_hazardous.set(event_target_checked(&ev)) />
                                        <span>"Hazardous"</span>
                                    </label>
                                    <label style="display:flex;gap:0.5rem;align-items:center;">
                                        <input type="checkbox" prop:checked=move || is_temperature_controlled.get() on:change=move |ev| is_temperature_controlled.set(event_target_checked(&ev)) />
                                        <span>"Temperature controlled"</span>
                                    </label>
                                </div>
                                <label style="display:grid;gap:0.35rem;">
                                    <span>"Special instructions"</span>
                                    <textarea prop:value=move || special_instructions.get() on:input=move |ev| special_instructions.set(event_target_value(&ev)) placeholder="Appointment notes, lumper details, securement reminders, or customer-specific instructions." style="min-height:120px;padding:0.9rem 1rem;border:1px solid #d1d5db;border-radius:0.85rem;resize:vertical;"></textarea>
                                </label>
                            </section>

                            <section style="display:grid;gap:0.85rem;">
                                <div style="display:flex;justify-content:space-between;gap:1rem;align-items:center;flex-wrap:wrap;">
                                    <div>
                                        <strong>"Load legs"</strong>
                                    </div>
                                    <button type="button" on:click=add_leg style="padding:0.65rem 0.9rem;border:none;border-radius:0.85rem;background:#111827;color:white;cursor:pointer;">"Add leg"</button>
                                </div>

                                {move || {
                                    let current_legs = legs.get();
                                    let removable = current_legs.len() > 1;
                                    current_legs.into_iter().enumerate().map(|(index, leg)| {
                                        let pickup_address_id = pickup_address_input_id(index);
                                        let pickup_city_id = pickup_city_input_id(index);
                                        let pickup_country_id = pickup_country_input_id(index);
                                        let pickup_place_id_id = pickup_place_id_input_id(index);
                                        let pickup_latitude_id = pickup_latitude_input_id(index);
                                        let pickup_longitude_id = pickup_longitude_input_id(index);
                                        let delivery_address_id = delivery_address_input_id(index);
                                        let delivery_city_id = delivery_city_input_id(index);
                                        let delivery_country_id = delivery_country_input_id(index);
                                        let delivery_place_id_id = delivery_place_id_input_id(index);
                                        let delivery_latitude_id = delivery_latitude_input_id(index);
                                        let delivery_longitude_id = delivery_longitude_input_id(index);
                                        view! {
                                            <section style="display:grid;gap:0.85rem;border:1px solid #e5e7eb;border-radius:1rem;background:#ffffff;padding:1rem;">
                                                <div style="display:flex;justify-content:space-between;gap:1rem;align-items:center;flex-wrap:wrap;">
                                                    <strong>{format!("Leg {}", index + 1)}</strong>
                                                    {removable.then(|| view! {
                                                        <button
                                                            type="button"
                                                            on:click=move |_| {
                                                                legs.update(|items| {
                                                                    if items.len() > 1 && index < items.len() {
                                                                        items.remove(index);
                                                                    }
                                                                });
                                                            }
                                                            style="padding:0.45rem 0.75rem;border:1px solid #fecaca;border-radius:0.75rem;background:#fff1f2;color:#be123c;cursor:pointer;"
                                                        >
                                                            "Remove"
                                                        </button>
                                                    })}
                                                </div>

                                                <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:1rem;align-items:start;">
                                                    <label style="display:grid;gap:0.35rem;grid-column:span 2;">
                                                        <span>"Pickup address"</span>
                                                        <input id=pickup_address_id type="text" prop:value=leg.pickup_location_address.clone() on:input=move |ev| {
                                                            let value = event_target_value(&ev);
                                                            legs.update(|items| {
                                                                if let Some(item) = items.get_mut(index) {
                                                                    item.pickup_location_address = value.clone();
                                                                }
                                                            });
                                                        } placeholder="Search pickup address with Google" style="padding:0.8rem 1rem;border:1px solid #d1d5db;border-radius:0.85rem;" />
                                                    </label>

                                                    <label style="display:grid;gap:0.35rem;grid-column:span 2;">
                                                        <span>"Delivery address"</span>
                                                        <input id=delivery_address_id type="text" prop:value=leg.delivery_location_address.clone() on:input=move |ev| {
                                                            let value = event_target_value(&ev);
                                                            legs.update(|items| {
                                                                if let Some(item) = items.get_mut(index) {
                                                                    item.delivery_location_address = value.clone();
                                                                }
                                                            });
                                                        } placeholder="Search delivery address with Google" style="padding:0.8rem 1rem;border:1px solid #d1d5db;border-radius:0.85rem;" />
                                                    </label>

                                                    <input id=pickup_city_id type="hidden" prop:value=leg.pickup_city.clone() on:input=move |ev| {
                                                        let value = event_target_value(&ev);
                                                        legs.update(|items| {
                                                            if let Some(item) = items.get_mut(index) {
                                                                item.pickup_city = value.clone();
                                                            }
                                                        });
                                                    } />
                                                    <input id=pickup_country_id type="hidden" prop:value=leg.pickup_country.clone() on:input=move |ev| {
                                                        let value = event_target_value(&ev);
                                                        legs.update(|items| {
                                                            if let Some(item) = items.get_mut(index) {
                                                                item.pickup_country = value.clone();
                                                            }
                                                        });
                                                    } />
                                                    <input id=pickup_place_id_id type="hidden" prop:value=leg.pickup_place_id.clone() on:input=move |ev| {
                                                        let value = event_target_value(&ev);
                                                        legs.update(|items| {
                                                            if let Some(item) = items.get_mut(index) {
                                                                item.pickup_place_id = value.clone();
                                                            }
                                                        });
                                                    } />
                                                    <input id=pickup_latitude_id type="hidden" prop:value=leg.pickup_latitude.clone() on:input=move |ev| {
                                                        let value = event_target_value(&ev);
                                                        legs.update(|items| {
                                                            if let Some(item) = items.get_mut(index) {
                                                                item.pickup_latitude = value.clone();
                                                            }
                                                        });
                                                    } />
                                                    <input id=pickup_longitude_id type="hidden" prop:value=leg.pickup_longitude.clone() on:input=move |ev| {
                                                        let value = event_target_value(&ev);
                                                        legs.update(|items| {
                                                            if let Some(item) = items.get_mut(index) {
                                                                item.pickup_longitude = value.clone();
                                                            }
                                                        });
                                                    } />
                                                    <input id=delivery_city_id type="hidden" prop:value=leg.delivery_city.clone() on:input=move |ev| {
                                                        let value = event_target_value(&ev);
                                                        legs.update(|items| {
                                                            if let Some(item) = items.get_mut(index) {
                                                                item.delivery_city = value.clone();
                                                            }
                                                        });
                                                    } />
                                                    <input id=delivery_country_id type="hidden" prop:value=leg.delivery_country.clone() on:input=move |ev| {
                                                        let value = event_target_value(&ev);
                                                        legs.update(|items| {
                                                            if let Some(item) = items.get_mut(index) {
                                                                item.delivery_country = value.clone();
                                                            }
                                                        });
                                                    } />
                                                    <input id=delivery_place_id_id type="hidden" prop:value=leg.delivery_place_id.clone() on:input=move |ev| {
                                                        let value = event_target_value(&ev);
                                                        legs.update(|items| {
                                                            if let Some(item) = items.get_mut(index) {
                                                                item.delivery_place_id = value.clone();
                                                            }
                                                        });
                                                    } />
                                                    <input id=delivery_latitude_id type="hidden" prop:value=leg.delivery_latitude.clone() on:input=move |ev| {
                                                        let value = event_target_value(&ev);
                                                        legs.update(|items| {
                                                            if let Some(item) = items.get_mut(index) {
                                                                item.delivery_latitude = value.clone();
                                                            }
                                                        });
                                                    } />
                                                    <input id=delivery_longitude_id type="hidden" prop:value=leg.delivery_longitude.clone() on:input=move |ev| {
                                                        let value = event_target_value(&ev);
                                                        legs.update(|items| {
                                                            if let Some(item) = items.get_mut(index) {
                                                                item.delivery_longitude = value.clone();
                                                            }
                                                        });
                                                    } />

                                                    <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:1rem;grid-column:span 2;">
                                                        <div style="padding:0.75rem;border:1px solid #e5e7eb;border-radius:0.85rem;background:#fcfcfb;display:grid;gap:0.2rem;">
                                                            <small style="color:#64748b;">"Pickup city / country"</small>
                                                            <strong>{if leg.pickup_city.is_empty() && leg.pickup_country.is_empty() { "Waiting for Google selection".to_string() } else { format!("{} {}{}", leg.pickup_city, if leg.pickup_city.is_empty() || leg.pickup_country.is_empty() { "" } else { "-" }, leg.pickup_country) }}</strong>
                                                        </div>
                                                        <div style="padding:0.75rem;border:1px solid #e5e7eb;border-radius:0.85rem;background:#fcfcfb;display:grid;gap:0.2rem;">
                                                            <small style="color:#64748b;">"Delivery city / country"</small>
                                                            <strong>{if leg.delivery_city.is_empty() && leg.delivery_country.is_empty() { "Waiting for Google selection".to_string() } else { format!("{} {}{}", leg.delivery_city, if leg.delivery_city.is_empty() || leg.delivery_country.is_empty() { "" } else { "-" }, leg.delivery_country) }}</strong>
                                                        </div>
                                                    </div>

                                                    <label style="display:grid;gap:0.35rem;">
                                                        <span>"Pickup date"</span>
                                                        <input type="date" prop:value=leg.pickup_date.clone() on:input=move |ev| {
                                                            let value = event_target_value(&ev);
                                                            legs.update(|items| {
                                                                if let Some(item) = items.get_mut(index) {
                                                                    item.pickup_date = value.clone();
                                                                }
                                                            });
                                                        } style="padding:0.8rem 1rem;border:1px solid #d1d5db;border-radius:0.85rem;" />
                                                    </label>

                                                    <label style="display:grid;gap:0.35rem;">
                                                        <span>"Delivery date"</span>
                                                        <input type="date" prop:value=leg.delivery_date.clone() on:input=move |ev| {
                                                            let value = event_target_value(&ev);
                                                            legs.update(|items| {
                                                                if let Some(item) = items.get_mut(index) {
                                                                    item.delivery_date = value.clone();
                                                                }
                                                            });
                                                        } style="padding:0.8rem 1rem;border:1px solid #d1d5db;border-radius:0.85rem;" />
                                                    </label>

                                                    <label style="display:grid;gap:0.35rem;">
                                                        <span>"Bid status"</span>
                                                        <select prop:value=leg.bid_status.clone() on:change=move |ev| {
                                                            let value = event_target_value(&ev);
                                                            legs.update(|items| {
                                                                if let Some(item) = items.get_mut(index) {
                                                                    item.bid_status = value.clone();
                                                                }
                                                            });
                                                        } style="padding:0.8rem 1rem;border:1px solid #d1d5db;border-radius:0.85rem;background:white;">
                                                            {move || screen.get().map(|value| value.bid_status_options.into_iter().map(|status| view! { <option value={status.clone()}>{status.clone()}</option> }).collect_view())}
                                                        </select>
                                                    </label>

                                                    <label style="display:grid;gap:0.35rem;">
                                                        <span>"Price"</span>
                                                        <input type="number" step="0.01" min="0" prop:value=leg.price.clone() on:input=move |ev| {
                                                            let value = event_target_value(&ev);
                                                            legs.update(|items| {
                                                                if let Some(item) = items.get_mut(index) {
                                                                    item.price = value.clone();
                                                                }
                                                            });
                                                        } placeholder="2450" style="padding:0.8rem 1rem;border:1px solid #d1d5db;border-radius:0.85rem;" />
                                                    </label>
                                                </div>
                                            </section>
                                        }
                                    }).collect_view()
                                }}
                            </section>

                            <section style="display:flex;justify-content:flex-end;gap:0.75rem;flex-wrap:wrap;">
                                <A href="/loads" attr:style="padding:0.75rem 1rem;border-radius:0.85rem;background:#f4f4f5;color:#111827;text-decoration:none;">"Cancel"</A>
                                <button type="submit" disabled=move || is_submitting.get() || !can_manage_loads.get() style="padding:0.75rem 1rem;border:none;border-radius:0.85rem;background:#111827;color:white;cursor:pointer;">
                                    {move || if is_submitting.get() { "Saving load...".to_string() } else { screen.get().map(|value| value.submit_label).unwrap_or_else(|| "Save load".into()) }}
                                </button>
                            </section>
                        </form>
                    }.into_any()
                }
            }}

        </article>
    }
}
