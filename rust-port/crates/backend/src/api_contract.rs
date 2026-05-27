use serde_json::{Value, json};

pub const API_VERSION: &str = "2026-05-26";

pub fn openapi_spec() -> Value {
    json!({
        "openapi": "3.1.0",
        "info": {
            "title": "STLoads Enterprise API",
            "version": API_VERSION,
            "summary": "Versioned contract for auth, loadboard, execution, payments, TMS, and webhook integrations.",
            "description": "This contract is the first enterprise integration surface. Partner API keys, idempotency, webhooks, EDI, and sandbox governance build on this v1 contract."
        },
        "servers": [
            { "url": "https://api.stloads.com", "description": "Production" },
            { "url": "https://sandbox-api.stloads.com", "description": "Sandbox" },
            { "url": "http://localhost:3001", "description": "Local development" }
        ],
        "tags": [
            { "name": "Auth", "description": "Session, SSO, SCIM, legal agreement, profile, and carrier onboarding APIs." },
            { "name": "Loads", "description": "Load creation, board search, lifecycle, carrier matching, documents, and booking APIs." },
            { "name": "Offers", "description": "Marketplace offer, counteroffer, tender, and chat APIs." },
            { "name": "Tracking", "description": "Execution timeline, route plans, location, telematics, POD, closeout, and customer tracking APIs." },
            { "name": "Documents", "description": "KYC, freight documents, document generation, upload, verification, and closeout package APIs." },
            { "name": "Payments", "description": "Escrow, Stripe Connect, invoices, settlements, accounting export, billing, credit, and payout controls." },
            { "name": "TMS", "description": "STLoads/TMS handoff, retry, reconciliation, status webhook, close, and sync APIs." },
            { "name": "Webhooks", "description": "Inbound webhook surfaces and outbound webhook delivery contracts." }
        ],
        "security": [
            { "SessionBearer": [] },
            { "PartnerApiKey": [] }
        ],
        "components": {
            "securitySchemes": {
                "SessionBearer": {
                    "type": "http",
                    "scheme": "bearer",
                    "bearerFormat": "STLoads session token"
                },
                "PartnerApiKey": {
                    "type": "apiKey",
                    "in": "header",
                    "name": "x-stloads-api-key"
                },
                "WebhookSignature": {
                    "type": "apiKey",
                    "in": "header",
                    "name": "x-stloads-signature"
                }
            },
            "parameters": {
                "RequestId": {
                    "name": "x-request-id",
                    "in": "header",
                    "required": false,
                    "schema": { "type": "string" },
                    "description": "Client supplied request correlation id. Generated when omitted."
                },
                "IdempotencyKey": {
                    "name": "idempotency-key",
                    "in": "header",
                    "required": false,
                    "schema": { "type": "string" },
                    "description": "Required for external write APIs once ENT-1103 is enabled."
                },
                "ApiVersion": {
                    "name": "stloads-api-version",
                    "in": "header",
                    "required": false,
                    "schema": { "type": "string", "default": API_VERSION },
                    "description": "Pins partner requests to a supported API contract version."
                }
            },
            "schemas": {
                "ApiResponse": {
                    "type": "object",
                    "required": ["success", "data"],
                    "properties": {
                        "success": { "type": "boolean" },
                        "data": { "description": "Endpoint specific payload." },
                        "error": { "type": ["string", "null"] }
                    }
                },
                "ErrorResponse": {
                    "type": "object",
                    "required": ["success", "error"],
                    "properties": {
                        "success": { "type": "boolean", "const": false },
                        "error": { "type": "string" },
                        "request_id": { "type": "string" }
                    }
                }
            },
            "responses": {
                "Ok": {
                    "description": "Successful STLoads API response.",
                    "content": {
                        "application/json": {
                            "schema": { "$ref": "#/components/schemas/ApiResponse" }
                        }
                    }
                },
                "Unauthorized": {
                    "description": "Missing, invalid, or insufficient credentials.",
                    "content": {
                        "application/json": {
                            "schema": { "$ref": "#/components/schemas/ErrorResponse" }
                        }
                    }
                }
            }
        },
        "paths": {
            "/health": {
                "get": {
                    "tags": ["Auth"],
                    "summary": "Service health",
                    "operationId": "getHealth",
                    "security": [],
                    "responses": { "200": { "$ref": "#/components/responses/Ok" } }
                }
            },
            "/auth/login": write_path("Auth", "login", "Password login with MFA handoff."),
            "/auth/session": read_path("Auth", "getSession", "Resolve the current authenticated session."),
            "/auth/sso/discovery": write_path("Auth", "discoverEnterpriseSso", "Discover enterprise SSO routing for a domain."),
            "/auth/scim/users": partner_write_path("Auth", "upsertScimUser", "Provision or update a SCIM user."),
            "/dispatch/load-board": read_path("Loads", "searchLoadBoard", "Search the carrier load board."),
            "/dispatch/loads": write_path("Loads", "createLoad", "Create a load with legs."),
            "/dispatch/loads/api-post": partner_write_path("Loads", "postLoadApi", "Partner load posting endpoint."),
            "/dispatch/loads/import/preview": write_path("Loads", "previewBulkLoadImport", "Preview bulk load import rows."),
            "/dispatch/loads/import/commit": write_path("Loads", "commitBulkLoadImport", "Commit validated bulk load import rows."),
            "/dispatch/load-board/{leg_id}/book": write_path("Loads", "bookLoadLeg", "Book a load leg with compliance and idempotency controls."),
            "/dispatch/loads/{load_id}/documents": read_path("Documents", "listLoadDocuments", "List freight documents for a load."),
            "/dispatch/loads/{load_id}/documents/upload": write_path("Documents", "uploadLoadDocument", "Upload a freight document."),
            "/dispatch/loads/{load_id}/documents/generate-standard": write_path("Documents", "generateStandardFreightDocuments", "Generate standard freight documents."),
            "/marketplace/chat-workspace": read_path("Offers", "getChatWorkspace", "Read offer/chat workspace."),
            "/marketplace/offers/{offer_id}/counter": write_path("Offers", "counterOffer", "Create a counteroffer."),
            "/marketplace/offers/{offer_id}/review": write_path("Offers", "reviewOffer", "Accept, reject, or review an offer/tender."),
            "/execution/legs/{leg_id}": read_path("Tracking", "getExecutionLeg", "Read execution timeline and closeout state."),
            "/execution/legs/{leg_id}/actions": write_path("Tracking", "runExecutionAction", "Advance pickup, transit, delivery, POD, and exception actions."),
            "/execution/legs/{leg_id}/location": write_path("Tracking", "storeLegLocation", "Store consented leg location ping."),
            "/execution/legs/{leg_id}/customer-tracking-link": write_path("Tracking", "createCustomerTrackingLink", "Create customer-safe tracking link."),
            "/execution/legs/{leg_id}/closeout": write_path("Documents", "reviewCloseoutPackage", "Review POD and closeout package."),
            "/payments/legs/{leg_id}/fund": write_path("Payments", "fundEscrow", "Fund escrow for a booked leg."),
            "/payments/legs/{leg_id}/hold": write_path("Payments", "holdEscrow", "Place an escrow hold."),
            "/payments/legs/{leg_id}/release": write_path("Payments", "releaseEscrow", "Release escrow after finance, closeout, payout, and compliance gates."),
            "/payments/connect/onboarding-link": write_path("Payments", "createStripeConnectOnboardingLink", "Create carrier Stripe Connect onboarding link."),
            "/payments/accounting/export": read_path("Payments", "exportAccounting", "Export payment ledger and accounting records."),
            "/payments/webhooks/stripe": webhook_path("Webhooks", "receiveStripeWebhook", "Receive signed Stripe webhook events."),
            "/tms/push": write_path("TMS", "pushTmsHandoff", "Push a load handoff to TMS."),
            "/tms/queue": write_path("TMS", "queueTmsHandoff", "Queue a TMS handoff for retry worker processing."),
            "/tms/requeue": write_path("TMS", "requeueTmsHandoff", "Requeue a failed TMS handoff."),
            "/tms/withdraw": write_path("TMS", "withdrawTmsHandoff", "Withdraw a published TMS handoff."),
            "/tms/close": write_path("TMS", "closeTmsHandoff", "Close/archive a TMS handoff."),
            "/tms/webhook/status": webhook_path("Webhooks", "receiveTmsStatusWebhook", "Receive TMS status webhook."),
            "/tms/webhook/bulk-status": webhook_path("Webhooks", "receiveTmsBulkStatusWebhook", "Receive bulk TMS status webhook."),
            "/tms/webhook/cancel": webhook_path("Webhooks", "receiveTmsCancelWebhook", "Receive TMS cancellation webhook."),
            "/tms/webhook/close": webhook_path("Webhooks", "receiveTmsCloseWebhook", "Receive TMS close webhook."),
            "/api/stloads/webhook/status": webhook_path("Webhooks", "receiveLegacyStloadsStatusWebhook", "Compatibility route for STLoads/TMS status webhook."),
            "/api/stloads/webhook/close": webhook_path("Webhooks", "receiveLegacyStloadsCloseWebhook", "Compatibility route for STLoads/TMS close webhook.")
        },
        "x-stloads-lifecycle": {
            "current_version": API_VERSION,
            "compatibility_window_days": 365,
            "sunset_notice_window_days": 180,
            "breaking_change_policy": "Breaking changes require a new version unless emergency security response is approved.",
            "external_write_policy": "All partner writes must support x-request-id and idempotency-key. ENT-1103 stores and enforces them."
        }
    })
}

