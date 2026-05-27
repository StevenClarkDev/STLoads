use serde::{Deserialize, Serialize};

use crate::dispatch::LegacyLoadLegStatusCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionActionKey {
    StartPickup,
    ArrivePickup,
    DepartPickup,
    ArriveDelivery,
    CompleteDelivery,
}

impl ExecutionActionKey {
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "start_pickup" | "start_tracking" => Some(Self::StartPickup),
            "arrive_pickup" => Some(Self::ArrivePickup),
            "depart_pickup" => Some(Self::DepartPickup),
            "arrive_delivery" => Some(Self::ArriveDelivery),
            "complete_delivery" => Some(Self::CompleteDelivery),
            _ => None,
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Self::StartPickup => "start_pickup",
            Self::ArrivePickup => "arrive_pickup",
            Self::DepartPickup => "depart_pickup",
            Self::ArriveDelivery => "arrive_delivery",
            Self::CompleteDelivery => "complete_delivery",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutionTransition {
    pub action: ExecutionActionKey,
    pub from: &'static [LegacyLoadLegStatusCode],
    pub to: LegacyLoadLegStatusCode,
    pub event_type: &'static str,
    pub success_label: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ExecutionTransitionContext {
    pub has_delivery_pod: bool,
    pub has_completion_note: bool,
    pub has_tracking_consent: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionTransitionError {
    UnknownAction,
    InvalidCurrentState,
    MissingTrackingConsent,
    MissingDeliveryPod,
    MissingCompletionNote,
}

pub const EXECUTION_TRANSITIONS: &[ExecutionTransition] = &[
    ExecutionTransition {
        action: ExecutionActionKey::StartPickup,
        from: &[
            LegacyLoadLegStatusCode::Booked,
            LegacyLoadLegStatusCode::EscrowFunded,
        ],
        to: LegacyLoadLegStatusCode::PickupStarted,
        event_type: "pickup_started",
        success_label: "Pickup Started",
    },
    ExecutionTransition {
        action: ExecutionActionKey::ArrivePickup,
        from: &[LegacyLoadLegStatusCode::PickupStarted],
        to: LegacyLoadLegStatusCode::AtPickup,
        event_type: "pickup_arrived",
        success_label: "At Pickup",
    },
    ExecutionTransition {
        action: ExecutionActionKey::DepartPickup,
        from: &[LegacyLoadLegStatusCode::AtPickup],
        to: LegacyLoadLegStatusCode::InTransit,
        event_type: "departed_pickup",
        success_label: "In Transit",
    },
    ExecutionTransition {
        action: ExecutionActionKey::ArriveDelivery,
        from: &[LegacyLoadLegStatusCode::InTransit],
        to: LegacyLoadLegStatusCode::AtDelivery,
        event_type: "delivery_arrived",
        success_label: "At Delivery",
    },
    ExecutionTransition {
        action: ExecutionActionKey::CompleteDelivery,
        from: &[LegacyLoadLegStatusCode::AtDelivery],
        to: LegacyLoadLegStatusCode::Delivered,
        event_type: "delivered",
        success_label: "Delivered",
    },
];

pub fn execution_transition_for(
    current_status: i16,
    action_key: &str,
    context: ExecutionTransitionContext,
) -> Result<&'static ExecutionTransition, ExecutionTransitionError> {
    let action =
        ExecutionActionKey::parse(action_key).ok_or(ExecutionTransitionError::UnknownAction)?;
    let current = LegacyLoadLegStatusCode::from_legacy_code(current_status)
        .ok_or(ExecutionTransitionError::InvalidCurrentState)?;
    let transition = EXECUTION_TRANSITIONS
        .iter()
        .find(|transition| transition.action == action)
        .ok_or(ExecutionTransitionError::UnknownAction)?;

    if !transition.from.contains(&current) {
        return Err(ExecutionTransitionError::InvalidCurrentState);
    }

    if action == ExecutionActionKey::StartPickup && !context.has_tracking_consent {
        return Err(ExecutionTransitionError::MissingTrackingConsent);
    }

    if action == ExecutionActionKey::CompleteDelivery && !context.has_delivery_pod {
        return Err(ExecutionTransitionError::MissingDeliveryPod);
    }

    if action == ExecutionActionKey::CompleteDelivery && !context.has_completion_note {
        return Err(ExecutionTransitionError::MissingCompletionNote);
    }

    Ok(transition)
}

pub fn is_trackable_execution_status(status_id: i16) -> bool {
    matches!(status_id, 5 | 6 | 7 | 9)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ready_context() -> ExecutionTransitionContext {
        ExecutionTransitionContext {
            has_delivery_pod: true,
            has_completion_note: true,
            has_tracking_consent: true,
        }
    }

    #[test]
    fn allows_only_the_canonical_execution_path() {
        let cases = [
            (4, "start_pickup", LegacyLoadLegStatusCode::PickupStarted),
            (5, "arrive_pickup", LegacyLoadLegStatusCode::AtPickup),
            (6, "depart_pickup", LegacyLoadLegStatusCode::InTransit),
            (7, "arrive_delivery", LegacyLoadLegStatusCode::AtDelivery),
            (9, "complete_delivery", LegacyLoadLegStatusCode::Delivered),
        ];

        for (status, action, expected) in cases {
            let transition = execution_transition_for(status, action, ready_context()).unwrap();
            assert_eq!(transition.to, expected);
        }
    }

    #[test]
    fn rejects_jumped_or_unknown_transitions() {
        assert_eq!(
            execution_transition_for(4, "complete_delivery", ready_context()),
            Err(ExecutionTransitionError::InvalidCurrentState)
        );
        assert_eq!(
            execution_transition_for(7, "depart_pickup", ready_context()),
            Err(ExecutionTransitionError::InvalidCurrentState)
        );
        assert_eq!(
            execution_transition_for(9, "close_leg", ready_context()),
            Err(ExecutionTransitionError::UnknownAction)
        );
    }

    #[test]
    fn enforces_tracking_consent_before_pickup_starts() {
        let context = ExecutionTransitionContext {
            has_tracking_consent: false,
            ..ready_context()
        };

        assert_eq!(
            execution_transition_for(4, "start_pickup", context),
            Err(ExecutionTransitionError::MissingTrackingConsent)
        );
    }

    #[test]
    fn enforces_closeout_pod_and_note_preconditions() {
        assert_eq!(
            execution_transition_for(
                9,
                "complete_delivery",
                ExecutionTransitionContext {
                    has_delivery_pod: false,
                    ..ready_context()
                },
            ),
            Err(ExecutionTransitionError::MissingDeliveryPod)
        );
        assert_eq!(
            execution_transition_for(
                9,
                "complete_delivery",
                ExecutionTransitionContext {
                    has_completion_note: false,
                    ..ready_context()
                },
            ),
            Err(ExecutionTransitionError::MissingCompletionNote)
        );
    }
}
