use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i16)]
pub enum UserRole {
    Admin = 1,
    Shipper = 2,
    Carrier = 3,
    Broker = 4,
    FreightForwarder = 5,
}

impl UserRole {
    pub const fn legacy_id(self) -> i16 {
        self as i16
    }

    pub const fn from_legacy_id(id: i16) -> Option<Self> {
        match id {
            1 => Some(Self::Admin),
            2 => Some(Self::Shipper),
            3 => Some(Self::Carrier),
            4 => Some(Self::Broker),
            5 => Some(Self::FreightForwarder),
            _ => None,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Admin => "Admin",
            Self::Shipper => "Shipper",
            Self::Carrier => "Carrier",
            Self::Broker => "Broker",
            Self::FreightForwarder => "Freight Forwarder",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i16)]
pub enum AccountStatus {
    EmailVerifiedPendingOnboarding = 0,
    Approved = 1,
    Rejected = 2,
    PendingReview = 3,
    PendingOtp = 4,
    RevisionRequested = 5,
}

impl AccountStatus {
    pub const fn legacy_code(self) -> i16 {
        self as i16
    }

    pub const fn from_legacy_code(code: i16) -> Option<Self> {
        match code {
            0 => Some(Self::EmailVerifiedPendingOnboarding),
            1 => Some(Self::Approved),
            2 => Some(Self::Rejected),
            3 => Some(Self::PendingReview),
            4 => Some(Self::PendingOtp),
            5 => Some(Self::RevisionRequested),
            _ => None,
        }
    }