fn read_path(tag: &str, operation_id: &str, summary: &str) -> Value {
    operation("get", tag, operation_id, summary, false)
}

fn write_path(tag: &str, operation_id: &str, summary: &str) -> Value {
    operation("post", tag, operation_id, summary, false)
}

fn partner_write_path(tag: &str, operation_id: &str, summary: &str) -> Value {
    operation("post", tag, operation_id, summary, true)
}

fn webhook_path(tag: &str, operation_id: &str, summary: &str) -> Value {
    json!({
        "post": {
            "tags": [tag],
            "summary": summary,
            "operationId": operation_id,
            "parameters": [
                { "$ref": "#/components/parameters/RequestId" },
                { "$ref": "#/components/parameters/ApiVersion" }
            ],
            "security": [{ "WebhookSignature": [] }],
            "requestBody": {
                "required": true,
                "content": {
                    "application/json": {
                        "schema": { "type": "object", "additionalProperties": true }
                    }
                }
            },
            "responses": {
                "200": { "$ref": "#/components/responses/Ok" },
                "401": { "$ref": "#/components/responses/Unauthorized" }
            }
        }
    })
}

fn operation(method: &str, tag: &str, operation_id: &str, summary: &str, partner: bool) -> Value {
    let security = if partner {
        json!([{ "PartnerApiKey": [] }])
    } else {
        json!([{ "SessionBearer": [] }])
    };
    let mut parameters = vec![
        json!({ "$ref": "#/components/parameters/RequestId" }),
        json!({ "$ref": "#/components/parameters/ApiVersion" }),
    ];
    if method == "post" {
        parameters.push(json!({ "$ref": "#/components/parameters/IdempotencyKey" }));
    }

    let mut operation = json!({
        "tags": [tag],
        "summary": summary,
        "operationId": operation_id,
        "parameters": parameters,
        "security": security,
        "responses": {
            "200": { "$ref": "#/components/responses/Ok" },
            "401": { "$ref": "#/components/responses/Unauthorized" }
        }
    });

    if method == "post" {
        operation["requestBody"] = json!({
            "required": true,
            "content": {
                "application/json": {
                    "schema": { "type": "object", "additionalProperties": true }
                }
            }
        });
    }

    json!({ method: operation })
}

#[cfg(test)]
mod tests {
    use super::{API_VERSION, openapi_spec};

    #[test]
    fn openapi_contract_covers_enterprise_integration_surfaces() {
        let spec = openapi_spec();
        assert_eq!(spec["openapi"], "3.1.0");
        assert_eq!(spec["info"]["version"], API_VERSION);

        let paths = spec["paths"].as_object().expect("paths should be object");
        for path in [
            "/auth/login",
            "/dispatch/loads",
            "/dispatch/load-board",
            "/dispatch/loads/{load_id}/documents",
            "/marketplace/offers/{offer_id}/counter",
            "/execution/legs/{leg_id}/location",
            "/payments/legs/{leg_id}/release",
            "/payments/webhooks/stripe",
            "/tms/push",
            "/tms/webhook/status",
            "/api/stloads/webhook/status",
        ] {
            assert!(paths.contains_key(path), "missing OpenAPI path {path}");
        }

        assert!(spec["components"]["securitySchemes"]["PartnerApiKey"].is_object());
        assert!(spec["components"]["parameters"]["IdempotencyKey"].is_object());
        assert_eq!(spec["x-stloads-lifecycle"]["current_version"], API_VERSION);
    }
}
