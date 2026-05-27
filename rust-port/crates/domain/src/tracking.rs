use serde::{Deserialize, Serialize};

use crate::execution::is_trackable_execution_status;

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

pub fn can_store_location_ping(status_id: i16, has_tracking_consent: bool) -> bool {
    has_tracking_consent && is_trackable_execution_status(status_id)
}

pub fn haversine_km(first: Coordinate, second: Coordinate) -> f64 {
    let earth_radius_km = 6371.0_f64;
    let d_lat = (second.lat - first.lat).to_radians();
    let d_lng = (second.lng - first.lng).to_radians();
    let first_lat = first.lat.to_radians();
    let second_lat = second.lat.to_radians();

    let a = (d_lat / 2.0).sin().powi(2)
        + first_lat.cos() * second_lat.cos() * (d_lng / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    earth_radius_km * c
}

pub fn is_inside_geofence(point: Coordinate, stop: Coordinate, radius_km: f64) -> bool {
    haversine_km(point, stop) <= radius_km
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn location_pings_require_active_state_and_consent() {
        assert!(can_store_location_ping(7, true));
        assert!(!can_store_location_ping(7, false));
        assert!(!can_store_location_ping(4, true));
    }

    #[test]
    fn geofence_checks_distance_from_stop() {
        let stop = Coordinate {
            lat: 32.7767,
            lng: -96.7970,
        };
        let nearby = Coordinate {
            lat: 32.7770,
            lng: -96.7980,
        };
        let far = Coordinate {
            lat: 35.1495,
            lng: -90.0490,
        };

        assert!(is_inside_geofence(nearby, stop, 0.25));
        assert!(!is_inside_geofence(far, stop, 0.25));
    }
}
