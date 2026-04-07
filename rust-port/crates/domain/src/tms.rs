use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HandoffStatus {
    Queued,
    PushInProgress,
    Published,
    PushFailed,
    RequeueRequired,
    Withdrawn,
    Closed,
}

impl HandoffStatus {
    pub const fn as_legacy_label(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::PushInProgress => "push_in_progress",
            Self::Published => "published",
            Self::PushFailed => "push_failed",
            Self::RequeueRequired => "requeue_required",
            Self::Withdrawn => "withdrawn",
            Self::Closed => "closed",
        }
    }

    pub fn from_legacy_label(label: &str) -> Option<Self> {
        match label {
            "queued" => Some(Self::Queued),
            "push_in_progress" => Some(Self::PushInProgress),
            "published" => Some(Self::Published),
            "push_failed" => Some(Self::PushFailed),
            "requeue_required" => Some(Self::RequeueRequired),
            "withdrawn" => Some(Self::Withdrawn),
            "closed" => Some(Self::Closed),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TmsStatus {
    Dispatched,
    InTransit,
    AtPickup,
    AtDelivery,
    Delivered,
    Cancelled,
    Invoiced,
    Settled,
}

impl TmsStatus {
    pub const fn as_legacy_label(self) -> &'static str {
        match self {
            Self::Dispatched => "dispatched",
            Self::InTransit => "in_transit",
            Self::AtPickup => "at_pickup",
            Self::AtDelivery => "at_delivery",
            Self::Delivered => "delivered",
            Self::Cancelled => "cancelled",
            Self::Invoiced => "invoiced",
            Self::Settled => "settled",
        }
    }

    pub fn from_legacy_label(label: &str) -> Option<Self> {
        match label {
            "dispatched" => Some(Self::Dispatched),
            "in_transit" => Some(Self::InTransit),
            "at_pickup" => Some(Self::AtPickup),
            "at_delivery" => Some(Self::AtDelivery),
            "delivered" => Some(Self::Delivered),
            "cancelled" => Some(Self::Cancelled),
            "invoiced" => Some(Self::Invoiced),
            "settled" => Some(Self::Settled),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReconciliationAction {
    StatusUpdate,
    AutoWithdraw,
    AutoClose,
    AutoArchive,
    RateUpdate,
    MismatchDetected,
    ForceSync,
}

impl ReconciliationAction {
    pub const fn as_legacy_label(self) -> &'static str {
        match self {
            Self::StatusUpdate => "status_update",
            Self::AutoWithdraw => "auto_withdraw",
            Self::AutoClose => "auto_close",
            Self::AutoArchive => "auto_archive",
            Self::RateUpdate => "rate_update",
            Self::MismatchDetected => "mismatch_detected",
            Self::ForceSync => "force_sync",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct HandoffStatusDescriptor {
    pub status: HandoffStatus,
    pub legacy_label: &'static str,
    pub description: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct TmsStatusDescriptor {
    pub status: TmsStatus,
    pub legacy_label: &'static str,
    pub description: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct TmsWebhookSurfaceDescriptor {
    pub method: &'static str,
    pub route: &'static str,
    pub purpose: &'static str,
    pub payload_keys: &'static [&'static str],
}

#[derive(Debug, Clone, Serialize)]
pub struct ReconciliationActionDescriptor {
    pub action: ReconciliationAction,
    pub legacy_label: &'static str,
    pub description: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct TmsModuleContract {
    pub aggregate_tables: &'static [&'static str],
    pub webhook_surfaces: &'static [TmsWebhookSurfaceDescriptor],
    pub reconciliation_actions: &'static [ReconciliationActionDescriptor],
    pub notes: &'static [&'static str],
}

pub const HANDOFF_STATUS_DESCRIPTORS: &[HandoffStatusDescriptor] = &[
    HandoffStatusDescriptor {
        status: HandoffStatus::Queued,
        legacy_label: HandoffStatus::Queued.as_legacy_label(),
        description: "TMS payload is accepted but not yet pushed to the public board.",
    },
    HandoffStatusDescriptor {
        status: HandoffStatus::PushInProgress,
        legacy_label: HandoffStatus::PushInProgress.as_legacy_label(),
        description: "Publish work is actively trying to create or update the local load.",
    },
    HandoffStatusDescriptor {
        status: HandoffStatus::Published,
        legacy_label: HandoffStatus::Published.as_legacy_label(),
        description: "The handoff is live on STLOADS and linked to a local load record.",
    },
    HandoffStatusDescriptor {
        status: HandoffStatus::PushFailed,
        legacy_label: HandoffStatus::PushFailed.as_legacy_label(),
        description: "The latest publish attempt failed and needs operator attention.",
    },
    HandoffStatusDescriptor {
        status: HandoffStatus::RequeueRequired,
        legacy_label: HandoffStatus::RequeueRequired.as_legacy_label(),
        description: "The handoff must be queued again before it can be republished.",
    },
    HandoffStatusDescriptor {
        status: HandoffStatus::Withdrawn,
        legacy_label: HandoffStatus::Withdrawn.as_legacy_label(),
        description: "The published freight is no longer active on the board.",
    },
    HandoffStatusDescriptor {
        status: HandoffStatus::Closed,
        legacy_label: HandoffStatus::Closed.as_legacy_label(),
        description: "The handoff lifecycle is complete and no further board activity is expected.",
    },
];

pub const TMS_STATUS_DESCRIPTORS: &[TmsStatusDescriptor] = &[
    TmsStatusDescriptor {
        status: TmsStatus::Dispatched,
        legacy_label: TmsStatus::Dispatched.as_legacy_label(),
        description: "Dispatch has been assigned in the TMS.",
    },
    TmsStatusDescriptor {
        status: TmsStatus::InTransit,
        legacy_label: TmsStatus::InTransit.as_legacy_label(),
        description: "The load is moving between pickup and delivery milestones.",
    },
    TmsStatusDescriptor {
        status: TmsStatus::AtPickup,
        legacy_label: TmsStatus::AtPickup.as_legacy_label(),
        description: "The carrier has arrived at pickup in the upstream TMS.",
    },
    TmsStatusDescriptor {
        status: TmsStatus::AtDelivery,
        legacy_label: TmsStatus::AtDelivery.as_legacy_label(),
        description: "The carrier has reached the delivery stop in the upstream TMS.",
    },
    TmsStatusDescriptor {
        status: TmsStatus::Delivered,
        legacy_label: TmsStatus::Delivered.as_legacy_label(),
        description: "Delivery is complete and the handoff can start closing out.",
    },
    TmsStatusDescriptor {
        status: TmsStatus::Cancelled,
        legacy_label: TmsStatus::Cancelled.as_legacy_label(),
        description: "The dispatch was cancelled in the TMS and should be withdrawn locally.",
    },
    TmsStatusDescriptor {
        status: TmsStatus::Invoiced,
        legacy_label: TmsStatus::Invoiced.as_legacy_label(),
        description: "Billing has started in the TMS after delivery.",
    },
    TmsStatusDescriptor {
        status: TmsStatus::Settled,
        legacy_label: TmsStatus::Settled.as_legacy_label(),
        description: "Financial settlement is complete on the TMS side.",
    },
];

pub const TMS_WEBHOOK_SURFACES: &[TmsWebhookSurfaceDescriptor] = &[
    TmsWebhookSurfaceDescriptor {
        method: "POST",
        route: "/api/stloads/webhook/status",
        purpose: "Single status reconciliation webhook from the upstream TMS.",
        payload_keys: &[
            "tms_load_id",
            "tenant_id",
            "tms_status",
            "status_at",
            "source_module",
            "pushed_by",
            "detail",
            "rate_update",
        ],
    },
    TmsWebhookSurfaceDescriptor {
        method: "POST",
        route: "/api/stloads/webhook/bulk-status",
        purpose: "Batch status reconciliation when the TMS ships multiple updates together.",
        payload_keys: &["updates"],
    },
    TmsWebhookSurfaceDescriptor {
        method: "POST",
        route: "/api/stloads/webhook/cancel",
        purpose: "Cancellation hook that auto-withdraws active board postings.",
        payload_keys: &["tms_load_id", "tenant_id", "reason", "pushed_by"],
    },
    TmsWebhookSurfaceDescriptor {
        method: "POST",
        route: "/api/stloads/webhook/close",
        purpose: "Archive hook that closes completed handoffs and reconciliation state.",
        payload_keys: &["tms_load_id", "tenant_id", "reason", "pushed_by"],
    },
];

pub const RECONCILIATION_ACTION_DESCRIPTORS: &[ReconciliationActionDescriptor] = &[
    ReconciliationActionDescriptor {
        action: ReconciliationAction::StatusUpdate,
        legacy_label: ReconciliationAction::StatusUpdate.as_legacy_label(),
        description: "Standard status transition processed from a webhook payload.",
    },
    ReconciliationActionDescriptor {
        action: ReconciliationAction::AutoWithdraw,
        legacy_label: ReconciliationAction::AutoWithdraw.as_legacy_label(),
        description: "Board posting withdrawn automatically after cancellation or mismatch resolution.",
    },
    ReconciliationActionDescriptor {
        action: ReconciliationAction::AutoClose,
        legacy_label: ReconciliationAction::AutoClose.as_legacy_label(),
        description: "Completion action that closes the active handoff lifecycle.",
    },
    ReconciliationActionDescriptor {
        action: ReconciliationAction::AutoArchive,
        legacy_label: ReconciliationAction::AutoArchive.as_legacy_label(),
        description: "Archive flow triggered by explicit close or delivered settlement logic.",
    },
    ReconciliationActionDescriptor {
        action: ReconciliationAction::RateUpdate,
        legacy_label: ReconciliationAction::RateUpdate.as_legacy_label(),
        description: "Upstream board rate change propagated into local records.",
    },
    ReconciliationActionDescriptor {
        action: ReconciliationAction::MismatchDetected,
        legacy_label: ReconciliationAction::MismatchDetected.as_legacy_label(),
        description: "Reconciliation scan detected a divergence between TMS and STLOADS state.",
    },
    ReconciliationActionDescriptor {
        action: ReconciliationAction::ForceSync,
        legacy_label: ReconciliationAction::ForceSync.as_legacy_label(),
        description: "Manual or scheduled operator action used to force records back into sync.",
    },
];

pub const TMS_MODULE_CONTRACT: TmsModuleContract = TmsModuleContract {
    aggregate_tables: &[
        "stloads_handoffs",
        "stloads_handoff_events",
        "stloads_external_refs",
        "stloads_sync_errors",
        "stloads_reconciliation_log",
    ],
    webhook_surfaces: TMS_WEBHOOK_SURFACES,
    reconciliation_actions: RECONCILIATION_ACTION_DESCRIPTORS,
    notes: &[
        "status reconciliation is keyed by tms_load_id plus tenant_id across webhook endpoints",
        "handoffs can update linked local load pricing when rate_update is supplied by the TMS",
        "reconciliation audit currently uses a singular table name: stloads_reconciliation_log",
    ],
};

pub fn handoff_status_descriptors() -> &'static [HandoffStatusDescriptor] {
    HANDOFF_STATUS_DESCRIPTORS
}

pub fn tms_status_descriptors() -> &'static [TmsStatusDescriptor] {
    TMS_STATUS_DESCRIPTORS
}

pub fn tms_webhook_surfaces() -> &'static [TmsWebhookSurfaceDescriptor] {
    TMS_WEBHOOK_SURFACES
}

pub fn reconciliation_action_descriptors() -> &'static [ReconciliationActionDescriptor] {
    RECONCILIATION_ACTION_DESCRIPTORS
}

pub fn tms_module_contract() -> TmsModuleContract {
    TMS_MODULE_CONTRACT.clone()
}
