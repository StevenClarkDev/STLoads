use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TableConfidence {
    MigrationBacked,
    RuntimeReferenced,
    CrossDatabase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum PortPriority {
    Phase0,
    Phase1,
    Phase2,
    Phase3,
    Phase4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct TableInventory {
    pub name: &'static str,
    pub confidence: TableConfidence,
    pub priority: PortPriority,
    pub notes: &'static str,
}

impl TableInventory {
    pub const fn new(
        name: &'static str,
        confidence: TableConfidence,
        priority: PortPriority,
        notes: &'static str,
    ) -> Self {
        Self {
            name,
            confidence,
            priority,
            notes,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct DriftNote {
    pub area: &'static str,
    pub risk: &'static str,
    pub rust_direction: &'static str,
}

impl DriftNote {
    pub const fn new(area: &'static str, risk: &'static str, rust_direction: &'static str) -> Self {
        Self {
            area,
            risk,
            rust_direction,
        }
    }
}

pub const MIGRATION_BACKED_TABLES: &[TableInventory] = &[
    TableInventory::new(
        "users",
        TableConfidence::MigrationBacked,
        PortPriority::Phase1,
        "Primary account records with OTP and onboarding fields.",
    ),
    TableInventory::new(
        "password_reset_tokens",
        TableConfidence::MigrationBacked,
        PortPriority::Phase1,
        "Password reset support for the auth flow.",
    ),
    TableInventory::new(
        "sessions",
        TableConfidence::MigrationBacked,
        PortPriority::Phase1,
        "Browser session state in the Laravel app.",
    ),
    TableInventory::new(
        "cache",
        TableConfidence::MigrationBacked,
        PortPriority::Phase1,
        "Application cache storage.",
    ),
    TableInventory::new(
        "cache_locks",
        TableConfidence::MigrationBacked,
        PortPriority::Phase1,
        "Distributed lock support.",
    ),
    TableInventory::new(
        "jobs",
        TableConfidence::MigrationBacked,
        PortPriority::Phase1,
        "Queued jobs backing email and async work.",
    ),
    TableInventory::new(
        "job_batches",
        TableConfidence::MigrationBacked,
        PortPriority::Phase1,
        "Queue batch tracking.",
    ),
    TableInventory::new(
        "failed_jobs",
        TableConfidence::MigrationBacked,
        PortPriority::Phase1,
        "Failed queue jobs.",
    ),
    TableInventory::new(
        "roles",
        TableConfidence::MigrationBacked,
        PortPriority::Phase1,
        "RBAC roles via Spatie permissions.",
    ),
    TableInventory::new(
        "permissions",
        TableConfidence::MigrationBacked,
        PortPriority::Phase1,
        "RBAC permissions via Spatie permissions.",
    ),
    TableInventory::new(
        "model_has_roles",
        TableConfidence::MigrationBacked,
        PortPriority::Phase1,
        "Role assignments for users and other models.",
    ),
    TableInventory::new(
        "model_has_permissions",
        TableConfidence::MigrationBacked,
        PortPriority::Phase1,
        "Direct permission assignments.",
    ),
    TableInventory::new(
        "role_has_permissions",
        TableConfidence::MigrationBacked,
        PortPriority::Phase1,
        "Role to permission mapping.",
    ),
    TableInventory::new(
        "conversations",
        TableConfidence::MigrationBacked,
        PortPriority::Phase3,
        "Chat thread storage with known column drift in runtime code.",
    ),
    TableInventory::new(
        "messages",
        TableConfidence::MigrationBacked,
        PortPriority::Phase3,
        "Chat message records.",
    ),
    TableInventory::new(
        "sequences",
        TableConfidence::MigrationBacked,
        PortPriority::Phase2,
        "Monotonic sequence generation for identifiers.",
    ),
    TableInventory::new(
        "escrows",
        TableConfidence::MigrationBacked,
        PortPriority::Phase4,
        "Escrow funding and payout records.",
    ),
    TableInventory::new(
        "personal_access_tokens",
        TableConfidence::MigrationBacked,
        PortPriority::Phase1,
        "Sanctum token records for API access.",
    ),
    TableInventory::new(
        "stloads_handoffs",
        TableConfidence::MigrationBacked,
        PortPriority::Phase4,
        "Published TMS handoff records.",
    ),
    TableInventory::new(
        "stloads_handoff_events",
        TableConfidence::MigrationBacked,
        PortPriority::Phase4,
        "Handoff event timeline entries.",
    ),
    TableInventory::new(
        "stloads_external_refs",
        TableConfidence::MigrationBacked,
        PortPriority::Phase4,
        "External identifiers associated with handoffs.",
    ),
    TableInventory::new(
        "stloads_sync_errors",
        TableConfidence::MigrationBacked,
        PortPriority::Phase4,
        "Reconciliation and sync error queue.",
    ),
    TableInventory::new(
        "stloads_reconciliation_log",
        TableConfidence::MigrationBacked,
        PortPriority::Phase4,
        "Background reconciliation audit trail.",
    ),
];

pub const INFERRED_RUNTIME_TABLES: &[TableInventory] = &[
    TableInventory::new(
        "loads",
        TableConfidence::RuntimeReferenced,
        PortPriority::Phase0,
        "Top-level shipment records referenced heavily across controllers.",
    ),
    TableInventory::new(
        "load_legs",
        TableConfidence::RuntimeReferenced,
        PortPriority::Phase0,
        "Execution unit for booking, status transitions, and tracking.",
    ),
    TableInventory::new(
        "load_status_master",
        TableConfidence::RuntimeReferenced,
        PortPriority::Phase0,
        "Legacy status lookup table for overloaded leg status codes.",
    ),
    TableInventory::new(
        "offers",
        TableConfidence::RuntimeReferenced,
        PortPriority::Phase0,
        "Carrier offers against load legs.",
    ),
    TableInventory::new(
        "offer_status_master",
        TableConfidence::RuntimeReferenced,
        PortPriority::Phase0,
        "Lookup table for offer statuses.",
    ),
    TableInventory::new(
        "locations",
        TableConfidence::RuntimeReferenced,
        PortPriority::Phase0,
        "Pickup and delivery locations for load legs.",
    ),
    TableInventory::new(
        "countries",
        TableConfidence::RuntimeReferenced,
        PortPriority::Phase1,
        "Master data for countries.",
    ),
    TableInventory::new(
        "cities",
        TableConfidence::RuntimeReferenced,
        PortPriority::Phase1,
        "Master data for cities.",
    ),
    TableInventory::new(
        "load_types",
        TableConfidence::RuntimeReferenced,
        PortPriority::Phase1,
        "Reference data for shipment type selection.",
    ),
    TableInventory::new(
        "equipments",
        TableConfidence::RuntimeReferenced,
        PortPriority::Phase1,
        "Reference data for trailer and equipment selection.",
    ),
    TableInventory::new(
        "commodity_types",
        TableConfidence::RuntimeReferenced,
        PortPriority::Phase1,
        "Reference data for commodity classification.",
    ),
    TableInventory::new(
        "load_documents",
        TableConfidence::RuntimeReferenced,
        PortPriority::Phase2,
        "Documents stored against the parent load.",
    ),
    TableInventory::new(
        "leg_documents",
        TableConfidence::RuntimeReferenced,
        PortPriority::Phase3,
        "Execution-stage documents stored against a leg.",
    ),
    TableInventory::new(
        "leg_events",
        TableConfidence::RuntimeReferenced,
        PortPriority::Phase3,
        "Pickup and delivery event timeline records.",
    ),
    TableInventory::new(
        "leg_locations",
        TableConfidence::RuntimeReferenced,
        PortPriority::Phase3,
        "Tracking points for load legs.",
    ),
    TableInventory::new(
        "load_history",
        TableConfidence::RuntimeReferenced,
        PortPriority::Phase2,
        "Administrative history records for loads.",
    ),
    TableInventory::new(
        "user_history",
        TableConfidence::RuntimeReferenced,
        PortPriority::Phase1,
        "Admin approval and rejection history for users.",
    ),
    TableInventory::new(
        "user_details",
        TableConfidence::RuntimeReferenced,
        PortPriority::Phase1,
        "Onboarding details reached through the missing user.details relation.",
    ),
    TableInventory::new(
        "kyc_documents",
        TableConfidence::RuntimeReferenced,
        PortPriority::Phase1,
        "Uploaded onboarding and compliance documents.",
    ),
    TableInventory::new(
        "shipper_detail",
        TableConfidence::RuntimeReferenced,
        PortPriority::Phase1,
        "Additional shipper-specific profile data.",
    ),
    TableInventory::new(
        "carrier_preferences",
        TableConfidence::RuntimeReferenced,
        PortPriority::Phase2,
        "Carrier matching and routing preference data.",
    ),
];

pub const CROSS_DATABASE_TABLES: &[TableInventory] = &[TableInventory::new(
    "logs",
    TableConfidence::CrossDatabase,
    PortPriority::Phase0,
    "Application log records stored through the second_db connection.",
)];

pub const DRIFT_NOTES: &[DriftNote] = &[
    DriftNote::new(
        "conversation schema",
        "Migration uses load_id while runtime code expects load_leg_id.",
        "Create an explicit Rust chat thread model keyed to leg-level context and add import mapping.",
    ),
    DriftNote::new(
        "user onboarding relation",
        "AuthController relies on user.details but User.php does not define the relation.",
        "Model profile details as a first-class aggregate and recover the live schema before porting writes.",
    ),
    DriftNote::new(
        "load leg lifecycle",
        "Legacy status_id mixes review, booking, funding, and execution concepts in one integer column.",
        "Split the Rust port into explicit posting, execution, and finance lifecycle enums.",
    ),
    DriftNote::new(
        "secondary logging database",
        "Logs are written through a second_db connection that is not represented in the Rust scaffold yet.",
        "Treat audit logging as a separate persistence concern from the primary application database.",
    ),
];

pub const fn tracked_table_count() -> usize {
    MIGRATION_BACKED_TABLES.len() + INFERRED_RUNTIME_TABLES.len() + CROSS_DATABASE_TABLES.len()
}
