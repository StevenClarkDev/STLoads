#!/usr/bin/env bash

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$REPO_ROOT"

BASE_URL="${RUST_API_BASE_URL:-http://127.0.0.1:3001}"
ADMIN_EMAIL="${RUST_ADMIN_EMAIL:-admin.smoke@stloads.test}"
ADMIN_PASSWORD="${RUST_ADMIN_PASSWORD:-AdminPass123!}"
SHIPPER_EMAIL="${RUST_SHIPPER_EMAIL:-shipper.smoke@stloads.test}"
SHIPPER_PASSWORD="${RUST_SHIPPER_PASSWORD:-ShipperPass123!}"
CARRIER_EMAIL="${RUST_CARRIER_EMAIL:-carrier.smoke@stloads.test}"
CARRIER_PASSWORD="${RUST_CARRIER_PASSWORD:-CarrierPass123!}"
BOOKING_LEG_ID="${RUST_BOOKING_LEG_ID:-9311}"
OFFER_CONVERSATION_ID="${RUST_OFFER_CONVERSATION_ID:-9401}"
OFFER_ID="${RUST_OFFER_ID:-9501}"
SEEDED_HANDOFF_ID="${RUST_SEEDED_HANDOFF_ID:-9601}"
SEEDED_SYNC_ERROR_ID="${RUST_SEEDED_SYNC_ERROR_ID:-9701}"
EXECUTION_PING_LAT="${RUST_EXECUTION_PING_LAT:-40.68920}"
EXECUTION_PING_LNG="${RUST_EXECUTION_PING_LNG:-74.17450}"
STRIPE_WEBHOOK_SECRET="${RUST_STRIPE_WEBHOOK_SECRET:-}"
TMS_SHARED_SECRET="${RUST_TMS_SHARED_SECRET:-}"
TMS_TENANT_ID="${RUST_TMS_TENANT_ID:-demo-tenant}"
CURL_CONNECT_TIMEOUT="${RUST_API_CONNECT_TIMEOUT:-20}"
CURL_MAX_TIME="${RUST_API_MAX_TIME:-120}"

for bin in curl php grep sed mktemp date; do
  if ! command -v "$bin" >/dev/null 2>&1; then
    echo "Missing required dependency: $bin" >&2
    exit 1
  fi
done

TMP_DIR="$(mktemp -d)"
RUN_ID="$(date +%Y%m%d%H%M%S)-$$"
TS_ID="$(date +%Y%m%d%H%M%S)"

