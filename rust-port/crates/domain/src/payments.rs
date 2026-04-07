use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EscrowStatus {
    Unfunded,
    Funded,
    Released,
    Refunded,
    OnHold,
    Failed,
}

impl EscrowStatus {
    pub const fn as_legacy_label(self) -> &'static str {
        match self {
            Self::Unfunded => "unfunded",
            Self::Funded => "funded",
            Self::Released => "released",
            Self::Refunded => "refunded",
            Self::OnHold => "on_hold",
            Self::Failed => "failed",
        }
    }

    pub fn from_legacy_label(label: &str) -> Option<Self> {
        match label {
            "unfunded" => Some(Self::Unfunded),
            "funded" => Some(Self::Funded),
            "released" => Some(Self::Released),
            "refunded" => Some(Self::Refunded),
            "on_hold" => Some(Self::OnHold),
            "failed" => Some(Self::Failed),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StripeWebhookEvent {
    PaymentIntentSucceeded,
    PaymentIntentPaymentFailed,
    AccountUpdated,
}

impl StripeWebhookEvent {
    pub const fn as_legacy_label(self) -> &'static str {
        match self {
            Self::PaymentIntentSucceeded => "payment_intent.succeeded",
            Self::PaymentIntentPaymentFailed => "payment_intent.payment_failed",
            Self::AccountUpdated => "account.updated",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EscrowStatusDescriptor {
    pub status: EscrowStatus,
    pub label: &'static str,
    pub legacy_label: &'static str,
    pub description: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct StripeWebhookEventDescriptor {
    pub event: StripeWebhookEvent,
    pub legacy_label: &'static str,
    pub updates: &'static [&'static str],
    pub notes: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct PaymentsModuleContract {
    pub aggregate_tables: &'static [&'static str],
    pub webhook_events: &'static [StripeWebhookEventDescriptor],
    pub lifecycle_side_effects: &'static [&'static str],
    pub drift_notes: &'static [&'static str],
}

pub const ESCROW_STATUS_DESCRIPTORS: &[EscrowStatusDescriptor] = &[
    EscrowStatusDescriptor {
        status: EscrowStatus::Unfunded,
        label: "Unfunded",
        legacy_label: EscrowStatus::Unfunded.as_legacy_label(),
        description: "Escrow row exists but Stripe funding has not completed yet.",
    },
    EscrowStatusDescriptor {
        status: EscrowStatus::Funded,
        label: "Funded",
        legacy_label: EscrowStatus::Funded.as_legacy_label(),
        description: "PaymentIntent succeeded and the platform holds the funds.",
    },
    EscrowStatusDescriptor {
        status: EscrowStatus::Released,
        label: "Released",
        legacy_label: EscrowStatus::Released.as_legacy_label(),
        description: "Carrier payout transfer has been created from the funded escrow.",
    },
    EscrowStatusDescriptor {
        status: EscrowStatus::Refunded,
        label: "Refunded",
        legacy_label: EscrowStatus::Refunded.as_legacy_label(),
        description: "Escrow has been reversed back to the payer side.",
    },
    EscrowStatusDescriptor {
        status: EscrowStatus::OnHold,
        label: "On Hold",
        legacy_label: EscrowStatus::OnHold.as_legacy_label(),
        description: "Escrow is blocked from release pending an operational review.",
    },
    EscrowStatusDescriptor {
        status: EscrowStatus::Failed,
        label: "Failed",
        legacy_label: EscrowStatus::Failed.as_legacy_label(),
        description: "Funding attempt failed or the related Stripe payment failed.",
    },
];

pub const STRIPE_WEBHOOK_EVENTS: &[StripeWebhookEventDescriptor] = &[
    StripeWebhookEventDescriptor {
        event: StripeWebhookEvent::PaymentIntentSucceeded,
        legacy_label: StripeWebhookEvent::PaymentIntentSucceeded.as_legacy_label(),
        updates: &["escrows.status", "escrows.charge_id"],
        notes: "Marks escrow rows as funded once Stripe confirms the payment intent.",
    },
    StripeWebhookEventDescriptor {
        event: StripeWebhookEvent::PaymentIntentPaymentFailed,
        legacy_label: StripeWebhookEvent::PaymentIntentPaymentFailed.as_legacy_label(),
        updates: &["escrows.status"],
        notes: "Transitions escrows into failed when Stripe rejects the payment.",
    },
    StripeWebhookEventDescriptor {
        event: StripeWebhookEvent::AccountUpdated,
        legacy_label: StripeWebhookEvent::AccountUpdated.as_legacy_label(),
        updates: &["users.payouts_enabled", "users.kyc_status", "users.status"],
        notes: "Keeps carrier payout capability and KYC state aligned with Stripe Connect.",
    },
];

pub const PAYMENTS_MODULE_CONTRACT: PaymentsModuleContract = PaymentsModuleContract {
    aggregate_tables: &["escrows", "users", "load_legs"],
    webhook_events: STRIPE_WEBHOOK_EVENTS,
    lifecycle_side_effects: &[
        "funding a leg writes or reuses an escrow row",
        "funding transitions load_legs.status_id to 8 in legacy Laravel code",
        "releasing funds transitions load_legs.status_id to 11 and stamps completed_at",
        "Stripe Connect onboarding feeds carrier payout readiness back into users",
    ],
    drift_notes: &[
        "legacy escrow migration defines leg_id as UUID while runtime code behaves like integer load_leg ids",
        "escrow lifecycle is represented as string labels today and should remain explicit enums in Rust",
    ],
};

pub fn escrow_status_descriptors() -> &'static [EscrowStatusDescriptor] {
    ESCROW_STATUS_DESCRIPTORS
}

pub fn stripe_webhook_events() -> &'static [StripeWebhookEventDescriptor] {
    STRIPE_WEBHOOK_EVENTS
}

pub fn payments_module_contract() -> PaymentsModuleContract {
    PAYMENTS_MODULE_CONTRACT.clone()
}
