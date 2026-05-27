use leptos::{prelude::*, tachys::view::any_view::IntoAny};

use super::shared::split_comma_values;
use shared::{CarrierCapacityProfile, SelfProfileFact};

pub(super) fn fact_grid(title: &'static str, facts: Vec<SelfProfileFact>) -> impl IntoView {
    view! {
        <section style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#fff;display:grid;gap:0.65rem;">
            <strong>{title}</strong>
            {if facts.is_empty() {
                view! { <p style="margin:0;color:#64748b;">"No profile facts are recorded yet."</p> }.into_any()
            } else {
                facts.into_iter().map(|fact| view! {
                    <p style="margin:0;"><strong>{fact.label}</strong>" : "{fact.value}</p>
                }).collect_view().into_any()
            }}
        </section>
    }
}

pub(super) fn join_capacity_values(values: &[String]) -> String {
    values.join(", ")
}

pub(super) fn split_capacity_values(value: String) -> Vec<String> {
    split_comma_values(value)
}

pub(super) fn optional_capacity_notes(value: String) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

pub(super) fn capacity_tone(capacity: Option<&CarrierCapacityProfile>) -> &'static str {
    match capacity.map(|item| item.readiness_label.as_str()) {
        Some(label) if label.starts_with("Eligible") => "success",
        Some(label) if label.starts_with("Limited") => "warning",
        Some(_) => "danger",
        None => "warning",
    }
}

#[derive(Clone)]
pub(super) struct LocalVerifyOutcome {
    pub(super) file_name: String,
    pub(super) hash_preview: String,
    pub(super) matches: bool,
}

pub(super) fn optional_string(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}
