use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i16)]
pub enum OfferStatus {
    Declined = 0,
    Pending = 1,
    Accepted = 3,
}

impl OfferStatus {
    pub const fn legacy_code(self) -> i16 {
        self as i16
    }

    pub const fn from_legacy_code(code: i16) -> Option<Self> {
        match code {
            0 => Some(Self::Declined),
            1 => Some(Self::Pending),
            3 => Some(Self::Accepted),
            _ => None,
        }
    }
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
        status: OfferStatus::Accepted,
        label: "Accepted",
        legacy_code: OfferStatus::Accepted as i16,
        description: "Offer accepted and used to book the load leg.",
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
    ],
};

pub fn offer_status_descriptors() -> &'static [OfferStatusDescriptor] {
    OFFER_STATUS_DESCRIPTORS
}

pub fn marketplace_module_contract() -> MarketplaceModuleContract {
    MARKETPLACE_MODULE_CONTRACT.clone()
}