cleanup() {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

declare -a RESULT_NAMES=()
declare -a RESULT_METHODS=()
declare -a RESULT_PATHS=()
declare -a RESULT_CODES=()
declare -a RESULT_STATES=()
declare -a RESULT_SUMMARIES=()

TOTAL_RUN=0
TOTAL_SUCCESS=0
TOTAL_FAILED=0
LAST_HTTP_CODE=""
LAST_BODY_FILE=""
LAST_HEADERS_FILE=""

ADMIN_TOKEN=""
SHIPPER_TOKEN=""
CARRIER_TOKEN=""

trim() {
  printf '%s' "$1" | tr -d '\r'
}

json_eval() {
  local file="$1"
  local code="$2"
  php -r '$d=json_decode(file_get_contents($argv[1]), true); if (json_last_error() !== JSON_ERROR_NONE) { fwrite(STDERR, json_last_error_msg()); exit(3); } '"$code" "$file"
}

json_string() {
  local file="$1"
  local code="$2"
  local output
  output="$(json_eval "$file" "$code" 2>/dev/null || true)"
  trim "$output"
}

http_in() {
  local code="$1"
  shift
  local candidate
  for candidate in "$@"; do
    if [[ "$code" == "$candidate" ]]; then
      return 0
    fi
  done
  return 1
}

record_result() {
  local name="$1"
  local method="$2"
  local path="$3"
  local code="$4"
  local state="$5"
  local summary="$6"

  RESULT_NAMES+=("$name")
  RESULT_METHODS+=("$method")
  RESULT_PATHS+=("$path")
  RESULT_CODES+=("$code")
  RESULT_STATES+=("$state")
  RESULT_SUMMARIES+=("$summary")

  TOTAL_RUN=$((TOTAL_RUN + 1))
  if [[ "$state" == "SUCCESS" ]]; then
    TOTAL_SUCCESS=$((TOTAL_SUCCESS + 1))
  else
    TOTAL_FAILED=$((TOTAL_FAILED + 1))
  fi
}

request_json() {
  local method="$1"
  local path="$2"
  local payload="${3:-}"
  local bearer="${4:-}"
  shift 4 || true
  local extra_headers=("$@")

  LAST_BODY_FILE="$TMP_DIR/body-${RANDOM}.txt"
  LAST_HEADERS_FILE="$TMP_DIR/headers-${RANDOM}.txt"

  local curl_args=(
    --silent
    --show-error
    --connect-timeout "$CURL_CONNECT_TIMEOUT"
    --max-time "$CURL_MAX_TIME"
    --request "$method"
    --dump-header "$LAST_HEADERS_FILE"
    --output "$LAST_BODY_FILE"
    --write-out "%{http_code}"
    --header "Accept: application/json"
    --header "Content-Type: application/json"
    "$BASE_URL$path"
  )

  if [[ -n "$bearer" ]]; then
    curl_args+=(--header "Authorization: Bearer $bearer")
  fi

  local header
  for header in "${extra_headers[@]}"; do
    [[ -n "$header" ]] && curl_args+=(--header "$header")
  done

  if [[ -n "$payload" ]]; then
    curl_args+=(--data "$payload")
  fi

  LAST_HTTP_CODE="$(curl "${curl_args[@]}" 2>"$TMP_DIR/curl.err")" || LAST_HTTP_CODE="000"
  LAST_HTTP_CODE="$(trim "$LAST_HTTP_CODE")"
}

request_multipart() {
  local method="$1"
  local path="$2"
  local bearer="$3"
  shift 3 || true
  local form_parts=("$@")

  LAST_BODY_FILE="$TMP_DIR/body-${RANDOM}.txt"
  LAST_HEADERS_FILE="$TMP_DIR/headers-${RANDOM}.txt"

  local curl_args=(
    --silent
    --show-error
    --connect-timeout "$CURL_CONNECT_TIMEOUT"
    --max-time "$CURL_MAX_TIME"
    --request "$method"
    --dump-header "$LAST_HEADERS_FILE"
    --output "$LAST_BODY_FILE"
    --write-out "%{http_code}"
    --header "Accept: application/json"
    --header "Authorization: Bearer $bearer"
    "$BASE_URL$path"
  )

  local part
  for part in "${form_parts[@]}"; do
    curl_args+=(-F "$part")
  done

  LAST_HTTP_CODE="$(curl "${curl_args[@]}" 2>"$TMP_DIR/curl.err")" || LAST_HTTP_CODE="000"
  LAST_HTTP_CODE="$(trim "$LAST_HTTP_CODE")"
}

envelope_ok() {
  [[ "$(json_string "$LAST_BODY_FILE" 'echo $d["status"] ?? "";')" == "ok" ]]
}

envelope_data_success() {
  [[ "$(json_string "$LAST_BODY_FILE" 'echo array_key_exists("success", $d["data"] ?? []) ? ($d["data"]["success"] ? "true" : "false") : "";')" == "true" ]]
}

route_count() {
  php -r '$files=glob($argv[1]); $count=0; foreach ($files as $file) { $count += substr_count(file_get_contents($file), ".route("); } echo $count;' "rust-port/crates/backend/src/routes/*.rs" 2>/dev/null || echo 0
}

login_user() {
  local label="$1"
  local email="$2"
  local password="$3"
  request_json "POST" "/auth/login" "{\"email\":\"${email}\",\"password\":\"${password}\"}" ""
  if ! http_in "$LAST_HTTP_CODE" 200 || ! envelope_ok || ! envelope_data_success; then
    record_result "Login ${label}" "POST" "/auth/login" "$LAST_HTTP_CODE" "FAILED" "Login failed for ${email}."
    return 1
  fi

  local token
  token="$(json_string "$LAST_BODY_FILE" 'echo $d["data"]["token"] ?? "";')"
  local message
  message="$(json_string "$LAST_BODY_FILE" 'echo $d["data"]["message"] ?? "";')"
  record_result "Login ${label}" "POST" "/auth/login" "$LAST_HTTP_CODE" "SUCCESS" "${message:-Logged in ${email}.}"
  printf '%s' "$token"
}

run_action() {
  local action_key="$1"
  local expected_status="$2"
  request_json "POST" "/execution/legs/${BOOKING_LEG_ID}/actions" "{\"action_key\":\"${action_key}\",\"note\":\"Bash chain ${action_key} step.\"}" "$CARRIER_TOKEN"
  if http_in "$LAST_HTTP_CODE" 200 && envelope_ok && envelope_data_success; then
    local status_label
    status_label="$(json_string "$LAST_BODY_FILE" 'echo $d["data"]["status_label"] ?? "";')"
    local summary
    summary="$(json_string "$LAST_BODY_FILE" 'echo $d["data"]["message"] ?? "";')"
    if [[ "$status_label" == "$expected_status" ]]; then
      record_result "Execution ${action_key}" "POST" "/execution/legs/${BOOKING_LEG_ID}/actions" "$LAST_HTTP_CODE" "SUCCESS" "${summary:-Status moved to ${status_label}.}"
    else
      record_result "Execution ${action_key}" "POST" "/execution/legs/${BOOKING_LEG_ID}/actions" "$LAST_HTTP_CODE" "FAILED" "Unexpected status ${status_label}, expected ${expected_status}."
    fi
  else
    record_result "Execution ${action_key}" "POST" "/execution/legs/${BOOKING_LEG_ID}/actions" "$LAST_HTTP_CODE" "FAILED" "Execution action ${action_key} failed."
  fi
}

echo "Rust API chain smoke"
echo "Base URL: $BASE_URL"
echo "Run ID:   $RUN_ID"
echo
echo "Discovered Rust .route(...) definitions: $(route_count)"
echo

# 1. Health
request_json "GET" "/health" "" ""
if http_in "$LAST_HTTP_CODE" 200; then
  summary="$(json_string "$LAST_BODY_FILE" 'echo "Health ok: deployment_target=" . ($d["deployment_target"] ?? "unknown") . ", environment=" . ($d["environment"] ?? "unknown") . ", database_state=" . ($d["database_state"] ?? "unknown");')"
  record_result "Health" "GET" "/health" "$LAST_HTTP_CODE" "SUCCESS" "$summary"
else
  record_result "Health" "GET" "/health" "$LAST_HTTP_CODE" "FAILED" "Health endpoint did not return HTTP 200."
fi

# 2-4. Logins
ADMIN_TOKEN="$(login_user "admin" "$ADMIN_EMAIL" "$ADMIN_PASSWORD")" || ADMIN_TOKEN=""
SHIPPER_TOKEN="$(login_user "shipper" "$SHIPPER_EMAIL" "$SHIPPER_PASSWORD")" || SHIPPER_TOKEN=""
CARRIER_TOKEN="$(login_user "carrier" "$CARRIER_EMAIL" "$CARRIER_PASSWORD")" || CARRIER_TOKEN=""

if [[ -z "$ADMIN_TOKEN" || -z "$SHIPPER_TOKEN" || -z "$CARRIER_TOKEN" ]]; then
  echo
  echo "Final summary"
  echo "  APIs ran:   $TOTAL_RUN"
  echo "  Successful: $TOTAL_SUCCESS"
  echo "  Failed:     $TOTAL_FAILED"
  exit 1
fi

# 5-7. Sessions
for role in admin shipper carrier; do
  token_var="$(printf '%s_TOKEN' "$(printf '%s' "$role" | tr '[:lower:]' '[:upper:]')")"
  token="${!token_var}"
  request_json "GET" "/auth/session" "" "$token"
  if http_in "$LAST_HTTP_CODE" 200 && envelope_ok; then
    actual_role="$(json_string "$LAST_BODY_FILE" 'echo $d["data"]["user"]["role_key"] ?? "";')"
    actual_status="$(json_string "$LAST_BODY_FILE" 'echo $d["data"]["user"]["account_status_label"] ?? "";')"
    if [[ "$actual_role" == "$role" ]]; then
      record_result "Session ${role}" "GET" "/auth/session" "$LAST_HTTP_CODE" "SUCCESS" "Session resolved role=${actual_role}, status=${actual_status}."
    else
      record_result "Session ${role}" "GET" "/auth/session" "$LAST_HTTP_CODE" "FAILED" "Session resolved role=${actual_role}, expected ${role}."
    fi
  else
    record_result "Session ${role}" "GET" "/auth/session" "$LAST_HTTP_CODE" "FAILED" "Session lookup failed for ${role}."
  fi
done

# 8-9. Load boards
request_json "GET" "/dispatch/load-board?tab=all" "" "$SHIPPER_TOKEN"
if http_in "$LAST_HTTP_CODE" 200 && envelope_ok; then
  rows="$(json_string "$LAST_BODY_FILE" 'echo count($d["data"]["rows"] ?? []);')"
  record_result "Shipper Load Board" "GET" "/dispatch/load-board?tab=all" "$LAST_HTTP_CODE" "SUCCESS" "Load board fetched of ${rows} row(s) for shipper."
else
  record_result "Shipper Load Board" "GET" "/dispatch/load-board?tab=all" "$LAST_HTTP_CODE" "FAILED" "Shipper load board could not be loaded."
fi

request_json "GET" "/dispatch/load-board?tab=all" "" "$CARRIER_TOKEN"
if http_in "$LAST_HTTP_CODE" 200 && envelope_ok; then
  rows="$(json_string "$LAST_BODY_FILE" 'echo count($d["data"]["rows"] ?? []);')"
  record_result "Carrier Load Board" "GET" "/dispatch/load-board?tab=all" "$LAST_HTTP_CODE" "SUCCESS" "Load board fetched of ${rows} row(s) for carrier."
else
  record_result "Carrier Load Board" "GET" "/dispatch/load-board?tab=all" "$LAST_HTTP_CODE" "FAILED" "Carrier load board could not be loaded."
fi

# 10. Book leg
request_json "POST" "/dispatch/load-board/${BOOKING_LEG_ID}/book" '{"booked_amount":2850.00}' "$CARRIER_TOKEN"
if http_in "$LAST_HTTP_CODE" 200 && envelope_ok && envelope_data_success; then
  summary="$(json_string "$LAST_BODY_FILE" 'echo $d["data"]["message"] ?? "Leg booked.";')"
  record_result "Book Leg" "POST" "/dispatch/load-board/${BOOKING_LEG_ID}/book" "$LAST_HTTP_CODE" "SUCCESS" "$summary"
else
  record_result "Book Leg" "POST" "/dispatch/load-board/${BOOKING_LEG_ID}/book" "$LAST_HTTP_CODE" "FAILED" "Booking the seeded leg failed."
fi

# 11. Execution screen
request_json "GET" "/execution/legs/${BOOKING_LEG_ID}" "" "$CARRIER_TOKEN"
if http_in "$LAST_HTTP_CODE" 200 && envelope_ok; then
  leg_code="$(json_string "$LAST_BODY_FILE" 'echo $d["data"]["leg_code"] ?? "";')"
  status_label="$(json_string "$LAST_BODY_FILE" 'echo $d["data"]["status_label"] ?? "";')"
  record_result "Execution Screen" "GET" "/execution/legs/${BOOKING_LEG_ID}" "$LAST_HTTP_CODE" "SUCCESS" "Execution screen loaded for ${leg_code} with status ${status_label}."
else
  record_result "Execution Screen" "GET" "/execution/legs/${BOOKING_LEG_ID}" "$LAST_HTTP_CODE" "FAILED" "Execution screen could not be loaded."
fi

# 12-18. Execution lifecycle
run_action "start_pickup" "Pickup Started"

request_json "POST" "/execution/legs/${BOOKING_LEG_ID}/location" "{\"lat\":${EXECUTION_PING_LAT},\"lng\":${EXECUTION_PING_LNG},\"recorded_at\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}" "$CARRIER_TOKEN"
if http_in "$LAST_HTTP_CODE" 200 && envelope_ok && envelope_data_success; then
  summary="$(json_string "$LAST_BODY_FILE" 'echo $d["data"]["message"] ?? "Location ping stored.";')"
  record_result "Execution Location 1" "POST" "/execution/legs/${BOOKING_LEG_ID}/location" "$LAST_HTTP_CODE" "SUCCESS" "$summary"
else
  record_result "Execution Location 1" "POST" "/execution/legs/${BOOKING_LEG_ID}/location" "$LAST_HTTP_CODE" "FAILED" "First execution location ping failed."
fi

run_action "arrive_pickup" "At Pickup"
run_action "depart_pickup" "In Transit"

SECOND_LAT="$(php -r "echo ((float)'$EXECUTION_PING_LAT') + 0.015;")"
SECOND_LNG="$(php -r "echo ((float)'$EXECUTION_PING_LNG') + 0.015;")"
request_json "POST" "/execution/legs/${BOOKING_LEG_ID}/location" "{\"lat\":${SECOND_LAT},\"lng\":${SECOND_LNG},\"recorded_at\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}" "$CARRIER_TOKEN"
if http_in "$LAST_HTTP_CODE" 200 && envelope_ok && envelope_data_success; then
  summary="$(json_string "$LAST_BODY_FILE" 'echo $d["data"]["message"] ?? "Location ping stored.";')"
  record_result "Execution Location 2" "POST" "/execution/legs/${BOOKING_LEG_ID}/location" "$LAST_HTTP_CODE" "SUCCESS" "$summary"
else
  record_result "Execution Location 2" "POST" "/execution/legs/${BOOKING_LEG_ID}/location" "$LAST_HTTP_CODE" "FAILED" "Second execution location ping failed."
fi

run_action "arrive_delivery" "At Delivery"

POD_FILE="$TMP_DIR/smoke-delivery-pod.txt"
printf 'Smoke POD generated by rust_api_chain_smoke.sh\n' > "$POD_FILE"
request_multipart "POST" "/execution/legs/${BOOKING_LEG_ID}/documents/upload" "$CARRIER_TOKEN" \
  "document_name=Smoke Delivery POD" \
  "document_type=delivery_pod" \
  "file=@${POD_FILE};type=text/plain"
if http_in "$LAST_HTTP_CODE" 200 && envelope_ok && envelope_data_success; then
  summary="$(json_string "$LAST_BODY_FILE" 'echo $d["data"]["message"] ?? "POD uploaded.";')"
  record_result "Execution POD Upload" "POST" "/execution/legs/${BOOKING_LEG_ID}/documents/upload" "$LAST_HTTP_CODE" "SUCCESS" "$summary"
else
  record_result "Execution POD Upload" "POST" "/execution/legs/${BOOKING_LEG_ID}/documents/upload" "$LAST_HTTP_CODE" "FAILED" "Execution POD upload failed."
fi

run_action "complete_delivery" "Delivered"

# 19-21. Escrow lifecycle
request_json "POST" "/payments/legs/${BOOKING_LEG_ID}/fund" "{\"amount_cents\":285000,\"currency\":\"USD\",\"platform_fee_cents\":15000,\"payment_intent_id\":\"pi_smoke_${TS_ID}\",\"charge_id\":\"ch_smoke_${TS_ID}\",\"transfer_group\":\"smoke_transfer_group_${TS_ID}\",\"note\":\"Bash funding step.\"}" "$ADMIN_TOKEN"
if http_in "$LAST_HTTP_CODE" 200 && envelope_ok && envelope_data_success; then
  status_label="$(json_string "$LAST_BODY_FILE" 'echo $d["data"]["status_label"] ?? "";')"
  record_result "Escrow Fund" "POST" "/payments/legs/${BOOKING_LEG_ID}/fund" "$LAST_HTTP_CODE" "SUCCESS" "Escrow funded with status ${status_label}."
else
  record_result "Escrow Fund" "POST" "/payments/legs/${BOOKING_LEG_ID}/fund" "$LAST_HTTP_CODE" "FAILED" "Escrow funding failed."
fi

request_json "POST" "/payments/legs/${BOOKING_LEG_ID}/hold" '{"note":"Bash hold step."}' "$ADMIN_TOKEN"
if http_in "$LAST_HTTP_CODE" 200 && envelope_ok && envelope_data_success; then
  status_label="$(json_string "$LAST_BODY_FILE" 'echo $d["data"]["status_label"] ?? "";')"
  record_result "Escrow Hold" "POST" "/payments/legs/${BOOKING_LEG_ID}/hold" "$LAST_HTTP_CODE" "SUCCESS" "Escrow held with status ${status_label}."
else
  record_result "Escrow Hold" "POST" "/payments/legs/${BOOKING_LEG_ID}/hold" "$LAST_HTTP_CODE" "FAILED" "Escrow hold failed."
fi

request_json "POST" "/payments/legs/${BOOKING_LEG_ID}/release" "{\"transfer_id\":\"tr_smoke_${TS_ID}\",\"note\":\"Bash release step.\"}" "$ADMIN_TOKEN"
if http_in "$LAST_HTTP_CODE" 200 && envelope_ok && envelope_data_success; then
  status_label="$(json_string "$LAST_BODY_FILE" 'echo $d["data"]["status_label"] ?? "";')"
  record_result "Escrow Release" "POST" "/payments/legs/${BOOKING_LEG_ID}/release" "$LAST_HTTP_CODE" "SUCCESS" "Escrow released with status ${status_label}."
else
  record_result "Escrow Release" "POST" "/payments/legs/${BOOKING_LEG_ID}/release" "$LAST_HTTP_CODE" "FAILED" "Escrow release failed."
fi

# 22. Stripe webhook sync
STRIPE_BEARER="$ADMIN_TOKEN"
STRIPE_HEADERS=()
if [[ -n "$STRIPE_WEBHOOK_SECRET" ]]; then
  STRIPE_BEARER=""
  STRIPE_HEADERS=("x-stripe-webhook-secret: ${STRIPE_WEBHOOK_SECRET}")
fi
request_json "POST" "/payments/webhooks/stripe" '{"event_type":"account.updated","stripe_account_id":"acct_smoke_carrier_9103","payouts_enabled":true,"kyc_status":"verified","note":"Bash account.updated webhook."}' "$STRIPE_BEARER" "${STRIPE_HEADERS[@]}"
if http_in "$LAST_HTTP_CODE" 200 && envelope_ok; then
  ack="$(json_string "$LAST_BODY_FILE" 'echo array_key_exists("acknowledged", $d["data"] ?? []) ? ($d["data"]["acknowledged"] ? "true" : "false") : "false";')"
  msg="$(json_string "$LAST_BODY_FILE" 'echo $d["data"]["message"] ?? "Stripe webhook handled.";')"
  if [[ "$ack" == "true" ]]; then
    record_result "Stripe Webhook" "POST" "/payments/webhooks/stripe" "$LAST_HTTP_CODE" "SUCCESS" "$msg"
  else
    record_result "Stripe Webhook" "POST" "/payments/webhooks/stripe" "$LAST_HTTP_CODE" "FAILED" "$msg"
  fi
else
  record_result "Stripe Webhook" "POST" "/payments/webhooks/stripe" "$LAST_HTTP_CODE" "FAILED" "Stripe webhook sync failed."
fi

# 23-25. Marketplace
request_json "GET" "/marketplace/chat-workspace?conversation_id=${OFFER_CONVERSATION_ID}" "" "$SHIPPER_TOKEN"
if http_in "$LAST_HTTP_CODE" 200 && envelope_ok; then
  messages="$(json_string "$LAST_BODY_FILE" 'echo count($d["data"]["messages"] ?? []);')"
  record_result "Chat Workspace" "GET" "/marketplace/chat-workspace?conversation_id=${OFFER_CONVERSATION_ID}" "$LAST_HTTP_CODE" "SUCCESS" "Chat workspace loaded with ${messages} message(s)."
else
  record_result "Chat Workspace" "GET" "/marketplace/chat-workspace?conversation_id=${OFFER_CONVERSATION_ID}" "$LAST_HTTP_CODE" "FAILED" "Chat workspace could not be loaded."
fi

request_json "POST" "/marketplace/conversations/${OFFER_CONVERSATION_ID}/messages" '{"body":"Smoke test carrier reply from bash."}' "$CARRIER_TOKEN"
if http_in "$LAST_HTTP_CODE" 200 && envelope_ok && envelope_data_success; then
  summary="$(json_string "$LAST_BODY_FILE" 'echo $d["data"]["message"] ?? "Message sent.";')"
  record_result "Send Chat Message" "POST" "/marketplace/conversations/${OFFER_CONVERSATION_ID}/messages" "$LAST_HTTP_CODE" "SUCCESS" "$summary"
else
  record_result "Send Chat Message" "POST" "/marketplace/conversations/${OFFER_CONVERSATION_ID}/messages" "$LAST_HTTP_CODE" "FAILED" "Sending chat message failed."
fi

request_json "POST" "/marketplace/conversations/${OFFER_CONVERSATION_ID}/read" '{}' "$SHIPPER_TOKEN"
if http_in "$LAST_HTTP_CODE" 200 && envelope_ok && envelope_data_success; then
  summary="$(json_string "$LAST_BODY_FILE" 'echo $d["data"]["message"] ?? "Conversation marked read.";')"
  record_result "Mark Conversation Read" "POST" "/marketplace/conversations/${OFFER_CONVERSATION_ID}/read" "$LAST_HTTP_CODE" "SUCCESS" "$summary"
else
  record_result "Mark Conversation Read" "POST" "/marketplace/conversations/${OFFER_CONVERSATION_ID}/read" "$LAST_HTTP_CODE" "FAILED" "Marking conversation as read failed."
fi

# 26. Offer review
request_json "POST" "/marketplace/offers/${OFFER_ID}/review" '{"decision":"accept"}' "$SHIPPER_TOKEN"
if http_in "$LAST_HTTP_CODE" 200 && envelope_ok && envelope_data_success; then
  summary="$(json_string "$LAST_BODY_FILE" 'echo $d["data"]["message"] ?? "Offer reviewed.";')"
  record_result "Offer Review" "POST" "/marketplace/offers/${OFFER_ID}/review" "$LAST_HTTP_CODE" "SUCCESS" "$summary"
else
  record_result "Offer Review" "POST" "/marketplace/offers/${OFFER_ID}/review" "$LAST_HTTP_CODE" "FAILED" "Offer review failed."
fi

# 27-29. Admin ops
request_json "GET" "/admin/stloads/operations" "" "$ADMIN_TOKEN"
if http_in "$LAST_HTTP_CODE" 200 && envelope_ok; then
  handoffs="$(json_string "$LAST_BODY_FILE" 'echo count($d["data"]["handoffs"] ?? []);')"
  sync_issues="$(json_string "$LAST_BODY_FILE" 'echo count($d["data"]["recent_sync_issues"] ?? []);')"
  record_result "Admin Operations" "GET" "/admin/stloads/operations" "$LAST_HTTP_CODE" "SUCCESS" "Operations screen loaded with ${handoffs} handoff row(s) and ${sync_issues} sync issue(s)."
else
  record_result "Admin Operations" "GET" "/admin/stloads/operations" "$LAST_HTTP_CODE" "FAILED" "Admin operations screen failed to load."
fi

request_json "GET" "/admin/stloads/reconciliation" "" "$ADMIN_TOKEN"
if http_in "$LAST_HTTP_CODE" 200 && envelope_ok; then
  logs="$(json_string "$LAST_BODY_FILE" 'echo count($d["data"]["logs"] ?? []);')"
  record_result "Admin Reconciliation" "GET" "/admin/stloads/reconciliation" "$LAST_HTTP_CODE" "SUCCESS" "Reconciliation screen loaded with ${logs} row(s)."
else
  record_result "Admin Reconciliation" "GET" "/admin/stloads/reconciliation" "$LAST_HTTP_CODE" "FAILED" "Admin reconciliation screen failed to load."
fi

request_json "POST" "/admin/stloads/sync-errors/${SEEDED_SYNC_ERROR_ID}/resolve" '{"resolution_note":"Resolved during bash smoke validation."}' "$ADMIN_TOKEN"
if http_in "$LAST_HTTP_CODE" 200 && envelope_ok && envelope_data_success; then
  summary="$(json_string "$LAST_BODY_FILE" 'echo $d["data"]["message"] ?? "Sync error resolved.";')"
  record_result "Resolve Sync Error" "POST" "/admin/stloads/sync-errors/${SEEDED_SYNC_ERROR_ID}/resolve" "$LAST_HTTP_CODE" "SUCCESS" "$summary"
else
  record_result "Resolve Sync Error" "POST" "/admin/stloads/sync-errors/${SEEDED_SYNC_ERROR_ID}/resolve" "$LAST_HTTP_CODE" "FAILED" "Sync error resolution failed."
fi

TMS_BEARER="$ADMIN_TOKEN"
TMS_HEADERS=()
if [[ -n "$TMS_SHARED_SECRET" ]]; then
  TMS_BEARER=""
  TMS_HEADERS=("x-tms-shared-secret: ${TMS_SHARED_SECRET}")
fi

# 30-31. Seeded TMS lifecycle
request_json "POST" "/tms/withdraw" "{\"handoff_id\":${SEEDED_HANDOFF_ID},\"reason\":\"Bash withdraw step.\",\"pushed_by\":\"bash-script\",\"source_module\":\"rust_api_chain_smoke.sh\"}" "$TMS_BEARER" "${TMS_HEADERS[@]}"
if http_in "$LAST_HTTP_CODE" 200 && envelope_ok && envelope_data_success; then
  summary="$(json_string "$LAST_BODY_FILE" 'echo $d["data"]["message"] ?? "Seeded handoff withdrawn.";')"
  record_result "TMS Withdraw" "POST" "/tms/withdraw" "$LAST_HTTP_CODE" "SUCCESS" "$summary"
else
  record_result "TMS Withdraw" "POST" "/tms/withdraw" "$LAST_HTTP_CODE" "FAILED" "Seeded handoff withdraw failed."
fi

request_json "POST" "/tms/close" "{\"handoff_id\":${SEEDED_HANDOFF_ID},\"reason\":\"Bash close step.\",\"pushed_by\":\"bash-script\",\"source_module\":\"rust_api_chain_smoke.sh\"}" "$TMS_BEARER" "${TMS_HEADERS[@]}"
if http_in "$LAST_HTTP_CODE" 200 && envelope_ok && envelope_data_success; then
  summary="$(json_string "$LAST_BODY_FILE" 'echo $d["data"]["message"] ?? "Seeded handoff closed.";')"
  record_result "TMS Close" "POST" "/tms/close" "$LAST_HTTP_CODE" "SUCCESS" "$summary"
else
  record_result "TMS Close" "POST" "/tms/close" "$LAST_HTTP_CODE" "FAILED" "Seeded handoff close failed."
fi

# 32-36. Dynamic TMS flow
NEW_TMS_LOAD_ID="TMS-BASH-${TS_ID}"
NEW_EXTERNAL_HANDOFF_ID="bash-handoff-${TS_ID}"
PUSH_PAYLOAD="$(cat <<EOF
{"tms_load_id":"${NEW_TMS_LOAD_ID}","tenant_id":"${TMS_TENANT_ID}","external_handoff_id":"${NEW_EXTERNAL_HANDOFF_ID}","party_type":"shipper","freight_mode":"truckload","equipment_type":"Dry Van","commodity_description":"Bash smoke packaged goods","weight":41000.0,"weight_unit":"lbs","piece_count":22,"is_hazardous":false,"pickup_city":"Newark","pickup_state":"NJ","pickup_zip":"07114","pickup_country":"US","pickup_address":"100 Port Way, Newark, NJ","pickup_window_start":"$(date -u +%Y-%m-%dT%H:%M:%SZ)","pickup_window_end":"$(date -u +%Y-%m-%dT%H:%M:%SZ)","pickup_instructions":"Check in at dock 7","pickup_appointment_ref":"APT-PICKUP-BASH","dropoff_city":"Chicago","dropoff_state":"IL","dropoff_zip":"60601","dropoff_country":"US","dropoff_address":"400 Market Ave, Chicago, IL","dropoff_window_start":"$(date -u +%Y-%m-%dT%H:%M:%SZ)","dropoff_window_end":"$(date -u +%Y-%m-%dT%H:%M:%SZ)","dropoff_instructions":"Delivery by noon if possible","dropoff_appointment_ref":"APT-DROPOFF-BASH","board_rate":3350.0,"rate_currency":"USD","accessorial_flags":{"lumper":true},"bid_type":"Fixed","quote_status":"open","tender_posture":"tendered","compliance_passed":true,"compliance_summary":{"passed":true,"notes":["seeded for bash smoke"]},"required_documents_status":{"bol":"required","pod":"required"},"readiness":"ready","pushed_by":"bash-script","push_reason":"Rust bash smoke validation","source_module":"rust_api_chain_smoke.sh","payload_version":"1.0","external_refs":[{"ref_type":"load_number","ref_value":"BASH-TMS-${TS_ID}","ref_source":"bash_script"}]}
EOF
)"

