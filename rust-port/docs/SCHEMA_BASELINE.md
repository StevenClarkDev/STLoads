# Schema Baseline

This file records the first SQL baseline added to the Rust port.

It is intentionally conservative: the schema is inferred from Laravel models and write paths, not from a live production schema dump.

## What This Baseline Covers

- account records in `users`
- onboarding details in `user_details`
- KYC uploads in `kyc_documents`
- user approval history in `user_history`
- location and reference data tables
- load and load leg records
- load documents and load history
- carrier preferences
- offers plus offer status master
- load leg status master

## Why It Exists

The current Laravel repo depends on many runtime tables that are not created by the committed PHP migrations.

This baseline gives the Rust workspace a concrete persistence target for:

1. SQLx model definitions
2. future repository code
3. backend route implementation
4. frontend DTO and form work

## Important Limitation

This is not yet the authoritative production schema.

It is also currently expressed in MySQL-flavored SQL because the first Rust persistence pass mirrored the existing Laravel environment.

Before production cutover, we still need to:

- compare this baseline against the live MySQL production database
- translate the validated schema into PostgreSQL DDL for IBM deployment
- port the repository layer away from MySQL-specific functions and placeholders

## Immediate Follow-Up

- validate column names and nullability against the live current production schema
- reconcile `conversations.load_id` versus runtime `load_leg_id`
- validate whether `users.role_id` survives in production or whether roles are fully pivot-driven
- confirm whether `loads.status` is meaningful or whether lifecycle state lives entirely on `load_legs`
- confirm whether `carrier_preferences` should be text, JSON, or normalized join tables
- rewrite the baseline migrations from MySQL syntax into PostgreSQL syntax once the schema is frozen
