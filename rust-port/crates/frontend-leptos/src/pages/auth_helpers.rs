use leptos::{prelude::*, task::spawn_local};
use leptos_router::components::A;
use serde::{Deserialize, Serialize};

use crate::{google_places, runtime_config, session::use_auth};
use shared::OtpPurpose;
#[component]
pub(crate) fn AuthArticle(
    #[prop(into)] title: Signal<String>,
    #[prop(into)] subtitle: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <article class="auth-article">
            <section class="auth-title-block">
                <span class="auth-kicker">"STLoads Portal"</span>
                <h2 class="auth-title">{move || title.get()}</h2>
                <p class="auth-subtitle">{move || subtitle.get()}</p>
            </section>
            <div class="auth-surface">{children()}</div>
        </article>
    }
}

#[component]
pub(crate) fn RoleRegisterSwitcher(current_role: RwSignal<String>) -> impl IntoView {
    view! {
        <section class="auth-field">
            <span class="auth-label">"Signup role"</span>
            <div class="auth-role-picker">
                <A href="/auth/register?role=shipper" attr:class=move || selection_option_class(current_role.get() == "shipper")>
                    <span class="auth-role-option-title">"Shipper"</span>
                    <span class="auth-role-option-copy">"Warehouse, fulfillment, and facility details."</span>
                </A>
                <A href="/auth/register?role=carrier" attr:class=move || selection_option_class(current_role.get() == "carrier")>
                    <span class="auth-role-option-title">"Carrier"</span>
                    <span class="auth-role-option-copy">"DOT, MC, and equipment-focused onboarding."</span>
                </A>
                <A href="/auth/register?role=broker" attr:class=move || selection_option_class(current_role.get() == "broker")>
                    <span class="auth-role-option-title">"Broker"</span>
                    <span class="auth-role-option-copy">"Broker license and authority details."</span>
                </A>
                <A href="/auth/register?role=freight_forwarder" attr:class=move || selection_option_class(current_role.get() == "freight_forwarder")>
                    <span class="auth-role-option-title">"Freight Forwarder"</span>
                    <span class="auth-role-option-copy">"Forwarder and customs licensing information."</span>
                </A>
            </div>
        </section>
    }
}

#[component]
pub(crate) fn RegisterStepHeader(
    role: RwSignal<String>,
    current_step: RwSignal<usize>,
) -> impl IntoView {
    let account_label = Signal::derive(move || register_step_labels(role.get())[0].to_string());
    let detail_label = Signal::derive(move || register_step_labels(role.get())[1].to_string());
    view! {
        <section class="auth-field">
            <span class="auth-label">"Signup steps"</span>
            <div class="auth-role-picker">
                <div class=move || selection_option_class(current_step.get() == 0)>
                    <span class="auth-role-option-title">{move || account_label.get()}</span>
                    <span class="auth-role-option-copy">"Create the account credentials first."</span>
                </div>
                <div class=move || selection_option_class(current_step.get() == 1)>
                    <span class="auth-role-option-title">{move || detail_label.get()}</span>
                    <span class="auth-role-option-copy">"Match the role-specific business fields from the PHP flow."</span>
                </div>
            </div>
        </section>
    }
}

#[component]
pub(crate) fn SharedNotice() -> impl IntoView {
    let auth = use_auth();
    view! {
        {move || auth.notice.get().map(|message| view! {
            <section class="auth-notice">
                {message}
            </section>
        })}
    }
}

#[component]
pub(crate) fn LocalNotice(message: RwSignal<Option<String>>) -> impl IntoView {
    view! {
        {move || message.get().map(|message| view! {
            <section class="auth-notice" style="white-space:pre-wrap;">
                {message}
            </section>
        })}
    }
}

#[component]
pub(crate) fn TextField(
    label: &'static str,
    value: RwSignal<String>,
    input_type: &'static str,
    placeholder: &'static str,
) -> impl IntoView {
    view! {
        <label class="auth-field">
            <span class="auth-label">{label}</span>
            <input
                type=input_type
                prop:value=move || value.get()
                on:input=move |ev| value.set(event_target_value(&ev))
                placeholder=placeholder
                class="auth-input"
            />
        </label>
    }
}