request_json "POST" "/tms/push" "$PUSH_PAYLOAD" "$TMS_BEARER" "${TMS_HEADERS[@]}"
NEW_HANDOFF_ID=""
if http_in "$LAST_HTTP_CODE" 200 && envelope_ok && envelope_data_success; then
  NEW_HANDOFF_ID="$(json_string "$LAST_BODY_FILE" 'echo $d["data"]["handoff_id"] ?? "";')"
  NEW_LOAD_ID="$(json_string "$LAST_BODY_FILE" 'echo $d["data"]["load_id"] ?? "";')"
  record_result "TMS Push" "POST" "/tms/push" "$LAST_HTTP_CODE" "SUCCESS" "Published handoff ${NEW_HANDOFF_ID} as load ${NEW_LOAD_ID}."
else
  record_result "TMS Push" "POST" "/tms/push" "$LAST_HTTP_CODE" "FAILED" "Dynamic TMS push failed."
fi

request_json "POST" "/tms/webhook/status" "{\"tms_load_id\":\"${NEW_TMS_LOAD_ID}\",\"tenant_id\":\"${TMS_TENANT_ID}\",\"tms_status\":\"dispatched\",\"status_at\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\",\"source_module\":\"rust_api_chain_smoke.sh\",\"pushed_by\":\"bash-script\",\"detail\":\"Rate update webhook from bash smoke.\",\"rate_update\":3450.0}" "$TMS_BEARER" "${TMS_HEADERS[@]}"
if http_in "$LAST_HTTP_CODE" 200 && envelope_ok && envelope_data_success; then
  summary="$(json_string "$LAST_BODY_FILE" 'echo $d["data"]["message"] ?? "Status webhook accepted.";')"
  record_result "TMS Status Webhook" "POST" "/tms/webhook/status" "$LAST_HTTP_CODE" "SUCCESS" "$summary"
