use leptos::{ev, prelude::*, tachys::view::any_view::IntoAny, task::spawn_local};
use leptos_router::components::A;

use crate::{
    api,
    session::{self, use_auth},
};
use shared::{CreateLoadLegRequest, CreateLoadRequest, LoadBuilderOption, LoadBuilderScreen};

#[derive(Debug, Clone, PartialEq)]
struct LoadLegDraft {
    pickup_location_id: String,
    delivery_location_id: String,
    pickup_date: String,
    delivery_date: String,
    bid_status: String,
    price: String,
}

fn first_option_id(options: &[LoadBuilderOption]) -> String {
    options
        .first()
        .map(|option| option.id.to_string())
        .unwrap_or_default()
}

fn second_option_id(options: &[LoadBuilderOption]) -> String {
    if options.len() > 1 {
        options[1].id.to_string()
    } else {
        first_option_id(options)
    }
}

fn default_bid_status(options: &[String]) -> String {
    options.first().cloned().unwrap_or_else(|| "Fixed".into())
}

fn default_weight_unit(options: &[String]) -> String {
    options.first().cloned().unwrap_or_else(|| "LBS".into())
}

fn default_leg(screen: &LoadBuilderScreen) -> LoadLegDraft {
    LoadLegDraft {
        pickup_location_id: first_option_id(&screen.location_options),
        delivery_location_id: second_option_id(&screen.location_options),
        pickup_date: String::new(),
        delivery_date: String::new(),
        bid_status: default_bid_status(&screen.bid_status_options),
        price: String::new(),
    }
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
            let pickup_location_id = parse_required_u64(
                &format!("pickup location for leg {}", index + 1),
                &leg.pickup_location_id,
            )?;
            let delivery_location_id = parse_required_u64(
                &format!("delivery location for leg {}", index + 1),
                &leg.delivery_location_id,
            )?;

            if pickup_location_id == delivery_location_id {
                return Err(format!(
                    "Leg {} must use different pickup and delivery locations.",
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
                pickup_location_id,
                delivery_location_id,
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
    title.set(String::new());
    load_type_id.set(first_option_id(&screen.load_type_options));
    equipment_id.set(first_option_id(&screen.equipment_options));
    commodity_type_id.set(first_option_id(&screen.commodity_type_options));
    weight_unit.set(default_weight_unit(&screen.weight_units));
    weight.set(String::new());
    special_instructions.set(String::new());
    is_hazardous.set(false);
    is_temperature_controlled.set(false);
    legs.set(vec![default_leg(screen)]);
}

#[component]
pub fn LoadBuilderPage() -> impl IntoView {
    let auth = use_auth();
    let screen = RwSignal::new(None::<LoadBuilderScreen>);
    let is_loading = RwSignal::new(false);
    let is_submitting = RwSignal::new(false);
    let error_message = RwSignal::new(None::<String>);
    let action_message = RwSignal::new(None::<String>);

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

    let can_manage_loads = Signal::derive(move || session::has_permission(&auth, "manage_loads"));

    Effect::new(move |_| {
        let ready = auth.session_ready.get();
        let current_session = auth.session.get();

        if !ready {
            return;
        }

        if !current_session.authenticated {
            screen.set(None);
            error_message.set(Some(
                "Sign in before creating loads from the Rust builder.".into(),
            ));
            is_loading.set(false);
            return;
        }

        if !can_manage_loads.get() {
            screen.set(None);
            error_message.set(Some(
                "The authenticated session does not have load creation access in this Rust slice."
                    .into(),
            ));
            is_loading.set(false);
            return;
        }

        is_loading.set(true);
        let auth = auth.clone();

        spawn_local(async move {
            let should_initialize = screen.get_untracked().is_none();

            match api::fetch_load_builder_screen().await {
                Ok(next_screen) => {
                    error_message.set(None);
                    if should_initialize {
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
                    }
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

    let submit = move |ev: ev::SubmitEvent| {
        ev.prevent_default();

        if !can_manage_loads.get() {
            action_message.set(Some(
                "The authenticated session does not have load creation access in this Rust slice."
                    .into(),
            ));
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

        spawn_local(async move {
            match api::create_load(&payload).await {
                Ok(response) => {
                    action_message.set(Some(response.message.clone()));
                    if response.success {
                        if let Some(screen_value) = screen.get_untracked() {
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
    };

    view! {
        <article style="display:grid;gap:1.25rem;max-width:1100px;">
            <section style="display:flex;justify-content:space-between;gap:1rem;align-items:flex-start;flex-wrap:wrap;">
                <div style="display:grid;gap:0.35rem;">
                    <h2>{move || screen.get().map(|value| value.title).unwrap_or_else(|| "Create Load".into())}</h2>
                    <p>{move || screen.get().map(|value| value.subtitle).unwrap_or_else(|| "First-pass Rust builder for core load creation.".into())}</p>
                </div>
                <div style="display:flex;gap:0.75rem;flex-wrap:wrap;align-items:center;">
                    <A href="/loads" attr:style="padding:0.7rem 1rem;border-radius:0.9rem;background:#f4f4f5;color:#111827;text-decoration:none;">"Back to load board"</A>
                    <span style="padding:0.35rem 0.65rem;border-radius:999px;background:#eff6ff;color:#1d4ed8;">
                        {move || if is_submitting.get() { "Saving..." } else { "Core builder active" }}
                    </span>
                </div>
            </section>

            {move || auth.session.get().user.map(|user| view! {
                <section style="padding:0.85rem 1rem;border:1px solid #dcfce7;border-radius:0.9rem;background:#f0fdf4;color:#166534;">
                    {format!("Authenticated as {} ({})", user.name, user.role_label)}
                </section>
            })}

            {move || action_message.get().map(|message| view! {
                <section style="padding:0.85rem 1rem;border:1px solid #dbeafe;border-radius:0.9rem;background:#eff6ff;color:#1d4ed8;">
                    {message}
                </section>
            })}

            {move || error_message.get().map(|message| view! {
                <section style="padding:0.85rem 1rem;border:1px solid #fecaca;border-radius:0.9rem;background:#fff1f2;color:#be123c;">
                    {message}
                </section>
            })}

            {move || {
                if is_loading.get() && screen.get().is_none() {
                    view! {
                        <section style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#fafaf9;">
                            "Loading Rust load-builder master data..."
                        </section>
                    }
                        .into_any()
                } else if screen.get().is_none() {
                    view! {
                        <section style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#fafaf9;">
                            "The load builder is waiting for a valid authenticated Rust session."
                        </section>
                    }
                        .into_any()
                } else {
                    view! {
                        <form on:submit=submit style="display:grid;gap:1.25rem;">
                            <section style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:1rem;align-items:start;">
                                <label style="display:grid;gap:0.35rem;grid-column:span 2;">
                                    <span>"Load title"</span>
                                    <input
                                        type="text"
                                        prop:value=move || title.get()
                                        on:input=move |ev| title.set(event_target_value(&ev))
                                        placeholder="Dallas to Joliet produce reload"
                                        style="padding:0.8rem 1rem;border:1px solid #d1d5db;border-radius:0.85rem;"
                                    />
                                </label>

                                <label style="display:grid;gap:0.35rem;">
                                    <span>"Load type"</span>
                                    <select
                                        prop:value=move || load_type_id.get()
                                        on:change=move |ev| load_type_id.set(event_target_value(&ev))
                                        style="padding:0.8rem 1rem;border:1px solid #d1d5db;border-radius:0.85rem;background:white;"
                                    >
                                        <option value="">"Select load type"</option>
                                        {move || screen.get().map(|value| {
                                            value.load_type_options.into_iter().map(|option| view! {
                                                <option value={option.id.to_string()}>{option.label}</option>
                                            }).collect_view()
                                        })}
                                    </select>
                                </label>

                                <label style="display:grid;gap:0.35rem;">
                                    <span>"Equipment"</span>
                                    <select
                                        prop:value=move || equipment_id.get()
                                        on:change=move |ev| equipment_id.set(event_target_value(&ev))
                                        style="padding:0.8rem 1rem;border:1px solid #d1d5db;border-radius:0.85rem;background:white;"
                                    >
                                        <option value="">"Select equipment"</option>
                                        {move || screen.get().map(|value| {
                                            value.equipment_options.into_iter().map(|option| view! {
                                                <option value={option.id.to_string()}>{option.label}</option>
                                            }).collect_view()
                                        })}
                                    </select>
                                </label>

                                <label style="display:grid;gap:0.35rem;">
                                    <span>"Commodity type"</span>
                                    <select
                                        prop:value=move || commodity_type_id.get()
                                        on:change=move |ev| commodity_type_id.set(event_target_value(&ev))
                                        style="padding:0.8rem 1rem;border:1px solid #d1d5db;border-radius:0.85rem;background:white;"
                                    >
                                        <option value="">"Select commodity type"</option>
                                        {move || screen.get().map(|value| {
                                            value.commodity_type_options.into_iter().map(|option| view! {
                                                <option value={option.id.to_string()}>{option.label}</option>
                                            }).collect_view()
                                        })}
                                    </select>
                                </label>

                                <label style="display:grid;gap:0.35rem;">
                                    <span>"Weight unit"</span>
                                    <select
                                        prop:value=move || weight_unit.get()
                                        on:change=move |ev| weight_unit.set(event_target_value(&ev))
                                        style="padding:0.8rem 1rem;border:1px solid #d1d5db;border-radius:0.85rem;background:white;"
                                    >
                                        {move || screen.get().map(|value| {
                                            value.weight_units.into_iter().map(|unit| view! {
                                                <option value={unit.clone()}>{unit.clone()}</option>
                                            }).collect_view()
                                        })}
                                    </select>
                                </label>

                                <label style="display:grid;gap:0.35rem;">
                                    <span>"Weight"</span>
                                    <input
                                        type="number"
                                        step="0.01"
                                        min="0"
                                        prop:value=move || weight.get()
                                        on:input=move |ev| weight.set(event_target_value(&ev))
                                        placeholder="42000"
                                        style="padding:0.8rem 1rem;border:1px solid #d1d5db;border-radius:0.85rem;"
                                    />
                                </label>
                            </section>

                            <section style="display:grid;gap:0.85rem;border:1px solid #e5e7eb;border-radius:1rem;background:#ffffff;padding:1rem;">
                                <strong>"Special handling"</strong>
                                <div style="display:flex;gap:1rem;flex-wrap:wrap;">
                                    <label style="display:flex;gap:0.5rem;align-items:center;">
                                        <input
                                            type="checkbox"
                                            prop:checked=move || is_hazardous.get()
                                            on:change=move |ev| is_hazardous.set(event_target_checked(&ev))
                                        />
                                        <span>"Hazardous"</span>
                                    </label>
                                    <label style="display:flex;gap:0.5rem;align-items:center;">
                                        <input
                                            type="checkbox"
                                            prop:checked=move || is_temperature_controlled.get()
                                            on:change=move |ev| is_temperature_controlled.set(event_target_checked(&ev))
                                        />
                                        <span>"Temperature controlled"</span>
                                    </label>
                                </div>
                                <label style="display:grid;gap:0.35rem;">
                                    <span>"Special instructions"</span>
                                    <textarea
                                        prop:value=move || special_instructions.get()
                                        on:input=move |ev| special_instructions.set(event_target_value(&ev))
                                        placeholder="Appointment notes, lumper details, securement reminders, or customer-specific instructions."
                                        style="min-height:120px;padding:0.9rem 1rem;border:1px solid #d1d5db;border-radius:0.85rem;resize:vertical;"
                                    ></textarea>
                                </label>
                            </section>

                            <section style="display:grid;gap:0.85rem;">
                                <div style="display:flex;justify-content:space-between;gap:1rem;align-items:center;flex-wrap:wrap;">
                                    <div>
                                        <strong>"Load legs"</strong>
                                        <p style="margin:0.25rem 0 0;">"This first Rust pass supports structured multi-leg creation, but document rows and address autocomplete are still next."</p>
                                    </div>
                                    <button
                                        type="button"
                                        on:click=add_leg
                                        style="padding:0.65rem 0.9rem;border:none;border-radius:0.85rem;background:#111827;color:white;cursor:pointer;"
                                    >
                                        "Add leg"
                                    </button>
                                </div>

                                {move || {
                                    let current_legs = legs.get();
                                    let removable = current_legs.len() > 1;
                                    current_legs
                                        .into_iter()
                                        .enumerate()
                                        .map(|(index, leg)| {
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
                                                        <label style="display:grid;gap:0.35rem;">
                                                            <span>"Pickup location"</span>
                                                            <select
                                                                prop:value=leg.pickup_location_id.clone()
                                                                on:change=move |ev| {
                                                                    let value = event_target_value(&ev);
                                                                    legs.update(|items| {
                                                                        if let Some(item) = items.get_mut(index) {
                                                                            item.pickup_location_id = value.clone();
                                                                        }
                                                                    });
                                                                }
                                                                style="padding:0.8rem 1rem;border:1px solid #d1d5db;border-radius:0.85rem;background:white;"
                                                            >
                                                                <option value="">"Select pickup"</option>
                                                                {move || screen.get().map(|value| {
                                                                    value.location_options.into_iter().map(|option| view! {
                                                                        <option value={option.id.to_string()}>{option.label}</option>
                                                                    }).collect_view()
                                                                })}
                                                            </select>
                                                        </label>

                                                        <label style="display:grid;gap:0.35rem;">
                                                            <span>"Delivery location"</span>
                                                            <select
                                                                prop:value=leg.delivery_location_id.clone()
                                                                on:change=move |ev| {
                                                                    let value = event_target_value(&ev);
                                                                    legs.update(|items| {
                                                                        if let Some(item) = items.get_mut(index) {
                                                                            item.delivery_location_id = value.clone();
                                                                        }
                                                                    });
                                                                }
                                                                style="padding:0.8rem 1rem;border:1px solid #d1d5db;border-radius:0.85rem;background:white;"
                                                            >
                                                                <option value="">"Select delivery"</option>
                                                                {move || screen.get().map(|value| {
                                                                    value.location_options.into_iter().map(|option| view! {
                                                                        <option value={option.id.to_string()}>{option.label}</option>
                                                                    }).collect_view()
                                                                })}
                                                            </select>
                                                        </label>

                                                        <label style="display:grid;gap:0.35rem;">
                                                            <span>"Pickup date"</span>
                                                            <input
                                                                type="date"
                                                                prop:value=leg.pickup_date.clone()
                                                                on:input=move |ev| {
                                                                    let value = event_target_value(&ev);
                                                                    legs.update(|items| {
                                                                        if let Some(item) = items.get_mut(index) {
                                                                            item.pickup_date = value.clone();
                                                                        }
                                                                    });
                                                                }
                                                                style="padding:0.8rem 1rem;border:1px solid #d1d5db;border-radius:0.85rem;"
                                                            />
                                                        </label>

                                                        <label style="display:grid;gap:0.35rem;">
                                                            <span>"Delivery date"</span>
                                                            <input
                                                                type="date"
                                                                prop:value=leg.delivery_date.clone()
                                                                on:input=move |ev| {
                                                                    let value = event_target_value(&ev);
                                                                    legs.update(|items| {
                                                                        if let Some(item) = items.get_mut(index) {
                                                                            item.delivery_date = value.clone();
                                                                        }
                                                                    });
                                                                }
                                                                style="padding:0.8rem 1rem;border:1px solid #d1d5db;border-radius:0.85rem;"
                                                            />
                                                        </label>

                                                        <label style="display:grid;gap:0.35rem;">
                                                            <span>"Bid status"</span>
                                                            <select
                                                                prop:value=leg.bid_status.clone()
                                                                on:change=move |ev| {
                                                                    let value = event_target_value(&ev);
                                                                    legs.update(|items| {
                                                                        if let Some(item) = items.get_mut(index) {
                                                                            item.bid_status = value.clone();
                                                                        }
                                                                    });
                                                                }
                                                                style="padding:0.8rem 1rem;border:1px solid #d1d5db;border-radius:0.85rem;background:white;"
                                                            >
                                                                {move || screen.get().map(|value| {
                                                                    value.bid_status_options.into_iter().map(|status| view! {
                                                                        <option value={status.clone()}>{status.clone()}</option>
                                                                    }).collect_view()
                                                                })}
                                                            </select>
                                                        </label>

                                                        <label style="display:grid;gap:0.35rem;">
                                                            <span>"Price"</span>
                                                            <input
                                                                type="number"
                                                                step="0.01"
                                                                min="0"
                                                                prop:value=leg.price.clone()
                                                                on:input=move |ev| {
                                                                    let value = event_target_value(&ev);
                                                                    legs.update(|items| {
                                                                        if let Some(item) = items.get_mut(index) {
                                                                            item.price = value.clone();
                                                                        }
                                                                    });
                                                                }
                                                                placeholder="2450"
                                                                style="padding:0.8rem 1rem;border:1px solid #d1d5db;border-radius:0.85rem;"
                                                            />
                                                        </label>
                                                    </div>
                                                </section>
                                            }
                                        })
                                        .collect_view()
                                }}
                            </section>

                            <section style="display:flex;justify-content:flex-end;gap:0.75rem;flex-wrap:wrap;">
                                <A href="/loads" attr:style="padding:0.75rem 1rem;border-radius:0.85rem;background:#f4f4f5;color:#111827;text-decoration:none;">"Cancel"</A>
                                <button
                                    type="submit"
                                    disabled=move || is_submitting.get() || !can_manage_loads.get()
                                    style="padding:0.75rem 1rem;border:none;border-radius:0.85rem;background:#111827;color:white;cursor:pointer;"
                                >
                                    {move || if is_submitting.get() { "Saving load..." } else { "Create load" }}
                                </button>
                            </section>
                        </form>
                    }
                        .into_any()
                }
            }}

            <section style="display:grid;gap:0.4rem;">
                {move || screen.get().map(|value| {
                    value.notes.into_iter().map(|note| view! { <p style="margin:0;">{note}</p> }).collect_view()
                })}
            </section>
        </article>
    }
}