#[component]
pub(crate) fn EmailField(label: &'static str, value: RwSignal<String>) -> impl IntoView {
    view! {
        <label class="auth-field">
            <span class="auth-label">{label}</span>
            <input
                type="email"
                prop:value=move || value.get()
                on:input=move |ev| value.set(event_target_value(&ev).trim().to_string())
                placeholder="abc@xyz.com"
                class="auth-input"
                inputmode="email"
                autocomplete="email"
            />
            <small class="auth-help">"Use a valid email like abc@xyz.com"</small>
        </label>
    }
}

#[component]
pub(crate) fn PhoneField(label: &'static str, value: RwSignal<String>) -> impl IntoView {
    let selected_country = RwSignal::new(default_phone_country().iso.to_string());
    let national_number = RwSignal::new(phone_national_input_from_value(&value.get_untracked()));

    let sync_phone_value = move || {
        let country = phone_country_by_iso(&selected_country.get());
        let national_digits = phone_digits(&national_number.get());
        if national_digits.is_empty() {
            value.set(String::new());
        } else {
            value.set(format_international_phone(country, &national_digits));
        }
    };

    view! {
        <label class="auth-field">
            <span class="auth-label">{label}</span>
            <div style="display:grid;grid-template-columns:minmax(150px, 0.55fr) minmax(0, 1fr);gap:0.65rem;">
                <select
                    class="auth-select"
                    aria-label="Phone country code"
                    prop:value=move || selected_country.get()
                    on:change=move |ev| {
                        selected_country.set(event_target_value(&ev));
                        sync_phone_value();
                    }
                >
                    {PHONE_COUNTRIES.iter().map(|country| view! {
                        <option value=country.iso>{format!("{} +{}", country.label, country.dial_code)}</option>
                    }).collect_view()}
                </select>
                <input
                    type="tel"
                    prop:value=move || format_national_phone(
                        phone_country_by_iso(&selected_country.get()),
                        &phone_digits(&national_number.get()),
                    )
                    on:input=move |ev| {
                        let country = phone_country_by_iso(&selected_country.get());
                        let digits = phone_digits(&event_target_value(&ev));
                        national_number.set(limit_phone_digits_for_country(country, &digits));
                        sync_phone_value();
                    }
                    placeholder=move || phone_country_by_iso(&selected_country.get()).placeholder
                    class="auth-input"
                    inputmode="tel"
                    autocomplete="tel-national"
                />
            </div>
            <small class="auth-help">
                {move || format!(
                    "Country code is selected automatically from your browser region. Saved as {}.",
                    phone_country_by_iso(&selected_country.get()).example
                )}
            </small>
        </label>
    }
}

#[component]
pub(crate) fn PasswordField(
    label: &'static str,
    value: RwSignal<String>,
    placeholder: &'static str,
) -> impl IntoView {
    let visible = RwSignal::new(false);
    view! {
        <label class="auth-field">
            <span class="auth-label">{label}</span>
            <div class="auth-input-shell">
                <input
                    type=move || if visible.get() { "text" } else { "password" }
                    prop:value=move || value.get()
                    on:input=move |ev| value.set(event_target_value(&ev))
                    placeholder=placeholder
                    class="auth-input auth-input-with-action"
                    autocomplete="current-password"
                />
                <button
                    type="button"
                    class="auth-input-action"
                    on:click=move |_| visible.update(|state| *state = !*state)
                    aria-label=move || if visible.get() { "Hide password" } else { "Show password" }
                    title=move || if visible.get() { "Hide password" } else { "Show password" }
                >
                    <i class=move || if visible.get() { "fas fa-eye-slash" } else { "fas fa-eye" }></i>
                </button>
            </div>
        </label>
    }
}

