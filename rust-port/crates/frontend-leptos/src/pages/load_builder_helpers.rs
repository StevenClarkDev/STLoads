use leptos::prelude::*;
use serde_json::Value;

use crate::runtime_config;
use shared::{CreateLoadLegRequest, CreateLoadRequest, LoadBuilderScreen};

pub(super) const FIELD_LABEL_STYLE: &str = "display:grid;gap:0.35rem;";
pub(super) const FIELD_INPUT_STYLE: &str =
    "padding:0.8rem 1rem;border:1px solid #d1d5db;border-radius:0.85rem;";
pub(super) const FIELD_SELECT_STYLE: &str =
    "padding:0.8rem 1rem;border:1px solid #d1d5db;border-radius:0.85rem;background:white;";
pub(super) const FIELD_TEXTAREA_STYLE: &str = "min-height:90px;padding:0.9rem 1rem;border:1px solid #d1d5db;border-radius:0.85rem;resize:vertical;";

#[derive(Debug, Clone, PartialEq)]
pub(super) struct LoadLegDraft {
    pub(super) pickup_location_address: String,
    pub(super) pickup_city: String,
    pub(super) pickup_country: String,
    pub(super) pickup_place_id: String,
    pub(super) pickup_latitude: String,
    pub(super) pickup_longitude: String,
    pub(super) delivery_location_address: String,
    pub(super) delivery_city: String,
    pub(super) delivery_country: String,
    pub(super) delivery_place_id: String,
    pub(super) delivery_latitude: String,
    pub(super) delivery_longitude: String,
    pub(super) pickup_date: String,
    pub(super) delivery_date: String,
    pub(super) bid_status: String,
    pub(super) price: String,
}

pub(super) fn default_bid_status(options: &[String]) -> String {
    options.first().cloned().unwrap_or_else(|| "Fixed".into())
}

pub(super) fn default_weight_unit(options: &[String]) -> String {
    options.first().cloned().unwrap_or_else(|| "LBS".into())
}

