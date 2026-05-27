use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i16)]
pub enum OfferStatus {
    Declined = 0,
    Pending = 1,
    Countered = 2,
    Accepted = 3,
    Withdrawn = 4,
    Expired = 5,
    Superseded = 6,
    Cancelled = 7,
}

impl OfferStatus {
    pub const fn legacy_code(self) -> i16 {
        self as i16
    }

    pub const fn from_legacy_code(code: i16) -> Option<Self> {
        match code {
            0 => Some(Self::Declined),
            1 => Some(Self::Pending),
            2 => Some(Self::Countered),
            3 => Some(Self::Accepted),
            4 => Some(Self::Withdrawn),
            5 => Some(Self::Expired),
            6 => Some(Self::Superseded),
            7 => Some(Self::Cancelled),
            _ => None,
        }
    }

    pub const fn slug(self) -> &'static str {
        match self {
            Self::Declined => "declined",
            Self::Pending => "pending",
            Self::Countered => "countered",
            Self::Accepted => "accepted",
            Self::Withdrawn => "withdrawn",
            Self::Expired => "expired",
            Self::Superseded => "superseded",
            Self::Cancelled => "cancelled",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Declined => "Declined",
            Self::Pending => "Pending",
            Self::Countered => "Countered",
            Self::Accepted => "Accepted",
            Self::Withdrawn => "Withdrawn",
            Self::Expired => "Expired",
            Self::Superseded => "Superseded",
            Self::Cancelled => "Cancelled",
        }
    }

    pub const fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Declined
                | Self::Accepted
                | Self::Withdrawn
                | Self::Expired
                | Self::Superseded
                | Self::Cancelled
        )
    }

    pub const fn is_reviewable(self) -> bool {
        matches!(self, Self::Pending | Self::Countered)
    }

    pub const fn can_transition_to(self, target: Self) -> bool {
        use OfferStatus::*;

        if self as i16 == target as i16 {
            return true;
        }

        match self {
            Pending => matches!(
                target,
                Countered | Withdrawn | Expired | Declined | Accepted | Superseded | Cancelled
            ),
            Countered => matches!(
                target,
                Pending | Withdrawn | Expired | Declined | Accepted | Superseded | Cancelled
            ),
            Declined | Accepted | Withdrawn | Expired | Superseded | Cancelled => false,
        }
    }
}

pub fn validate_offer_transition(current: OfferStatus, target: OfferStatus) -> Result<(), String> {
    current
        .can_transition_to(target)
        .then_some(())
        .ok_or_else(|| {
            format!(
                "Offer transition {} -> {} is not allowed.",
                current.slug(),
                target.slug()
            )
        })
}

#[derive(Debug, Clone, Serialize)]
pub struct OfferStatusDescriptor {
    pub status: OfferStatus,
    pub label: &'static str,
    pub legacy_code: i16,
    pub description: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct RealtimeChannelDescriptor {
    pub channel_pattern: &'static str,
    pub broadcast_as: &'static str,
    pub privacy: &'static str,
    pub purpose: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketplaceModuleContract {
    pub aggregate_tables: &'static [&'static str],
    pub booking_side_effects: &'static [&'static str],
    pub realtime_channels: &'static [RealtimeChannelDescriptor],
    pub notes: &'static [&'static str],
}

pub const OFFER_STATUS_DESCRIPTORS: &[OfferStatusDescriptor] = &[
    OfferStatusDescriptor {
        status: OfferStatus::Declined,
        label: "Declined",
        legacy_code: OfferStatus::Declined as i16,
        description: "Offer declined by the load owner or carrier.",
    },
    OfferStatusDescriptor {
        status: OfferStatus::Pending,
        label: "Pending",
        legacy_code: OfferStatus::Pending as i16,
        description: "Offer submitted and awaiting action.",
    },
    OfferStatusDescriptor {
        status: OfferStatus::Countered,
        label: "Countered",
        legacy_code: OfferStatus::Countered as i16,
        description: "Offer has an active counter awaiting response.",
    },
    OfferStatusDescriptor {
        status: OfferStatus::Accepted,
        label: "Accepted",
        legacy_code: OfferStatus::Accepted as i16,
        description: "Offer accepted and used to book the load leg.",
    },
    OfferStatusDescriptor {
        status: OfferStatus::Withdrawn,
        label: "Withdrawn",
        legacy_code: OfferStatus::Withdrawn as i16,
        description: "Offer was withdrawn by the carrier or load owner before acceptance.",
    },
    OfferStatusDescriptor {
        status: OfferStatus::Expired,
        label: "Expired",
        legacy_code: OfferStatus::Expired as i16,
        description: "Offer expired before a final decision was recorded.",
    },
    OfferStatusDescriptor {
        status: OfferStatus::Superseded,
        label: "Superseded",
        legacy_code: OfferStatus::Superseded as i16,
        description: "Offer was replaced by a newer counteroffer or accepted offer.",
    },
    OfferStatusDescriptor {
        status: OfferStatus::Cancelled,
        label: "Cancelled",
        legacy_code: OfferStatus::Cancelled as i16,
        description: "Offer was cancelled before it could continue through tendering.",
    },
];