#[component]
pub(crate) fn AddressAutocompleteField(
    label: &'static str,
    value: RwSignal<String>,
    placeholder: &'static str,
    #[prop(into)] input_id: String,
) -> impl IntoView {
    let input_id_for_effect = input_id.clone();
    let city = RwSignal::new(String::new());
    let country = RwSignal::new(String::new());
    let place_id = RwSignal::new(String::new());
    let latitude = RwSignal::new(String::new());
    let longitude = RwSignal::new(String::new());
    let google_status = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        let api_key = runtime_config::google_maps_api_key();
        let input_id = input_id_for_effect.clone();
        let city_id = format!("{}-city", input_id);
        let country_id = format!("{}-country", input_id);
        let place_id_id = format!("{}-place-id", input_id);
        let latitude_id = format!("{}-latitude", input_id);
        let longitude_id = format!("{}-longitude", input_id);

        spawn_local(async move {
            match api_key {
                Some(api_key) => {
                    if let Err(error) = google_places::ensure_loaded(&api_key).await {
                        google_status.set(Some(error));
                        return;
                    }
                    if let Err(error) = google_places::attach_address_autocomplete(
                        &input_id,
                        &city_id,
                        &country_id,
                        &place_id_id,
                        &latitude_id,
                        &longitude_id,
                    )
                    .await
                    {
                        google_status.set(Some(error));
                    }
                }
                None => {
                    google_status.set(Some(
                        "Address suggestions are temporarily unavailable.".into(),
                    ));
                }
            }
        });
    });

    view! {
        <label class="auth-field">
            <span class="auth-label">{label}</span>
            <input
                id=input_id.clone()
                type="text"
                prop:value=move || value.get()
                on:input=move |ev| value.set(event_target_value(&ev))
                placeholder=placeholder
                class="auth-input"
                autocomplete="street-address"
            />
            <input id=format!("{}-city", input_id) type="hidden" prop:value=move || city.get() on:input=move |ev| city.set(event_target_value(&ev)) />
            <input id=format!("{}-country", input_id) type="hidden" prop:value=move || country.get() on:input=move |ev| country.set(event_target_value(&ev)) />
            <input id=format!("{}-place-id", input_id) type="hidden" prop:value=move || place_id.get() on:input=move |ev| place_id.set(event_target_value(&ev)) />
            <input id=format!("{}-latitude", input_id) type="hidden" prop:value=move || latitude.get() on:input=move |ev| latitude.set(event_target_value(&ev)) />
            <input id=format!("{}-longitude", input_id) type="hidden" prop:value=move || longitude.get() on:input=move |ev| longitude.set(event_target_value(&ev)) />
            <small class="auth-help">"Start typing and choose an address from Google suggestions."</small>
            {move || google_status.get().map(|message| view! {
                <small class="auth-help">{message}</small>
            })}
        </label>
    }
}

#[component]
pub(crate) fn TextAreaField(
    label: &'static str,
    value: RwSignal<String>,
    placeholder: &'static str,
) -> impl IntoView {
    view! {
        <label class="auth-field">
            <span class="auth-label">{label}</span>
            <textarea
                prop:value=move || value.get()
                on:input=move |ev| value.set(event_target_value(&ev))
                placeholder=placeholder
                rows="3"
                class="auth-textarea"
            ></textarea>
        </label>
    }
}

#[component]
pub(crate) fn OtpPurposeField(value: RwSignal<OtpPurpose>) -> impl IntoView {
    view! {
        <section class="auth-field">
            <span class="auth-label">"OTP purpose"</span>
            <div class="auth-role-picker">
                <button
                    type="button"
                    class=move || selection_option_class(matches!(value.get(), OtpPurpose::Registration))
                    on:click=move |_| value.set(OtpPurpose::Registration)
                >
                    <span class="auth-role-option-title">"Registration"</span>
                    <span class="auth-role-option-copy">"Verify a newly created account and continue onboarding."</span>
                </button>
                <button
                    type="button"
                    class=move || selection_option_class(matches!(value.get(), OtpPurpose::PasswordReset))
                    on:click=move |_| value.set(OtpPurpose::PasswordReset)
                >
                    <span class="auth-role-option-title">"Password reset"</span>
                    <span class="auth-role-option-copy">"Use OTP to recover access and set a new password."</span>
                </button>
            </div>
        </section>
    }
}

pub(crate) fn optional_string(value: String) -> Option<String> {
    let trimmed = value.trim().to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

pub(crate) fn button_style(color: &'static str) -> String {
    format!(
        "padding:0.7rem 1rem;border:none;border-radius:0.85rem;background:{};color:white;cursor:pointer;font-weight:700;box-shadow:0 14px 32px rgba(15,23,42,0.08);",
        color
    )
}

pub(crate) fn tone_style(tone: &str) -> &'static str {
    match tone {
        "success" => "color:#166534;font-weight:700;",
        "warning" => "color:#92400e;font-weight:700;",
        "danger" => "color:#991b1b;font-weight:700;",
        _ => "color:#334155;font-weight:700;",
    }
}

