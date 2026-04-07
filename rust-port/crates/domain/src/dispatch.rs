use serde::{Deserialize, Serialize};

/// Legacy PHP stores board, booking, execution, and finance milestones in one integer field.
/// The Rust port will split that overloaded meaning into explicit sub-state enums.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LegPostingStatus {
    Draft,
    OpenForReview,
    OpenForOffers,
    Booked,
    Withdrawn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LegExecutionStatus {
    AwaitingFunding,
    ReadyForPickup,
    PickupStarted,
    AtPickup,
    InTransit,
    AtDelivery,
    Delivered,
    PaidOut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoadDocumentKind {
    Standard,
    Blockchain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i16)]
pub enum LegacyLoadLegStatusCode {
    Draft = 0,
    New = 1,
    Reviewed = 2,
    OfferReady = 3,
    Booked = 4,
    PickupStarted = 5,
    AtPickup = 6,
    InTransit = 7,
    EscrowFunded = 8,
    AtDelivery = 9,
    Delivered = 10,
    PaidOut = 11,
}

impl LegacyLoadLegStatusCode {
    pub const fn legacy_code(self) -> i16 {
        self as i16
    }

    pub const fn from_legacy_code(code: i16) -> Option<Self> {
        match code {
            0 => Some(Self::Draft),
            1 => Some(Self::New),
            2 => Some(Self::Reviewed),
            3 => Some(Self::OfferReady),
            4 => Some(Self::Booked),
            5 => Some(Self::PickupStarted),
            6 => Some(Self::AtPickup),
            7 => Some(Self::InTransit),
            8 => Some(Self::EscrowFunded),
            9 => Some(Self::AtDelivery),
            10 => Some(Self::Delivered),
            11 => Some(Self::PaidOut),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct LegacyLoadLegStatusDescriptor {
    pub code: LegacyLoadLegStatusCode,
    pub label: &'static str,
    pub phase: &'static str,
    pub meaning: &'static str,
    pub posting_status: Option<LegPostingStatus>,
    pub execution_status: Option<LegExecutionStatus>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LoadModuleContract {
    pub aggregate_tables: &'static [&'static str],
    pub writable_tables: &'static [&'static str],
    pub required_master_data: &'static [&'static str],
    pub create_fields: &'static [&'static str],
    pub list_fields: &'static [&'static str],
    pub document_kinds: &'static [LoadDocumentKind],
    pub notes: &'static [&'static str],
}

pub const LOAD_DOCUMENT_KINDS: &[LoadDocumentKind] =
    &[LoadDocumentKind::Standard, LoadDocumentKind::Blockchain];

