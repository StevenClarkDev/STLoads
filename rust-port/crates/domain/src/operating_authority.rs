use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum OperatingModelScope {
    InScopeFirstRelease,
    ExplicitlyOutOfScope,
    LegalReviewRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum EvidenceStatus {
    Required,
    NotRequiredForCurrentModel,
    CustomerSpecific,
    LegalReviewRequired,
}

#[derive(Debug, Clone, Serialize)]
pub struct OperatingModelDecision {
    pub key: &'static str,
    pub label: &'static str,
    pub scope: OperatingModelScope,
    pub decision: &'static str,
    pub regulatory_note: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComplianceEvidenceRequirement {
    pub key: &'static str,
    pub label: &'static str,
    pub owner: &'static str,
    pub status: EvidenceStatus,
    pub renewal_required: bool,
    pub customer_disclosable: bool,
    pub evidence_examples: &'static [&'static str],
}

#[derive(Debug, Clone, Serialize)]
pub struct OperatingAuthorityContract {
    pub operating_model: &'static [OperatingModelDecision],
    pub evidence_requirements: &'static [ComplianceEvidenceRequirement],
    pub global_rules: &'static [&'static str],
}

pub const OPERATING_MODEL_DECISIONS: &[OperatingModelDecision] = &[
    OperatingModelDecision {
        key: "software_tms_extension",
        label: "Software-only loadboard and TMS extension",
        scope: OperatingModelScope::InScopeFirstRelease,
        decision: "STLoads provides software workflows, customer-configured loadboard operations, TMS handoffs, documents, audit, and payment orchestration without taking carrier, broker, or freight-forwarder responsibility.",
        regulatory_note: "This model still needs customer contract clarity and legal review, but does not intentionally represent STLoads as the motor carrier, broker, or freight forwarder.",
    },
    OperatingModelDecision {
        key: "carrier_marketplace",
        label: "Carrier marketplace",
        scope: OperatingModelScope::InScopeFirstRelease,
        decision: "STLoads may surface customer/carrier freight opportunities as a software marketplace while the contracting customer remains responsible for its own authority and freight obligations.",
        regulatory_note: "Marketplace copy must avoid implying STLoads arranges freight as broker of record unless broker authority is approved.",
    },
    OperatingModelDecision {
        key: "broker_of_record",
        label: "Broker of record",
        scope: OperatingModelScope::ExplicitlyOutOfScope,
        decision: "STLoads must not act as broker of record in the first enterprise release.",
        regulatory_note: "Official FMCSA guidance says brokers are required to register with FMCSA, and broker registration for property requires financial security such as BMC-84 or BMC-85.",
    },
    OperatingModelDecision {
        key: "freight_forwarder",
        label: "Freight forwarder",
        scope: OperatingModelScope::ExplicitlyOutOfScope,
        decision: "STLoads must not act as freight forwarder in the first enterprise release.",
        regulatory_note: "Official FMCSA guidance says freight forwarders are required to register with FMCSA and freight forwarders assume responsibility for transportation.",
    },
    OperatingModelDecision {
        key: "motor_carrier",
        label: "Motor carrier",
        scope: OperatingModelScope::ExplicitlyOutOfScope,
        decision: "STLoads must not operate commercial motor vehicles or hold itself out as a carrier in the first enterprise release.",
        regulatory_note: "Motor carrier operations require separate safety/commercial registration analysis.",
    },
    OperatingModelDecision {
        key: "payment_facilitator",
        label: "Payment facilitator",
        scope: OperatingModelScope::LegalReviewRequired,
        decision: "STLoads may orchestrate Stripe Connect escrow-like flows only within the approved processor model and must not market itself as a bank, money transmitter, or payment facilitator without finance/legal approval.",
        regulatory_note: "Payment-facilitator, money-transmission, and escrow claims remain out of scope until finance/legal review is complete.",
    },
];