pub(crate) fn selection_option_class(active: bool) -> &'static str {
    if active {
        "auth-role-option is-active"
    } else {
        "auth-role-option"
    }
}

pub(crate) fn is_supported_role(value: &str) -> bool {
    matches!(
        value,
        "shipper" | "carrier" | "broker" | "freight_forwarder"
    )
}

pub(crate) fn register_step_labels(role_key: String) -> [&'static str; 2] {
    match role_key.as_str() {
        "carrier" => ["Identity", "Carrier Details"],
        "broker" => ["Identity", "Brokerage Details"],
        "freight_forwarder" => ["Company", "Licensing Details"],
        _ => ["Account", "Company Details"],
    }
}

pub(crate) fn role_label(value: String) -> &'static str {
    match value.as_str() {
        "carrier" => "Carrier",
        "broker" => "Broker",
        "freight_forwarder" => "Freight Forwarder",
        _ => "Shipper",
    }
}

pub(crate) fn encode_query_value(value: &str) -> String {
    urlencoding::encode(value).into_owned()
}

pub(crate) fn build_verify_otp_path(email: &str, purpose: OtpPurpose) -> String {
    format!(
        "/auth/verify-otp?email={}&purpose={}",
        encode_query_value(email),
        otp_purpose_query_value(purpose)
    )
}

pub(crate) fn build_mfa_path(email: &str, challenge_id: &str) -> String {
    format!(
        "/auth/mfa?email={}&challenge={}",
        encode_query_value(email),
        encode_query_value(challenge_id)
    )
}

pub(crate) fn build_reset_password_path(email: &str, token: &str) -> String {
    format!(
        "/auth/reset-password?email={}&token={}",
        encode_query_value(email),
        encode_query_value(token)
    )
}

pub(crate) fn otp_purpose_query_value(purpose: OtpPurpose) -> &'static str {
    match purpose {
        OtpPurpose::Registration => "registration",
        OtpPurpose::PasswordReset => "password_reset",
    }
}