pub(super) fn default_leg(screen: &LoadBuilderScreen) -> LoadLegDraft {
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

pub(super) fn first_option_id<T>(items: &[T], id: impl Fn(&T) -> u64) -> String {
    items
        .first()
        .map(|item| id(item).to_string())
        .unwrap_or_default()
}

pub(super) fn parse_required_u64(label: &str, value: &str) -> Result<u64, String> {
    value
        .trim()
        .parse::<u64>()
        .map_err(|_| format!("Select a valid {} before saving the load.", label))
}

pub(super) fn parse_positive_f64(label: &str, value: &str) -> Result<f64, String> {
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

pub(super) fn parse_non_negative_f64(label: &str, value: &str) -> Result<f64, String> {
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

pub(super) fn parse_optional_f64(value: &str) -> Result<Option<f64>, String> {
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

pub(super) fn optional_trimmed(value: &str) -> Option<String> {
    (!value.trim().is_empty()).then_some(value.trim().to_string())
}

pub(super) fn parse_optional_jsonish(label: &str, value: &str) -> Result<Option<Value>, String> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }

    if value.starts_with('{') || value.starts_with('[') {
        return serde_json::from_str::<Value>(value)
            .map(Some)
            .map_err(|error| format!("{} must be valid JSON: {}", label, error));
    }

    Ok(Some(Value::String(value.to_string())))
}

pub(super) fn jsonish_to_form(value: Option<&Value>) -> String {
    match value {
        Some(Value::String(text)) => text.clone(),
        Some(value) => serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string()),
        None => String::new(),
    }
}

pub(super) fn build_create_load_request(
    title: &str,
    load_type_id: &str,
    equipment_id: &str,
    commodity_type_id: &str,
    freight_mode: &str,
    visibility: &str,
    service_level: &str,
    customer_reference: &str,
    po_number: &str,
    pickup_appointment_ref: &str,
    delivery_appointment_ref: &str,
    facility_contact_name: &str,
    facility_contact_phone: &str,
    facility_contact_email: &str,
    appointment_window_start: &str,
    appointment_window_end: &str,
    accessorial_flags: &str,
    weight_unit: &str,
    weight: &str,
    temperature_data: &str,
    container_data: &str,
    securement_data: &str,
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
        customer_contract_id: None,
        customer_contract_lane_id: None,
        freight_mode: optional_trimmed(freight_mode),
        visibility: optional_trimmed(visibility),
        service_level: optional_trimmed(service_level),
        customer_reference: optional_trimmed(customer_reference),
        po_number: optional_trimmed(po_number),
        pickup_appointment_ref: optional_trimmed(pickup_appointment_ref),
        delivery_appointment_ref: optional_trimmed(delivery_appointment_ref),
        facility_contact_name: optional_trimmed(facility_contact_name),
        facility_contact_phone: optional_trimmed(facility_contact_phone),
        facility_contact_email: optional_trimmed(facility_contact_email),
        appointment_window_start: optional_trimmed(appointment_window_start),
        appointment_window_end: optional_trimmed(appointment_window_end),
        accessorial_flags: parse_optional_jsonish("Accessorials", accessorial_flags)?,
        weight_unit: weight_unit.trim().to_string(),
        weight: parse_positive_f64("weight", weight)?,
        temperature_data: parse_optional_jsonish("Temperature details", temperature_data)?,
        container_data: parse_optional_jsonish("Container details", container_data)?,
        securement_data: parse_optional_jsonish("Securement details", securement_data)?,
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

pub(super) fn initialize_form(
    screen: &LoadBuilderScreen,
    title: RwSignal<String>,
    load_type_id: RwSignal<String>,
    equipment_id: RwSignal<String>,
    commodity_type_id: RwSignal<String>,
    freight_mode: RwSignal<String>,
    visibility: RwSignal<String>,
    service_level: RwSignal<String>,
    customer_reference: RwSignal<String>,
    po_number: RwSignal<String>,
    pickup_appointment_ref: RwSignal<String>,
    delivery_appointment_ref: RwSignal<String>,
    facility_contact_name: RwSignal<String>,
    facility_contact_phone: RwSignal<String>,
    facility_contact_email: RwSignal<String>,
    appointment_window_start: RwSignal<String>,
    appointment_window_end: RwSignal<String>,
    accessorial_flags: RwSignal<String>,
    weight_unit: RwSignal<String>,
    weight: RwSignal<String>,
    temperature_data: RwSignal<String>,
    container_data: RwSignal<String>,
    securement_data: RwSignal<String>,
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
        freight_mode.set(draft.freight_mode.clone().unwrap_or_else(|| "FTL".into()));
        visibility.set(draft.visibility.clone().unwrap_or_else(|| "public".into()));
        service_level.set(draft.service_level.clone().unwrap_or_default());
        customer_reference.set(draft.customer_reference.clone().unwrap_or_default());
        po_number.set(draft.po_number.clone().unwrap_or_default());
        pickup_appointment_ref.set(draft.pickup_appointment_ref.clone().unwrap_or_default());
        delivery_appointment_ref.set(draft.delivery_appointment_ref.clone().unwrap_or_default());
        facility_contact_name.set(draft.facility_contact_name.clone().unwrap_or_default());
        facility_contact_phone.set(draft.facility_contact_phone.clone().unwrap_or_default());
        facility_contact_email.set(draft.facility_contact_email.clone().unwrap_or_default());
        appointment_window_start.set(draft.appointment_window_start.clone().unwrap_or_default());
        appointment_window_end.set(draft.appointment_window_end.clone().unwrap_or_default());
        accessorial_flags.set(jsonish_to_form(draft.accessorial_flags.as_ref()));
        temperature_data.set(jsonish_to_form(draft.temperature_data.as_ref()));
        container_data.set(jsonish_to_form(draft.container_data.as_ref()));
        securement_data.set(jsonish_to_form(draft.securement_data.as_ref()));
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
    freight_mode.set("FTL".into());
    visibility.set("public".into());
    service_level.set(String::new());
    customer_reference.set(String::new());
    po_number.set(String::new());
    pickup_appointment_ref.set(String::new());
    delivery_appointment_ref.set(String::new());
    facility_contact_name.set(String::new());
    facility_contact_phone.set(String::new());
    facility_contact_email.set(String::new());
    appointment_window_start.set(String::new());
    appointment_window_end.set(String::new());
    accessorial_flags.set(String::new());
    temperature_data.set(String::new());
    container_data.set(String::new());
    securement_data.set(String::new());
    special_instructions.set(String::new());
    is_hazardous.set(false);
    is_temperature_controlled.set(false);
    legs.set(vec![default_leg(screen)]);
}

pub(super) fn google_maps_api_key() -> Option<String> {
    runtime_config::google_maps_api_key().or_else(|| {
        option_env!("GOOGLE_MAPS_API_KEY")
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    })
}

pub(super) fn pickup_address_input_id(index: usize) -> String {
    format!("load-leg-{}-pickup-address", index)
}
pub(super) fn pickup_city_input_id(index: usize) -> String {
    format!("load-leg-{}-pickup-city", index)
}
pub(super) fn pickup_country_input_id(index: usize) -> String {
    format!("load-leg-{}-pickup-country", index)
}
pub(super) fn pickup_place_id_input_id(index: usize) -> String {
    format!("load-leg-{}-pickup-place-id", index)
}
pub(super) fn pickup_latitude_input_id(index: usize) -> String {
    format!("load-leg-{}-pickup-latitude", index)
}
pub(super) fn pickup_longitude_input_id(index: usize) -> String {
    format!("load-leg-{}-pickup-longitude", index)
}
pub(super) fn delivery_address_input_id(index: usize) -> String {
    format!("load-leg-{}-delivery-address", index)
}
pub(super) fn delivery_city_input_id(index: usize) -> String {
    format!("load-leg-{}-delivery-city", index)
}
pub(super) fn delivery_country_input_id(index: usize) -> String {
    format!("load-leg-{}-delivery-country", index)
}
pub(super) fn delivery_place_id_input_id(index: usize) -> String {
    format!("load-leg-{}-delivery-place-id", index)
}
pub(super) fn delivery_latitude_input_id(index: usize) -> String {
    format!("load-leg-{}-delivery-latitude", index)
}
pub(super) fn delivery_longitude_input_id(index: usize) -> String {
    format!("load-leg-{}-delivery-longitude", index)
}