else
  record_result "TMS Status Webhook" "POST" "/tms/webhook/status" "$LAST_HTTP_CODE" "FAILED" "TMS status webhook failed."
fi

if [[ -n "$NEW_HANDOFF_ID" ]]; then
  request_json "POST" "/tms/requeue" "{\"handoff_id\":${NEW_HANDOFF_ID},\"pushed_by\":\"bash-script\",\"source_module\":\"rust_api_chain_smoke.sh\"}" "$TMS_BEARER" "${TMS_HEADERS[@]}"
  if http_in "$LAST_HTTP_CODE" 200 && envelope_ok && envelope_data_success; then
    summary="$(json_string "$LAST_BODY_FILE" 'echo $d["data"]["message"] ?? "Handoff requeued.";')"
    record_result "TMS Requeue" "POST" "/tms/requeue" "$LAST_HTTP_CODE" "SUCCESS" "$summary"
  else
    record_result "TMS Requeue" "POST" "/tms/requeue" "$LAST_HTTP_CODE" "FAILED" "TMS requeue failed for handoff ${NEW_HANDOFF_ID}."
  fi
else
  record_result "TMS Requeue" "POST" "/tms/requeue" "PRECHECK" "FAILED" "Skipped because the dynamic TMS push did not produce a handoff id."
