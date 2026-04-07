use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Coordinate {
    pub lat: f64,
    pub lng: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LegEventType {
    PickupStarted,
    PickupArrived,
    DepartedPickup,
    DeliveryArrived,
    Delivered,
    DocumentUploaded,
    LocationPing,
}

#[derive(Debug, Clone, Serialize)]
pub struct TrackingModuleContract {
    pub aggregate_tables: &'static [&'static str],
    pub trackable_status_codes: &'static [i16],
    pub realtime_channel_pattern: &'static str,
    pub realtime_event_name: &'static str,
    pub route_actions: &'static [&'static str],
    pub notes: &'static [&'static str],
}

pub const TRACKABLE_STATUS_CODES: &[i16] = &[5, 6, 7, 9];

pub const LEG_EVENT_TYPES: &[LegEventType] = &[
    LegEventType::PickupStarted,
    LegEventType::PickupArrived,
    LegEventType::DepartedPickup,
    LegEventType::DeliveryArrived,
    LegEventType::Delivered,
    LegEventType::DocumentUploaded,
    LegEventType::LocationPing,
];

pub const TRACKING_MODULE_CONTRACT: TrackingModuleContract = TrackingModuleContract {
    aggregate_tables: &["leg_locations", "leg_events", "leg_documents"],
    trackable_status_codes: TRACKABLE_STATUS_CODES,
    realtime_channel_pattern: "leg.{leg_id}.tracking",
    realtime_event_name: "LegLocationUpdated",
    route_actions: &[
        "start pickup",
        "arrive pickup",
        "depart pickup",
        "arrive delivery",
        "complete delivery",
        "upload leg document",
        "store location ping",
    ],
    notes: &[
        "tracking writes are only allowed while a booked carrier owns the leg",
        "location pings are currently accepted for status codes 5, 6, 7, and 9",
        "carrier execution actions append rows to leg_events",
    ],
};

pub fn tracking_module_contract() -> TrackingModuleContract {
    TRACKING_MODULE_CONTRACT.clone()
}

pub fn leg_event_types() -> &'static [LegEventType] {
    LEG_EVENT_TYPES
}