pub(crate) fn otp_purpose_from_query(value: &str) -> Option<OtpPurpose> {
    match value {
        "registration" => Some(OtpPurpose::Registration),
        "password_reset" | "reset" => Some(OtpPurpose::PasswordReset),
        _ => None,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct PendingOnboardingDraft {
    pub(crate) role_key: String,
    pub(crate) company_name: Option<String>,
    pub(crate) company_address: Option<String>,
    pub(crate) dot_number: Option<String>,
    pub(crate) mc_number: Option<String>,
    pub(crate) equipment_types: Option<String>,
    pub(crate) business_entity_id: Option<String>,
    pub(crate) facility_address: Option<String>,
    pub(crate) fulfillment_contact_info: Option<String>,
    pub(crate) fmcsa_broker_license_no: Option<String>,
    pub(crate) mc_authority_number: Option<String>,
    pub(crate) freight_forwarder_license: Option<String>,
    pub(crate) customs_license: Option<String>,
}

pub(crate) fn validate_register_step(
    step: usize,
    role_key: &str,
    name: &str,
    email: &str,
    phone: &str,
    address: &str,
    password: &str,
    password_confirmation: &str,
    company_name: &str,
    company_address: &str,
    dot_number: &str,
    mc_number: &str,
    equipment_types: &str,
    business_entity_id: &str,
    facility_address: &str,
    fulfillment_contact_info: &str,
    fmcsa_broker_license_no: &str,
    mc_authority_number: &str,
    freight_forwarder_license: &str,
    customs_license: &str,
) -> Option<String> {
    if step == 0 {
        if name.trim().is_empty()
            || email.trim().is_empty()
            || phone.trim().is_empty()
            || address.trim().is_empty()
            || password.is_empty()
            || password_confirmation.is_empty()
        {
            return Some(
                "Complete all account fields before moving to the role-specific step.".into(),
            );
        }
        if password != password_confirmation {
            return Some("Passwords do not match.".into());
        }
        if password.len() < 8 {
            return Some("Use a password with at least 8 characters.".into());
        }
        if !is_valid_email_format(email) {
            return Some("Enter a valid email in the format abc@xyz.com.".into());
        }
        if !is_valid_phone_format(phone) {
            return Some("Enter a valid phone number with a country code.".into());
        }
        return None;
    }

    if company_name.trim().is_empty() || company_address.trim().is_empty() {
        return Some("Company name and company address are required.".into());
    }

    match role_key {
        "carrier"
            if dot_number.trim().is_empty()
                || mc_number.trim().is_empty()
                || equipment_types.trim().is_empty() =>
        {
            Some("Carrier signup requires DOT number, MC number, and equipment types.".into())
        }
        "shipper"
            if business_entity_id.trim().is_empty()
                || facility_address.trim().is_empty()
                || fulfillment_contact_info.trim().is_empty() =>
        {
            Some("Shipper signup requires business entity ID, facility address, and fulfillment contact info.".into())
        }
        "broker"
            if fmcsa_broker_license_no.trim().is_empty()
                || mc_authority_number.trim().is_empty() =>
        {
            Some("Broker signup requires FMCSA broker license and MC authority number.".into())
        }
        "freight_forwarder"
            if freight_forwarder_license.trim().is_empty()
                || customs_license.trim().is_empty() =>
        {
            Some("Freight forwarder signup requires freight forwarder and customs license values.".into())
        }
        _ => None,
    }
}

pub(crate) fn is_valid_email_format(value: &str) -> bool {
    let trimmed = value.trim();
    let mut parts = trimmed.split('@');
    let local = parts.next().unwrap_or_default();
    let domain = parts.next().unwrap_or_default();
    parts.next().is_none()
        && !local.is_empty()
        && domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
}

pub(crate) fn is_valid_phone_format(value: &str) -> bool {
    let trimmed = value.trim();
    if !trimmed.starts_with('+') {
        return false;
    }

    let digits = phone_digits(trimmed);
    (8..=15).contains(&digits.len())
}

#[derive(Clone, Copy)]
struct PhoneCountry {
    iso: &'static str,
    label: &'static str,
    dial_code: &'static str,
    national_max_digits: usize,
    placeholder: &'static str,
    example: &'static str,
}

const PHONE_COUNTRIES: &[PhoneCountry] = &[
    PhoneCountry {
        iso: "US",
        label: "United States",
        dial_code: "1",
        national_max_digits: 10,
        placeholder: "(555) 010-1234",
        example: "+1 (555) 010-1234",
    },
    PhoneCountry {
        iso: "CA",
        label: "Canada",
        dial_code: "1",
        national_max_digits: 10,
        placeholder: "(416) 555-0100",
        example: "+1 (416) 555-0100",
    },
    PhoneCountry {
        iso: "PK",
        label: "Pakistan",
        dial_code: "92",
        national_max_digits: 10,
        placeholder: "300 1234567",
        example: "+92 300 1234567",
    },
    PhoneCountry {
        iso: "IN",
        label: "India",
        dial_code: "91",
        national_max_digits: 10,
        placeholder: "98765 43210",
        example: "+91 98765 43210",
    },
    PhoneCountry {
        iso: "GB",
        label: "United Kingdom",
        dial_code: "44",
        national_max_digits: 10,
        placeholder: "7400 123456",
        example: "+44 7400 123456",
    },
    PhoneCountry {
        iso: "MX",
        label: "Mexico",
        dial_code: "52",
        national_max_digits: 10,
        placeholder: "55 1234 5678",
        example: "+52 55 1234 5678",
    },
    PhoneCountry {
        iso: "AU",
        label: "Australia",
        dial_code: "61",
        national_max_digits: 9,
        placeholder: "412 345 678",
        example: "+61 412 345 678",
    },
    PhoneCountry {
        iso: "AE",
        label: "United Arab Emirates",
        dial_code: "971",
        national_max_digits: 9,
        placeholder: "50 123 4567",
        example: "+971 50 123 4567",
    },
];

fn default_phone_country() -> PhoneCountry {
    let region = browser_region_code();
    PHONE_COUNTRIES
        .iter()
        .copied()
        .find(|country| country.iso == region)
        .unwrap_or(PHONE_COUNTRIES[0])
}

fn phone_country_by_iso(iso: &str) -> PhoneCountry {
    PHONE_COUNTRIES
        .iter()
        .copied()
        .find(|country| country.iso == iso)
        .unwrap_or(PHONE_COUNTRIES[0])
}

fn phone_digits(value: &str) -> String {
    value.chars().filter(|ch| ch.is_ascii_digit()).collect()
}

fn limit_phone_digits_for_country(country: PhoneCountry, digits: &str) -> String {
    digits
        .chars()
        .take(country.national_max_digits)
        .collect::<String>()
}

fn phone_national_input_from_value(value: &str) -> String {
    let digits = phone_digits(value);
    PHONE_COUNTRIES
        .iter()
        .find_map(|country| digits.strip_prefix(country.dial_code))
        .unwrap_or(digits.as_str())
        .to_string()
}

fn format_international_phone(country: PhoneCountry, national_digits: &str) -> String {
    let limited = limit_phone_digits_for_country(country, national_digits);
    let national = format_national_phone(country, &limited);
    if national.is_empty() {
        String::new()
    } else {
        format!("+{} {}", country.dial_code, national)
    }
}

fn format_national_phone(country: PhoneCountry, digits: &str) -> String {
    let digits = limit_phone_digits_for_country(country, digits);
    match country.iso {
        "US" | "CA" => format_north_american_phone(&digits),
        "PK" => format_grouped_phone(&digits, &[3, 7]),
        "IN" => format_grouped_phone(&digits, &[5, 5]),
        "GB" => format_grouped_phone(&digits, &[4, 6]),
        "MX" => format_grouped_phone(&digits, &[2, 4, 4]),
        "AU" => format_grouped_phone(&digits, &[3, 3, 3]),
        "AE" => format_grouped_phone(&digits, &[2, 3, 4]),
        _ => format_grouped_phone(&digits, &[3, 3, 4]),
    }
}

fn format_north_american_phone(digits: &str) -> String {
    match digits.len() {
        0 => String::new(),
        1..=3 => format!("({digits}"),
        4..=6 => format!("({}) {}", &digits[..3], &digits[3..]),
        _ => format!("({}) {}-{}", &digits[..3], &digits[3..6], &digits[6..]),
    }
}

fn format_grouped_phone(digits: &str, groups: &[usize]) -> String {
    let mut remaining = digits;
    let mut parts = Vec::new();
    for group_len in groups {
        if remaining.is_empty() {
            break;
        }
        let take = (*group_len).min(remaining.len());
        parts.push(remaining[..take].to_string());
        remaining = &remaining[take..];
    }
    if !remaining.is_empty() {
        parts.push(remaining.to_string());
    }
    parts.join(" ")
}

#[cfg(target_arch = "wasm32")]
fn browser_region_code() -> String {
    web_sys::window()
        .and_then(|window| window.navigator().language())
        .and_then(|language| {
            language
                .split(['-', '_'])
                .nth(1)
                .map(|region| region.to_ascii_uppercase())
        })
        .unwrap_or_else(|| "US".to_string())
}

#[cfg(not(target_arch = "wasm32"))]
fn browser_region_code() -> String {
    "US".to_string()
}

pub(crate) fn prefill_auth_field(
    server_value: Option<String>,
    local_value: Option<String>,
) -> String {
    server_value
        .filter(|value| !value.trim().is_empty())
        .or(local_value.filter(|value| !value.trim().is_empty()))
        .unwrap_or_default()
}

#[cfg(target_arch = "wasm32")]
const PENDING_ONBOARDING_DRAFT_KEY: &str = "stloads.pending-onboarding-draft";

#[cfg(target_arch = "wasm32")]
pub(crate) fn save_pending_onboarding_draft(draft: &PendingOnboardingDraft) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(serialized) = serde_json::to_string(draft) {
                let _ = storage.set_item(PENDING_ONBOARDING_DRAFT_KEY, &serialized);
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn save_pending_onboarding_draft(_draft: &PendingOnboardingDraft) {}

#[cfg(target_arch = "wasm32")]
pub(crate) fn load_pending_onboarding_draft() -> Option<PendingOnboardingDraft> {
    let window = web_sys::window()?;
    let storage = window.local_storage().ok()??;
    let serialized = storage.get_item(PENDING_ONBOARDING_DRAFT_KEY).ok()??;
    serde_json::from_str(&serialized).ok()
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn load_pending_onboarding_draft() -> Option<PendingOnboardingDraft> {
    None
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn clear_pending_onboarding_draft() {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.remove_item(PENDING_ONBOARDING_DRAFT_KEY);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn clear_pending_onboarding_draft() {}
