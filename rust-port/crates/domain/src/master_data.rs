use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MasterDataKind {
    Countries,
    Cities,
    Locations,
    LoadTypes,
    Equipments,
    CommodityTypes,
    LoadStatuses,
    OfferStatuses,
}

#[derive(Debug, Clone, Serialize)]
pub struct MasterDataDescriptor {
    pub kind: MasterDataKind,
    pub label: &'static str,
    pub table: &'static str,
    pub admin_route: &'static str,
    pub used_by: &'static [&'static str],
    pub soft_deletes: bool,
    pub seed_in_rust_baseline: bool,
}

pub const MASTER_DATA_DESCRIPTORS: &[MasterDataDescriptor] = &[
    MasterDataDescriptor {
        kind: MasterDataKind::Countries,
        label: "Countries",
        table: "countries",
        admin_route: "/admin/countries",
        used_by: &[
            "user onboarding",
            "location creation",
            "load builder",
            "TMS ingestion",
        ],
        soft_deletes: false,
        seed_in_rust_baseline: false,
    },
    MasterDataDescriptor {
        kind: MasterDataKind::Cities,
        label: "Cities",
        table: "cities",
        admin_route: "/admin/cities",
        used_by: &["location creation", "load builder", "TMS ingestion"],
        soft_deletes: false,
        seed_in_rust_baseline: false,
    },
    MasterDataDescriptor {
        kind: MasterDataKind::Locations,
        label: "Locations",
        table: "locations",
        admin_route: "/admin/locations",
        used_by: &["load builder", "load profiles", "tracking", "TMS ingestion"],
        soft_deletes: true,
        seed_in_rust_baseline: false,
    },
    MasterDataDescriptor {
        kind: MasterDataKind::LoadTypes,
        label: "Load Types",
        table: "load_types",
        admin_route: "/admin/load-types",
        used_by: &[
            "load builder",
            "carrier preference matching",
            "TMS ingestion",
        ],
        soft_deletes: true,
        seed_in_rust_baseline: false,
    },
    MasterDataDescriptor {
        kind: MasterDataKind::Equipments,
        label: "Equipments",
        table: "equipments",
        admin_route: "/admin/equipments",
        used_by: &[
            "load builder",
            "carrier preference matching",
            "TMS ingestion",
        ],
        soft_deletes: true,
        seed_in_rust_baseline: false,
    },
    MasterDataDescriptor {
        kind: MasterDataKind::CommodityTypes,
        label: "Commodity Types",
        table: "commodity_types",
        admin_route: "/admin/commodity-types",
        used_by: &["load builder", "load detail views"],
        soft_deletes: true,
        seed_in_rust_baseline: false,
    },
    MasterDataDescriptor {
        kind: MasterDataKind::LoadStatuses,
        label: "Load Statuses",
        table: "load_status_master",
        admin_route: "/admin/load-status-master",
        used_by: &[
            "dispatch lifecycle",
            "dashboards",
            "tracking",
            "payments gates",
        ],
        soft_deletes: false,
        seed_in_rust_baseline: true,
    },
    MasterDataDescriptor {
        kind: MasterDataKind::OfferStatuses,
        label: "Offer Statuses",
        table: "offer_status_master",
        admin_route: "/admin/offer-status-master",
        used_by: &["offer creation", "booking", "chat sidepanels"],
        soft_deletes: false,
        seed_in_rust_baseline: true,
    },
];

pub const LOAD_CREATION_MASTER_DATA: &[MasterDataKind] = &[
    MasterDataKind::Countries,
    MasterDataKind::Cities,
    MasterDataKind::Locations,
    MasterDataKind::LoadTypes,
    MasterDataKind::Equipments,
    MasterDataKind::CommodityTypes,
];

pub fn master_data_descriptors() -> &'static [MasterDataDescriptor] {
    MASTER_DATA_DESCRIPTORS
}
