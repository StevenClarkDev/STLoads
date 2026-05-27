# Operating Authority, Insurance, And Jurisdiction Requirements

This document supports `ENT-0306` and should be reviewed by Legal before enterprise launch.

## First-Release Operating Model

STLoads first enterprise release is scoped as:

- Software-only loadboard and TMS extension.
- Customer/carrier marketplace software where the contracting customer remains responsible for its own freight authority.
- Stripe Connect payment orchestration within the approved processor model.

STLoads must not market itself as:

- Broker of record.
- Freight forwarder.
- Motor carrier.
- Customs broker.
- Bank, escrow agent, money transmitter, or payment facilitator.

Those regulated roles remain out of scope unless Legal/Finance opens a new task with authority,
insurance, customer-contract, and operating-procedure evidence.

## Regulatory Source Notes

Official FMCSA guidance says brokers and freight forwarders are required to register with FMCSA.
FMCSA also describes broker property registration financial security through Form BMC-84 or BMC-85.
These facts are reflected in `domain::operating_authority::OPERATING_AUTHORITY_CONTRACT`, but Legal
must verify final requirements before any regulated role is launched.

Sources:

- FMCSA broker registration guidance: https://www.fmcsa.dot.gov/registration/broker-registration
- FMCSA freight-forwarder registration guidance: https://www.fmcsa.dot.gov/registration/freight-forwarder-registration
- FMCSA financial responsibility forms: https://www.fmcsa.dot.gov/registration/insurance-filing-requirements

## Evidence Tracked

Evidence is stored in `compliance_evidence_records` with owner, status, evidence type, issuer,
policy or authority number, jurisdiction, document URI, customer-disclosure flag, renewal flag,
effective date, expiration date, review due date, and notes.

Customer-disclosable evidence includes:

- Operating model legal memo.
- Broker authority status or out-of-scope memo.
- Freight-forwarder authority status or out-of-scope memo.
- Cyber liability certificate.
- Technology E&O certificate.
- Commercial general liability certificate.
- Customer-specific contingent cargo or broker liability coverage when contractually required.
- State/province registration evidence when regulated expansion requires it.

## Renewal And Review Rules

- Required insurance certificates must have owner and expiration or review dates.
- Customer-specific certificate requirements must be attached to the customer/account package.
- Authority records must be reviewed before any regulated operating model change.
- Evidence due within 30 days should be escalated to Finance/Legal before enterprise customer renewal packets are promised.

## Customer Disclosure Process

1. Confirm the customer contract requires authority, bond, insurance, or jurisdiction evidence.
2. Pull customer-disclosable records from `compliance_evidence_records`.
3. Confirm evidence is current and not within renewal escalation window.
4. Include the operating model disclosure so customers understand whether STLoads is software-only or acting under a regulated authority.
5. Escalate to Legal for any request involving broker of record, freight forwarding, payment-facilitator, escrow-agent, customs, or cross-border regulatory claims.