pub const LEGACY_LOAD_LEG_STATUS_DESCRIPTORS: &[LegacyLoadLegStatusDescriptor] = &[
    LegacyLoadLegStatusDescriptor {
        code: LegacyLoadLegStatusCode::Draft,
        label: "Draft",
        phase: "posting",
        meaning: "Pre-release state observed in the Laravel codebase.",
        posting_status: Some(LegPostingStatus::Draft),
        execution_status: None,
    },
    LegacyLoadLegStatusDescriptor {
        code: LegacyLoadLegStatusCode::New,
        label: "New",
        phase: "posting",
        meaning: "Freshly created leg awaiting review and board workflow.",
        posting_status: Some(LegPostingStatus::OpenForReview),
        execution_status: None,
    },
    LegacyLoadLegStatusDescriptor {
        code: LegacyLoadLegStatusCode::Reviewed,
        label: "Reviewed",
        phase: "posting",
        meaning: "Intermediate reviewed state before open offer handling.",
        posting_status: Some(LegPostingStatus::OpenForOffers),
        execution_status: None,
    },
    LegacyLoadLegStatusDescriptor {
        code: LegacyLoadLegStatusCode::OfferReady,
        label: "Offer Ready",
        phase: "posting",
        meaning: "Offer collection state before a carrier is booked.",
        posting_status: Some(LegPostingStatus::OpenForOffers),
        execution_status: None,
    },
    LegacyLoadLegStatusDescriptor {
        code: LegacyLoadLegStatusCode::Booked,
        label: "Booked",
        phase: "booking",
        meaning: "Offer accepted and carrier assigned.",
        posting_status: Some(LegPostingStatus::Booked),
        execution_status: Some(LegExecutionStatus::AwaitingFunding),
    },
    LegacyLoadLegStatusDescriptor {
        code: LegacyLoadLegStatusCode::PickupStarted,
        label: "Pickup Started",
        phase: "execution",
        meaning: "Carrier confirmed the pickup workflow has started.",
        posting_status: Some(LegPostingStatus::Booked),
        execution_status: Some(LegExecutionStatus::PickupStarted),
    },
    LegacyLoadLegStatusDescriptor {
        code: LegacyLoadLegStatusCode::AtPickup,
        label: "At Pickup",
        phase: "execution",
        meaning: "Carrier arrived at pickup.",
        posting_status: Some(LegPostingStatus::Booked),
        execution_status: Some(LegExecutionStatus::AtPickup),
    },
    LegacyLoadLegStatusDescriptor {
        code: LegacyLoadLegStatusCode::InTransit,
        label: "In Transit",
        phase: "execution",
        meaning: "Carrier departed pickup and is moving to delivery.",
        posting_status: Some(LegPostingStatus::Booked),
        execution_status: Some(LegExecutionStatus::InTransit),
    },
    LegacyLoadLegStatusDescriptor {
        code: LegacyLoadLegStatusCode::EscrowFunded,
        label: "Escrow Funded",
        phase: "funding",
        meaning: "Funding gate satisfied and execution may start.",
        posting_status: Some(LegPostingStatus::Booked),
        execution_status: Some(LegExecutionStatus::ReadyForPickup),
    },
    LegacyLoadLegStatusDescriptor {
        code: LegacyLoadLegStatusCode::AtDelivery,
        label: "At Delivery",
        phase: "execution",
        meaning: "Carrier arrived at delivery.",
        posting_status: Some(LegPostingStatus::Booked),
        execution_status: Some(LegExecutionStatus::AtDelivery),
    },
    LegacyLoadLegStatusDescriptor {
        code: LegacyLoadLegStatusCode::Delivered,
        label: "Delivered",
        phase: "closeout",
        meaning: "Physical delivery is complete.",
        posting_status: Some(LegPostingStatus::Booked),
        execution_status: Some(LegExecutionStatus::Delivered),
    },
    LegacyLoadLegStatusDescriptor {
        code: LegacyLoadLegStatusCode::PaidOut,
        label: "Paid Out",
        phase: "finance",
        meaning: "Financial completion after delivery.",
        posting_status: Some(LegPostingStatus::Booked),
        execution_status: Some(LegExecutionStatus::PaidOut),
    },
];

pub const LOAD_MODULE_CONTRACT: LoadModuleContract = LoadModuleContract {
    aggregate_tables: &[
        "loads",
        "load_legs",
        "load_documents",
        "load_history",
        "locations",
    ],
    writable_tables: &[
        "loads",
        "load_legs",
        "load_documents",
        "load_history",
        "carrier_preferences",
    ],
    required_master_data: &[
        "countries",
        "cities",
        "locations",
        "load_types",
        "equipments",
        "commodity_types",
        "load_status_master",
    ],
    create_fields: &[
        "title",
        "load_type_id",
        "equipment_id",
        "commodity_type_id",
        "weight_unit",
        "weight",
        "special_instructions",
        "is_hazardous",
        "is_temperature_controlled",
        "documents[]",
        "legs[]",
    ],
    list_fields: &[
        "load_number",
        "title",
        "status",
        "leg_count",
        "latest_leg_status",
        "user_id",
        "created_at",
    ],
    document_kinds: LOAD_DOCUMENT_KINDS,
    notes: &[
        "load lifecycle is still anchored primarily on load_legs.status_id",
        "load documents support both standard and blockchain-marked uploads",
        "carrier preferences influence load board filtering and matching",
    ],
};

pub fn legacy_load_leg_status_descriptors() -> &'static [LegacyLoadLegStatusDescriptor] {
    LEGACY_LOAD_LEG_STATUS_DESCRIPTORS
}

pub fn load_module_contract() -> LoadModuleContract {
    LOAD_MODULE_CONTRACT.clone()
}