pub const COMPLIANCE_EVIDENCE_REQUIREMENTS: &[ComplianceEvidenceRequirement] = &[
    ComplianceEvidenceRequirement {
        key: "operating_model_memo",
        label: "Operating model legal memo",
        owner: "Legal",
        status: EvidenceStatus::Required,
        renewal_required: false,
        customer_disclosable: true,
        evidence_examples: &[
            "approved operating model memo",
            "customer-facing disclosure text",
        ],
    },
    ComplianceEvidenceRequirement {
        key: "broker_authority",
        label: "FMCSA broker authority",
        owner: "Legal / Operations",
        status: EvidenceStatus::NotRequiredForCurrentModel,
        renewal_required: true,
        customer_disclosable: true,
        evidence_examples: &[
            "FMCSA L&I authority record",
            "BMC-84 surety bond or BMC-85 trust filing",
        ],
    },
    ComplianceEvidenceRequirement {
        key: "freight_forwarder_authority",
        label: "FMCSA freight-forwarder authority",
        owner: "Legal / Operations",
        status: EvidenceStatus::NotRequiredForCurrentModel,
        renewal_required: true,
        customer_disclosable: true,
        evidence_examples: &["FMCSA L&I authority record", "financial security evidence"],
    },
    ComplianceEvidenceRequirement {
        key: "cyber_liability",
        label: "Cyber liability insurance",
        owner: "Finance / Security",
        status: EvidenceStatus::Required,
        renewal_required: true,
        customer_disclosable: true,
        evidence_examples: &["certificate of insurance", "policy declaration page"],
    },
    ComplianceEvidenceRequirement {
        key: "technology_errors_omissions",
        label: "Technology E&O insurance",
        owner: "Finance / Legal",
        status: EvidenceStatus::Required,
        renewal_required: true,
        customer_disclosable: true,
        evidence_examples: &["certificate of insurance", "policy declaration page"],
    },
    ComplianceEvidenceRequirement {
        key: "general_liability",
        label: "Commercial general liability insurance",
        owner: "Finance",
        status: EvidenceStatus::Required,
        renewal_required: true,
        customer_disclosable: true,
        evidence_examples: &["certificate of insurance"],
    },
    ComplianceEvidenceRequirement {
        key: "contingent_cargo",
        label: "Contingent cargo / broker liability insurance",
        owner: "Finance / Legal",
        status: EvidenceStatus::CustomerSpecific,
        renewal_required: true,
        customer_disclosable: true,
        evidence_examples: &["customer-required COI", "coverage endorsement"],
    },
    ComplianceEvidenceRequirement {
        key: "state_province_registration",
        label: "State/province operating registrations",
        owner: "Legal / Operations",
        status: EvidenceStatus::LegalReviewRequired,
        renewal_required: true,
        customer_disclosable: true,
        evidence_examples: &["registration certificate", "jurisdiction memo"],
    },
];

pub const OPERATING_AUTHORITY_GLOBAL_RULES: &[&str] = &[
    "Do not market STLoads as broker, freight forwarder, motor carrier, customs broker, bank, escrow agent, or payment facilitator unless Legal marks that model in scope.",
    "Customer-facing disclosures must say whether STLoads is software-only, marketplace operator, broker of record, freight forwarder, or mixed model for the relevant workflow.",
    "Authority, bond, insurance, and jurisdiction evidence must have an owner, renewal date when applicable, customer-disclosure setting, storage location, and review cadence.",
    "Enterprise customer evidence requests should be fulfilled from stored compliance evidence instead of ad hoc legal work.",
    "Any move into broker-of-record, freight-forwarder, motor-carrier, or payment-facilitator scope requires a new task, legal approval, authority evidence, insurance review, and customer contract update.",
];

pub const OPERATING_AUTHORITY_CONTRACT: OperatingAuthorityContract = OperatingAuthorityContract {
    operating_model: OPERATING_MODEL_DECISIONS,
    evidence_requirements: COMPLIANCE_EVIDENCE_REQUIREMENTS,
    global_rules: OPERATING_AUTHORITY_GLOBAL_RULES,
};

pub fn operating_authority_contract() -> OperatingAuthorityContract {
    OPERATING_AUTHORITY_CONTRACT.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operating_model_explicitly_blocks_regulated_roles_for_first_release() {
        let decisions = operating_authority_contract().operating_model;
        for key in ["broker_of_record", "freight_forwarder", "motor_carrier"] {
            let decision = decisions
                .iter()
                .find(|item| item.key == key)
                .expect("regulated model decision exists");
            assert_eq!(decision.scope, OperatingModelScope::ExplicitlyOutOfScope);
        }
    }

    #[test]
    fn compliance_evidence_has_owners_and_customer_package_rules() {
        for requirement in COMPLIANCE_EVIDENCE_REQUIREMENTS {
            assert!(
                !requirement.owner.is_empty(),
                "{} needs owner",
                requirement.key
            );
            assert!(
                !requirement.evidence_examples.is_empty(),
                "{} needs evidence examples",
                requirement.key
            );
            if requirement.status == EvidenceStatus::Required {
                assert!(
                    requirement.customer_disclosable,
                    "{} should be customer package ready",
                    requirement.key
                );
            }
        }
    }
}