fi

request_json "POST" "/api/stloads/webhook/cancel" "{\"tms_load_id\":\"${NEW_TMS_LOAD_ID}\",\"tenant_id\":\"${TMS_TENANT_ID}\",\"reason\":\"Bash cancel webhook.\",\"pushed_by\":\"bash-script\",\"source_module\":\"rust_api_chain_smoke.sh\"}" "$TMS_BEARER" "${TMS_HEADERS[@]}"
if http_in "$LAST_HTTP_CODE" 200 && envelope_ok && envelope_data_success; then
  summary="$(json_string "$LAST_BODY_FILE" 'echo $d["data"]["message"] ?? "Cancel webhook accepted.";')"
  record_result "TMS Cancel Webhook" "POST" "/api/stloads/webhook/cancel" "$LAST_HTTP_CODE" "SUCCESS" "$summary"
else
  record_result "TMS Cancel Webhook" "POST" "/api/stloads/webhook/cancel" "$LAST_HTTP_CODE" "FAILED" "Cancel webhook failed."
fi

request_json "POST" "/api/stloads/webhook/close" "{\"tms_load_id\":\"${NEW_TMS_LOAD_ID}\",\"tenant_id\":\"${TMS_TENANT_ID}\",\"reason\":\"Bash close webhook.\",\"pushed_by\":\"bash-script\",\"source_module\":\"rust_api_chain_smoke.sh\"}" "$TMS_BEARER" "${TMS_HEADERS[@]}"
if http_in "$LAST_HTTP_CODE" 200 && envelope_ok && envelope_data_success; then
  summary="$(json_string "$LAST_BODY_FILE" 'echo $d["data"]["message"] ?? "Close webhook accepted.";')"
  record_result "TMS Close Webhook" "POST" "/api/stloads/webhook/close" "$LAST_HTTP_CODE" "SUCCESS" "$summary"
else
  record_result "TMS Close Webhook" "POST" "/api/stloads/webhook/close" "$LAST_HTTP_CODE" "FAILED" "Close webhook failed."
fi

echo
printf '%-28s %-6s %-44s %-8s %s\n' "API" "METHOD" "PATH" "STATUS" "SUMMARY"
printf '%-28s %-6s %-44s %-8s %s\n' "----------------------------" "------" "--------------------------------------------" "--------" "----------------------------------------------"
for i in "${!RESULT_NAMES[@]}"; do
  printf '%-28s %-6s %-44s %-8s %s\n' \
    "${RESULT_NAMES[$i]}" \
    "${RESULT_METHODS[$i]}" \
    "${RESULT_PATHS[$i]}" \
    "${RESULT_STATES[$i]}" \
    "${RESULT_SUMMARIES[$i]}"
done

echo
echo "Final summary"
echo "  APIs ran:   $TOTAL_RUN"
echo "  Successful: $TOTAL_SUCCESS"
echo "  Failed:     $TOTAL_FAILED"

if [[ "$TOTAL_FAILED" -gt 0 ]]; then
  exit 1
fi

