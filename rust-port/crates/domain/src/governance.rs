use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum StatusVisibility {
    CustomerVisible,
    InternalOnly,
    MixedByStatus,
}

impl StatusVisibility {
    pub const fn as_label(self) -> &'static str {
        match self {
            Self::CustomerVisible => "customer_visible",
            Self::InternalOnly => "internal_only",
            Self::MixedByStatus => "mixed_by_status",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct StatusGovernanceFamily {
    pub key: &'static str,
    pub owner: &'static str,
    pub source_of_truth: &'static str,
    pub data_surface: &'static str,
    pub visibility: StatusVisibility,
    pub customer_visible_statuses: &'static [&'static str],
    pub internal_only_statuses: &'static [&'static str],
    pub change_control: &'static [&'static str],
    pub required_verification: &'static [&'static str],
}

#[derive(Debug, Clone, Serialize)]
pub struct StatusGovernanceContract {
    pub families: &'static [StatusGovernanceFamily],
    pub global_rules: &'static [&'static str],
}

pub const STATUS_CHANGE_CONTROL: &[&str] = &[
    "Open a product/backend change request naming the status family, affected customers, and migration risk.",
    "Update the Rust domain enum or descriptor before changing runtime code.",
    "Add a database migration when persisted values, lookup rows, indexes, constraints, or backfills change.",
    "Update API DTOs, UI copy, admin/support documentation, and audit/export behavior when visibility changes.",
    "Add or update tests for allowed transitions, forbidden transitions, customer visibility, and historical data compatibility.",
    "Record rollout, rollback, and customer communication notes before deployment.",
];

pub const STATUS_GOVERNANCE_FAMILIES: &[StatusGovernanceFamily] = &[
    StatusGovernanceFamily {
        key: "load_leg_status",
        owner: "Product Operations / Backend",
        source_of_truth: "domain::dispatch::LegacyLoadLegStatusCode and LEGACY_LOAD_LEG_STATUS_DESCRIPTORS",
        data_surface: "load_legs.status_id, load_status_master, load_history.status",
        visibility: StatusVisibility::MixedByStatus,
        customer_visible_statuses: &[
            "New",
            "Reviewed",
            "Offer Ready",
            "Booked",
            "Pickup Started",
            "At Pickup",
            "In Transit",
            "Escrow Funded",
            "At Delivery",
            "Delivered",
            "Paid Out",
        ],
        internal_only_statuses: &["Draft"],
        change_control: STATUS_CHANGE_CONTROL,
        required_verification: &[
            "cargo test -p db tenant_scoped_queries_reject_cross_organization_records",
            "cargo test -p db escrow_transition_updates_leg_status_and_history",
            "cargo test -p backend routes::execution::tests::execution_routes_enforce_pod_note_and_document_visibility",
            "trunk build --release",
        ],
    },
    StatusGovernanceFamily {
        key: "offer_status",
        owner: "Marketplace Product / Backend",
        source_of_truth: "domain::marketplace::OfferStatus and OFFER_STATUS_DESCRIPTORS",
        data_surface: "offers.status_id, offer_status_master, conversations/messages realtime",
        visibility: StatusVisibility::CustomerVisible,
        customer_visible_statuses: &["Pending", "Accepted", "Declined"],
        internal_only_statuses: &[],
        change_control: STATUS_CHANGE_CONTROL,
        required_verification: &[
            "cargo test -p db tms_rate_update_marks_requeue_required_and_updates_leg_price",
            "cargo test -p backend",
            "trunk build --release",
        ],
    },
    StatusGovernanceFamily {
        key: "escrow_status",
        owner: "Finance Operations / Backend",
        source_of_truth: "domain::payments::EscrowStatus and ESCROW_STATUS_DESCRIPTORS",
        data_surface: "escrows.status, Stripe PaymentIntent/Transfer metadata, load_history.status",
        visibility: StatusVisibility::MixedByStatus,
        customer_visible_statuses: &["unfunded", "funded", "released", "refunded"],
        internal_only_statuses: &["on_hold", "failed"],
        change_control: STATUS_CHANGE_CONTROL,
        required_verification: &[
            "cargo test -p db escrow_transition_updates_leg_status_and_history",
            "cargo test -p backend routes::payments::tests::verifies_valid_stripe_signature",
            "cargo test -p backend routes::payments::tests::parses_real_stripe_payment_intent_payload",
        ],
    },
    StatusGovernanceFamily {
        key: "tms_handoff_status",
        owner: "Integrations Operations / Backend",
        source_of_truth: "domain::tms::HandoffStatus and HANDOFF_STATUS_DESCRIPTORS",
        data_surface: "stloads_handoffs.status, stloads_handoff_events.event_type",
        visibility: StatusVisibility::InternalOnly,
        customer_visible_statuses: &[],
        internal_only_statuses: &[
            "queued",
            "push_in_progress",
            "published",
            "push_failed",
            "requeue_required",
            "withdrawn",
            "closed",
        ],
        change_control: STATUS_CHANGE_CONTROL,
        required_verification: &[
            "cargo test -p db tms_retry_worker_publishes_queued_handoff",
            "cargo test -p db tms_reconciliation_scan_archives_and_flags_drift",
            "cargo test -p backend",
        ],
    },
    StatusGovernanceFamily {
        key: "tms_external_status",
        owner: "Integrations Product / Backend",
        source_of_truth: "domain::tms::TmsStatus and TMS_STATUS_DESCRIPTORS",
        data_surface: "stloads_handoff_events, stloads_reconciliation_log, upstream TMS webhooks",
        visibility: StatusVisibility::MixedByStatus,
        customer_visible_statuses: &[
            "dispatched",
            "in_transit",
            "at_pickup",
            "at_delivery",
            "delivered",
            "cancelled",
        ],
        internal_only_statuses: &["invoiced", "settled"],
        change_control: STATUS_CHANGE_CONTROL,
        required_verification: &[
            "cargo test -p db tms_cancel_webhook_withdraws_local_projection",
            "cargo test -p db tms_reconciliation_scan_archives_and_flags_drift",
            "cargo test -p backend",
        ],
    },
    StatusGovernanceFamily {
        key: "master_data_reference",
        owner: "Data Stewardship / Product Operations",
        source_of_truth: "domain::master_data::MASTER_DATA_SECTIONS and admin master-data workflows",
        data_surface: "countries, cities, locations, load_types, equipments, commodity_types, load_status_master",
        visibility: StatusVisibility::MixedByStatus,
        customer_visible_statuses: &[
            "countries",
            "cities",
            "locations",
            "load_types",
            "equipments",
            "commodity_types",
        ],
        internal_only_statuses: &["load_status_master"],
        change_control: STATUS_CHANGE_CONTROL,
        required_verification: &[
            "cargo test -p db master_data_crud_covers_location_dependencies",
            "cargo test -p backend routes::master_data::tests::master_data_route_handlers_cover_crud_and_archive_flows",
            "cargo test -p backend routes::master_data::tests::master_data_screen_enforces_access_and_returns_db_sections",
            "trunk build --release",
        ],
    },
];

