Developer Connection Map
This section tells the STLOADS developer exactly where the integration connects inside the existing dispatch architecture.
Primary connection points in the live system
1. API connection point
The dispatch service already exposes the STLOADS integration surface in the main router.
The STLOADS developer should connect to:
/api/stloads/push
/api/stloads/queue
/api/stloads/requeue
These are the formal handoff points where STLOADS is supposed to attach.
2. Backend service connection point
The connection belongs in the main Rust service described in the freight flow architecture.
The STLOADS developer should wire into the dispatch backend layer that already handles:
load validation
state machine enforcement
audit creation
notification creation
board/read-model refresh
That means STLOADS should connect after dispatch has validated and normalized the load, not directly to ad hoc UI fields.
3. Source-of-truth record to connect from
STLOADS should consume a projection built from the canonical internal freight records, primarily:
loads
assignments
compliance alerts
documents
POD state
finance readiness
notifications
audit context
It should not connect to raw browser state and should not treat the UI board itself as the system of record.
4. UI connection points
The STLOADS push action should be wired anywhere the operator already makes a shipment release decision.
Primary UI modules:
/ dashboard
/quote-desk
/tender-desk
/facility-desk
/operations
/closeout-desk
/collections-desk
The most important operational oversight page is:
/operations
That module should be the STLOADS developer’s main exception/queue/sync workspace.
What the STLOADS developer is actually building
The STLOADS developer is not building load creation.
They are building the connector that:
receives a board-ready load from dispatch
publishes or queues that load into STLOADS
returns publish status or failure status
supports requeue and later reconciliation
Exact architectural rule for the STLOADS developer
Connect STLOADS to the dispatch handoff layer, not to intake, not to raw form state, and not to the internal load board as if that board were the source of truth.
Simplified developer instruction
If the STLOADS developer asks “where do I connect this?”, the answer is:
connect to the dispatch API handoff routes
build from the server-side validated load projection
surface status back into operations, dashboard, quote desk, and tender desk
keep the dispatch app as the authoritative shipment lifecycle owner
This canvas translates the existing freight load flow into a clear architecture for how the dispatch system connects to the STLOADS load board.
Source Context
This outline is based on the current freight load flow architecture, where the dispatch application is already freight-first, uses server-side state enforcement, derives the live board from operational records, and includes placeholder STLOADS handoff surfaces for later external integration.
1. Role of STLOADS in the Operating Model
STLOADS is not the system where a load is born.
The dispatch application remains the source-of-truth system for:
account ownership
load intake
freight validation
compliance requirements
assignment readiness
document packet state
proof of delivery
finance readiness
audit history
notifications
STLOADS functions as the external board and handoff lane for capacity exposure, downstream board placement, and external load distribution.
That means the operating rule is:
Create and govern the load inside dispatch first. Push to STLOADS only after the record is valid enough for board exposure or external handoff.
2. Canonical Relationship Between Dispatch and STLOADS
Dispatch owns
canonical load record
operational state machine
compliance evaluation
assignment records
document packet control
POD approval
finance generation
audit and notification truth
STLOADS owns
board-facing representation of eligible freight
downstream load visibility for external placement
queued or published handoff representation
later carrier-facing or market-facing exposure behavior
Architectural rule
STLOADS should consume a sanitized, board-ready projection of the internal load, not raw internal tables and not free-form UI state.
3. Integration Position in the Lifecycle
The load flow should remain:
Account setup
Load intake
Dispatch validation
Dashboard/board visibility inside dispatch
Pricing and tender review
Compliance and assignment readiness
Facility readiness
In-transit and delivery control
Document and packet control
Finance and closeout
Collections and reconciliation
STLOADS handoff where the external lane requires it
This means STLOADS is a connected external lane, not a replacement for the dispatch lifecycle.
4. When a Load Should Be Allowed to Push to STLOADS
A load should only be eligible for STLOADS push when it passes a release gate.
Minimum STLOADS release gate
valid account attached
valid party type
valid freight mode
complete origin and destination data
scheduled pickup present
positive shipment weight
required party-specific metadata complete
required mode-specific metadata complete
no unresolved compliance blocker that forbids market exposure
audit event recorded for release action
Optional stricter board gate
Depending on how STLOADS is meant to behave, the gate can require additional readiness:
quote completed
pricing reviewed
carrier target strategy selected
appointment/dock requirements populated
document packet minimums satisfied
handoff reason selected
5. STLOADS Push States
The connection should use explicit handoff statuses instead of a single vague pushed/not pushed flag.
Recommended STLOADS lifecycle
Not Queued
Queued for STLOADS
Push In Progress
Published to STLOADS
Push Failed
Requeue Required
Withdrawn from STLOADS
Closed on STLOADS
These states should exist as a dedicated integration layer and should not replace the core load lifecycle.
6. Data Contract From Dispatch to STLOADS
Dispatch should generate a board-ready payload for STLOADS.
Core payload groups
A. Load identity
internal load id
tenant/account id
external handoff id
created at
released at
B. Freight classification
party type
freight mode
equipment type
commodity description
weight
piece or pallet data where applicable
temperature data for reefer
container/port data for drayage
securement/tarp data for flatbed
C. Route and timing
pickup city/state/zip
dropoff city/state/zip
scheduled pickup window
scheduled dropoff window
facility instructions
appointment references
D. Commercial projection
customer-facing rate context if allowed
target cost or board rate if modeled
accessorial flags
quote desk status
tender posture
E. Compliance and readiness
compliance pass/fail summary
blocking flags
required document status
readiness summary
F. Handoff metadata
pushed by user
push reason
source module
queue timestamp
retry count
last push result
7. UI Placement Across the Existing Dispatch Modules
STLOADS actions should appear where operators already make operational decisions, not in an isolated hidden integration page only.
Dashboard
Should allow:
see STLOADS eligibility
see current STLOADS queue/publish state
push eligible loads
open handoff detail
Quote Desk
Should allow:
push quote-ready loads to STLOADS
mark loads as internal-only vs board-eligible
show pricing readiness before push
Tender Desk
Should allow:
push tender-stage loads when external capacity exposure is needed
see whether a load has already been board-exposed
prevent duplicate handoff behavior
Facility Desk
Should allow:
confirm readiness before STLOADS exposure when appointments or dock timing matter
Closeout Desk / Collections Desk
Should allow:
see whether the STLOADS representation must be closed, reconciled, or archived downstream
Operations Module
Should act as the main STLOADS oversight surface for:
queued pushes
n- failed pushes
requeue actions
manual review
publication status
withdrawal status
synchronization exceptions
8. Recommended Backend Integration Shape
The backend should treat STLOADS as an integration boundary with its own controlled records.
Recommended server components
stloads_handoff_requests
stloads_handoff_events
stloads_handoff_status
stloads_external_refs
stloads_sync_errors
Recommended server flow
operator clicks Push to STLOADS
server validates release gate
server builds board-ready payload
server records audit event
server inserts queue/handoff row
server emits notification/event stream update
integration worker or API route attempts external push
result is written back to handoff status and audit
9. Audit Requirements
Every STLOADS action must be attributable.
Required audit events
STLOADS push requested
STLOADS push accepted into queue
STLOADS push succeeded
STLOADS push failed
STLOADS requeue requested
STLOADS withdrawn
STLOADS closed/reconciled
Each event should record:
who performed the action
which load was affected
when it occurred
which module initiated it
what payload version was used
what the result was
10. Notifications and Exception Handling
The notification layer should surface STLOADS failures as operational work, not silent integration noise.
Example exception classes
release gate failed
payload validation failed
external publish failed
duplicate publish detected
withdrawn internally but still live externally
delivered internally but still open externally
finance closed internally but STLOADS status not reconciled
These should appear in:
operations
notifications
audit
board severity overlays where appropriate
11. Synchronization Rules
The dispatch application remains authoritative for load governance.
Core synchronization rules
internal canonical lifecycle always wins
STLOADS cannot independently redefine the dispatch source-of-truth lifecycle
compliance-blocked loads should not remain externally exposed without explicit override policy
delivered/completed loads should trigger STLOADS closure workflow
withdrawn loads should unpublish or suppress downstream exposure
requeue should create a new attempt event without deleting prior history
12. Phase-Based Implementation Path
Phase 1 — Stable internal handoff layer
Build:
release gate logic
queue table(s)
status tracking
audit events
operations visibility
UI action wiring in dashboard, quote desk, tender desk, and operations
Outcome:
push-to-STLOADS becomes a real governed internal action even before external API wiring
Phase 2 — External STLOADS API adapter
Build:
outbound payload mapper
authenticated STLOADS API client
response normalization
retry/requeue behavior
external reference persistence
Outcome:
queued handoffs become real publication events
Phase 3 — Reconciliation and board sync
Build:
STLOADS status pullback or webhook handling
publish/withdraw/close reconciliation
exception dashboards
downstream archive sync
Outcome:
dispatch and STLOADS remain operationally aligned over time
13. One-Line Architecture Rule
Dispatch creates, validates, governs, and closes the load; STLOADS receives a controlled board-ready projection only when the shipment has passed the proper release gate.