pub const MARKETPLACE_REALTIME_CHANNELS: &[RealtimeChannelDescriptor] = &[
    RealtimeChannelDescriptor {
        channel_pattern: "private-convo.{conversation_id}",
        broadcast_as: "message.sent",
        privacy: "private",
        purpose: "Live chat messages inside a conversation thread.",
    },
    RealtimeChannelDescriptor {
        channel_pattern: "private-convo.{conversation_id}",
        broadcast_as: "offer.updated",
        privacy: "private",
        purpose: "Offer updates rendered inside the chat experience.",
    },
];

pub const MARKETPLACE_MODULE_CONTRACT: MarketplaceModuleContract = MarketplaceModuleContract {
    aggregate_tables: &["offers", "conversations", "messages", "offer_status_master"],
    booking_side_effects: &[
        "accepting an offer books the leg",
        "accepted_offer_id is written back to load_legs",
        "other pending offers are declined in the same transaction",
    ],
    realtime_channels: MARKETPLACE_REALTIME_CHANNELS,
    notes: &[
        "conversation identity is scoped to load_leg_id in runtime code",
        "legacy Laravel migration still references load_id and must be treated as drift",
        "chat and offer updates share the same user-facing workflow",
        "offer state transitions are governed by domain::marketplace::validate_offer_transition",
    ],
};

pub fn offer_status_descriptors() -> &'static [OfferStatusDescriptor] {
    OFFER_STATUS_DESCRIPTORS
}

pub fn marketplace_module_contract() -> MarketplaceModuleContract {
    MARKETPLACE_MODULE_CONTRACT.clone()
}

#[cfg(test)]
mod tests {
    use super::{OfferStatus, validate_offer_transition};

    #[test]
    fn offer_state_machine_allows_expected_active_transitions() {
        let valid = [
            (OfferStatus::Pending, OfferStatus::Countered),
            (OfferStatus::Pending, OfferStatus::Withdrawn),
            (OfferStatus::Pending, OfferStatus::Expired),
            (OfferStatus::Pending, OfferStatus::Declined),
            (OfferStatus::Pending, OfferStatus::Accepted),
            (OfferStatus::Pending, OfferStatus::Cancelled),
            (OfferStatus::Countered, OfferStatus::Pending),
            (OfferStatus::Countered, OfferStatus::Withdrawn),
            (OfferStatus::Countered, OfferStatus::Expired),
            (OfferStatus::Countered, OfferStatus::Declined),
            (OfferStatus::Countered, OfferStatus::Accepted),
            (OfferStatus::Countered, OfferStatus::Superseded),
            (OfferStatus::Countered, OfferStatus::Cancelled),
        ];

        for (current, target) in valid {
            assert!(
                validate_offer_transition(current, target).is_ok(),
                "expected {} -> {} to be valid",
                current.slug(),
                target.slug()
            );
        }
    }

    #[test]
    fn offer_state_machine_blocks_terminal_and_ambiguous_transitions() {
        let invalid = [
            (OfferStatus::Accepted, OfferStatus::Declined),
            (OfferStatus::Declined, OfferStatus::Accepted),
            (OfferStatus::Withdrawn, OfferStatus::Pending),
            (OfferStatus::Expired, OfferStatus::Accepted),
            (OfferStatus::Superseded, OfferStatus::Accepted),
            (OfferStatus::Cancelled, OfferStatus::Pending),
        ];

        for (current, target) in invalid {
            assert!(
                validate_offer_transition(current, target).is_err(),
                "expected {} -> {} to be invalid",
                current.slug(),
                target.slug()
            );
        }
    }
}