pub const STATUS_GOVERNANCE_GLOBAL_RULES: &[&str] = &[
    "Persisted status values are contract data and cannot be renamed, reused, or deleted without a migration and compatibility plan.",
    "Customer-visible status copy must be reviewed by Product Operations before release.",
    "Internal-only statuses may appear in admin, support, audit, reconciliation, and export surfaces but not in customer workflow copy unless explicitly approved.",
    "Any status transition that changes money, carrier assignment, document requirements, or tracking behavior must write audit/history evidence.",
    "Every new status family needs an owner, source of truth, visibility decision, migration plan, tests, rollback notes, and customer communication notes.",
];

pub const STATUS_GOVERNANCE_CONTRACT: StatusGovernanceContract = StatusGovernanceContract {
    families: STATUS_GOVERNANCE_FAMILIES,
    global_rules: STATUS_GOVERNANCE_GLOBAL_RULES,
};

pub fn status_governance_contract() -> StatusGovernanceContract {
    STATUS_GOVERNANCE_CONTRACT.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn governance_contract_covers_required_enterprise_status_families() {
        let contract = status_governance_contract();
        let keys = contract
            .families
            .iter()
            .map(|family| family.key)
            .collect::<Vec<_>>();

        for required in [
            "load_leg_status",
            "offer_status",
            "escrow_status",
            "tms_handoff_status",
            "tms_external_status",
            "master_data_reference",
        ] {
            assert!(
                keys.contains(&required),
                "missing governance family {required}"
            );
        }
    }

    #[test]
    fn governance_contract_requires_migration_tests_and_visibility() {
        for family in STATUS_GOVERNANCE_FAMILIES {
            assert!(!family.owner.is_empty());
            assert!(!family.source_of_truth.is_empty());
            assert!(
                family
                    .change_control
                    .iter()
                    .any(|rule| rule.contains("migration")),
                "{} must require migration review",
                family.key
            );
            assert!(
                family
                    .required_verification
                    .iter()
                    .any(|check| check.contains("cargo test")),
                "{} must name at least one test gate",
                family.key
            );
            assert!(
                !family.customer_visible_statuses.is_empty()
                    || !family.internal_only_statuses.is_empty(),
                "{} must make visibility explicit",
                family.key
            );
        }
    }
}