    pub const fn requires_admin_review(self) -> bool {
        matches!(self, Self::PendingReview | Self::RevisionRequested)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Permission {
    AccessAdminPortal,
    ManageUsers,
    ManageRoles,
    ManageMasterData,
    ManageLoads,
    ManageDispatchDesk,
    ManageMarketplace,
    ManageTracking,
    ManagePayments,
    ManageTmsOperations,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionTokenKind {
    BrowserSession,
    PasswordReset,
    PersonalAccessToken,
    OneTimePassword,
}

#[derive(Debug, Clone, Serialize)]
pub struct RoleDescriptor {
    pub role: UserRole,
    pub label: &'static str,
    pub legacy_id: i16,
    pub default_dashboard: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct AccountStatusDescriptor {
    pub status: AccountStatus,
    pub label: &'static str,
    pub legacy_code: i16,
    pub description: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct PermissionDescriptor {
    pub permission: Permission,
    pub label: &'static str,
    pub description: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct RolePermissionContract {
    pub role: UserRole,
    pub permissions: &'static [Permission],
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthModuleContract {
    pub browser_sessions_table: &'static str,
    pub password_reset_table: &'static str,
    pub api_tokens_table: &'static str,
    pub otp_storage: &'static str,
    pub otp_ttl_minutes: u16,
    pub max_otp_resends_per_hour: u8,
    pub guards: &'static [&'static str],
    pub session_token_kinds: &'static [SessionTokenKind],
    pub rbac_tables: &'static [&'static str],
    pub legacy_role_source: &'static str,
}

pub const ROLE_DESCRIPTORS: &[RoleDescriptor] = &[
    RoleDescriptor {
        role: UserRole::Admin,
        label: "Admin",
        legacy_id: UserRole::Admin as i16,
        default_dashboard: "/admin",
    },
    RoleDescriptor {
        role: UserRole::Shipper,
        label: "Shipper",
        legacy_id: UserRole::Shipper as i16,
        default_dashboard: "/",
    },
    RoleDescriptor {
        role: UserRole::Carrier,
        label: "Carrier",
        legacy_id: UserRole::Carrier as i16,
        default_dashboard: "/",
    },
    RoleDescriptor {
        role: UserRole::Broker,
        label: "Broker",
        legacy_id: UserRole::Broker as i16,
        default_dashboard: "/",
    },
    RoleDescriptor {
        role: UserRole::FreightForwarder,
        label: "Freight Forwarder",
        legacy_id: UserRole::FreightForwarder as i16,
        default_dashboard: "/",
    },
];

pub const ACCOUNT_STATUS_DESCRIPTORS: &[AccountStatusDescriptor] = &[
    AccountStatusDescriptor {
        status: AccountStatus::EmailVerifiedPendingOnboarding,
        label: "Email Verified",
        legacy_code: AccountStatus::EmailVerifiedPendingOnboarding as i16,
        description: "OTP complete and onboarding form not yet submitted.",
    },
    AccountStatusDescriptor {
        status: AccountStatus::Approved,
        label: "Approved",
        legacy_code: AccountStatus::Approved as i16,
        description: "Admin approved the account for active platform use.",
    },
    AccountStatusDescriptor {
        status: AccountStatus::Rejected,
        label: "Rejected",
        legacy_code: AccountStatus::Rejected as i16,
        description: "Admin rejected the account application.",
    },
    AccountStatusDescriptor {
        status: AccountStatus::PendingReview,
        label: "Pending Review",
        legacy_code: AccountStatus::PendingReview as i16,
        description: "Onboarding submitted and awaiting admin review.",
    },
    AccountStatusDescriptor {
        status: AccountStatus::PendingOtp,
        label: "Pending OTP",
        legacy_code: AccountStatus::PendingOtp as i16,
        description: "Registration created but email OTP verification not completed.",
    },
    AccountStatusDescriptor {
        status: AccountStatus::RevisionRequested,
        label: "Revision Requested",
        legacy_code: AccountStatus::RevisionRequested as i16,
        description: "Admin sent the account back for onboarding changes.",
    },
];

pub const PERMISSION_DESCRIPTORS: &[PermissionDescriptor] = &[
    PermissionDescriptor {
        permission: Permission::AccessAdminPortal,
        label: "Access Admin Portal",
        description: "View admin dashboards and operational pages.",
    },
    PermissionDescriptor {
        permission: Permission::ManageUsers,
        label: "Manage Users",
        description: "Approve, reject, revise, and maintain user accounts.",
    },
    PermissionDescriptor {
        permission: Permission::ManageRoles,
        label: "Manage Roles",
        description: "Maintain role and permission assignments.",
    },
    PermissionDescriptor {
        permission: Permission::ManageMasterData,
        label: "Manage Master Data",
        description: "Maintain countries, cities, equipment, load types, and other lookup data.",
    },
    PermissionDescriptor {
        permission: Permission::ManageLoads,
        label: "Manage Loads",
        description: "Create and manage load postings and related documents.",
    },
    PermissionDescriptor {
        permission: Permission::ManageDispatchDesk,
        label: "Manage Dispatch Desk",
        description: "Use quote, tender, facility, closeout, and collection desk workflows.",
    },
    PermissionDescriptor {
        permission: Permission::ManageMarketplace,
        label: "Manage Marketplace",
        description: "Submit, review, accept, and decline offers and chat activity.",
    },
    PermissionDescriptor {
        permission: Permission::ManageTracking,
        label: "Manage Tracking",
        description: "Update leg tracking and execution milestones.",
    },
    PermissionDescriptor {
        permission: Permission::ManagePayments,
        label: "Manage Payments",
        description: "Use Stripe Connect, escrow funding, release, and payout workflows.",
    },
    PermissionDescriptor {
        permission: Permission::ManageTmsOperations,
        label: "Manage TMS Operations",
        description: "Operate STLOADS handoffs, reconciliation, and sync monitoring.",
    },
];

pub const ADMIN_PERMISSIONS: &[Permission] = &[
    Permission::AccessAdminPortal,
    Permission::ManageUsers,
    Permission::ManageRoles,
    Permission::ManageMasterData,
    Permission::ManageLoads,
    Permission::ManageDispatchDesk,
    Permission::ManageMarketplace,
    Permission::ManageTracking,
    Permission::ManagePayments,
    Permission::ManageTmsOperations,
];

pub const SHIPPER_PERMISSIONS: &[Permission] = &[
    Permission::ManageLoads,
    Permission::ManageMarketplace,
    Permission::ManagePayments,
];

pub const CARRIER_PERMISSIONS: &[Permission] = &[
    Permission::ManageMarketplace,
    Permission::ManageTracking,
    Permission::ManagePayments,
];

pub const BROKER_PERMISSIONS: &[Permission] = &[
    Permission::ManageLoads,
    Permission::ManageMarketplace,
    Permission::ManageDispatchDesk,
    Permission::ManagePayments,
];

pub const FREIGHT_FORWARDER_PERMISSIONS: &[Permission] = &[
    Permission::ManageLoads,
    Permission::ManageMarketplace,
    Permission::ManageDispatchDesk,
    Permission::ManagePayments,
    Permission::ManageTmsOperations,
];

pub const ROLE_PERMISSION_CONTRACTS: &[RolePermissionContract] = &[
    RolePermissionContract {
        role: UserRole::Admin,
        permissions: ADMIN_PERMISSIONS,
    },
    RolePermissionContract {
        role: UserRole::Shipper,
        permissions: SHIPPER_PERMISSIONS,
    },
    RolePermissionContract {
        role: UserRole::Carrier,
        permissions: CARRIER_PERMISSIONS,
    },
    RolePermissionContract {
        role: UserRole::Broker,
        permissions: BROKER_PERMISSIONS,
    },
    RolePermissionContract {
        role: UserRole::FreightForwarder,
        permissions: FREIGHT_FORWARDER_PERMISSIONS,
    },
];

pub const SESSION_TOKEN_KINDS: &[SessionTokenKind] = &[
    SessionTokenKind::BrowserSession,
    SessionTokenKind::PasswordReset,
    SessionTokenKind::PersonalAccessToken,
    SessionTokenKind::OneTimePassword,
];

pub const RBAC_TABLES: &[&str] = &[
    "roles",
    "permissions",
    "model_has_roles",
    "model_has_permissions",
    "role_has_permissions",
];

pub fn role_descriptors() -> &'static [RoleDescriptor] {
    ROLE_DESCRIPTORS
}

pub fn account_status_descriptors() -> &'static [AccountStatusDescriptor] {
    ACCOUNT_STATUS_DESCRIPTORS
}

pub fn permission_descriptors() -> &'static [PermissionDescriptor] {
    PERMISSION_DESCRIPTORS
}

pub fn role_permission_contracts() -> &'static [RolePermissionContract] {
    ROLE_PERMISSION_CONTRACTS
}

pub fn auth_module_contract() -> AuthModuleContract {
    AuthModuleContract {
        browser_sessions_table: "sessions",
        password_reset_table: "password_reset_tokens",
        api_tokens_table: "personal_access_tokens",
        otp_storage: "users.otp + otp_expires_at + resend counters",
        otp_ttl_minutes: 5,
        max_otp_resends_per_hour: 5,
        guards: &["web", "sanctum"],
        session_token_kinds: SESSION_TOKEN_KINDS,
        rbac_tables: RBAC_TABLES,
        legacy_role_source: "users.role_id plus Spatie role pivots",
    }
}
