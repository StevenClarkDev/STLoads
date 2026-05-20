# STLoads Go-To-Market Overview

## Product Summary

STLoads is a professional freight marketplace and load-board platform built as a standalone Rust and Leptos product. It supports shippers, brokers, freight forwarders, carriers, dispatch operators, administrators, and payment operations through one coordinated marketplace workflow.

STLoads is designed to run beside ATMP-OS without becoming an ATMP screen. ATMP Dispatch can create and push loads into STLoads, while STLoads handles marketplace exposure, carrier interaction, booking, documents, payments, execution visibility, and reconciliation.

## What STLoads Does

- publishes freight opportunities to approved marketplace users
- supports role-aware registration, verification, onboarding, and KYC review
- lets operators create, review, update, approve, reject, and manage load records
- manages carrier offers, bookings, execution status, documents, and closeout
- supports Stripe Connect onboarding, payment intent funding, escrow state, and transfer release
- gives administrators user, role, master-data, payment, onboarding, and reconciliation control
- provides dispatch integration endpoints for ATMP-side load handoff
- keeps documents protected by tenant and role rules

## Why It Is Enterprise-Grade

STLoads is enterprise-grade because the core workflows are built around production controls rather than demo screens:

- tenant-aware data paths
- role-based access control
- signed Stripe webhook handling
- audited operational actions
- protected document upload and read flows
- health and readiness endpoints
- IBM Code Engine deployment runbook
- PostgreSQL-backed state
- Rust service reliability and compile-time safety
- Leptos frontend compiled into a controlled app surface

The product is not a static board. It is a governed freight execution environment where users, loads, payments, documents, and operational actions are controlled through defined workflows.

## Integration With ATMP

ATMP remains the dispatch system of record. STLoads remains the public and private marketplace layer.

The integration model is:

- ATMP Dispatch creates or manages operational load intent.
- ATMP pushes eligible loads into STLoads through signed API handoff.
- STLoads exposes loads to the appropriate marketplace audience.
- STLoads captures marketplace-side carrier actions, documents, payment events, and execution signals.
- ATMP can consume STLoads status and reconcile dispatch operations.

This keeps dispatch operations and marketplace operations separate while allowing the two systems to work together.

## Carrier Marketplace Value

For carriers, STLoads gives a direct marketplace surface with controlled onboarding, verified access, searchable load opportunities, booking workflows, document handling, and payment transparency.

Carrier value:

- faster access to available freight
- cleaner booking and execution workflow
- protected document exchange
- payment visibility
- marketplace identity tied to KYC and role approval
- fewer phone-only status loops

## Shipper, Broker, And Operator Value

For shippers, brokers, forwarders, and operators, STLoads creates a governed way to expose freight without losing control of quality, visibility, or payment state.

Operational value:

- better carrier reach
- clearer approval and exception workflows
- searchable account and load control
- reduced duplicate manual entry
- dispatch-to-marketplace connection with ATMP
- safer payment handling through Stripe Connect
- audit-friendly operational history

## Positioning

STLoads should be positioned as a serious freight marketplace product for companies that need more than a simple posting board. The message is operational control, marketplace reach, payment accountability, and enterprise-grade integration with dispatch infrastructure.

## Partner Readiness Message

STLoads is ready to be presented as a production-grade marketplace foundation when the live deployment, support procedures, security posture, and partner operating model are reviewed together. The strongest story is not just the interface. It is the controlled backend, middleware, payment, document, and deployment architecture behind it.

