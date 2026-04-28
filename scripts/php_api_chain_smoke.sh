#!/usr/bin/env bash

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

BASE_URL="${PHP_API_BASE_URL:-https://portal.stloads.com}"
LOGIN_EMAIL="${PHP_API_EMAIL:-}"
LOGIN_PASSWORD="${PHP_API_PASSWORD:-}"
LOGIN_ROLE_ID="${PHP_API_ROLE_ID:-3}"
COUNTRY_ID="${PHP_API_COUNTRY_ID:-1}"
ESCROW_RELEASE_PARAM="${PHP_API_ESCROW_RELEASE_PARAM:-}"
REQUEUE_HANDOFF_ID="${PHP_API_REQUEUE_HANDOFF_ID:-}"
TMS_TENANT_ID="${PHP_API_TMS_TENANT_ID:-bash-smoke-tenant}"
CURL_CONNECT_TIMEOUT="${PHP_API_CONNECT_TIMEOUT:-20}"
CURL_MAX_TIME="${PHP_API_MAX_TIME:-90}"

if [[ -z "$LOGIN_EMAIL" || -z "$LOGIN_PASSWORD" ]]; then
  echo "Set PHP_API_EMAIL and PHP_API_PASSWORD before running this script." >&2
  echo "Example:" >&2
  echo "  PHP_API_EMAIL='carrier@example.com' PHP_API_PASSWORD='secret' bash scripts/php_api_chain_smoke.sh" >&2
  exit 1
fi

for bin in curl php grep sed mktemp; do
  if ! command -v "$bin" >/dev/null 2>&1; then
    echo "Missing required dependency: $bin" >&2
    exit 1
  fi
done

TMP_DIR="$(mktemp -d)"
COOKIE_JAR="$TMP_DIR/cookies.txt"
RUN_ID="$(date +%Y%m%d%H%M%S)-$$"

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
LAST_LOCATION=""

trim() {
  printf '%s' "$1" | tr -d '\r'
}

json_eval() {
  local file="$1"
  local code="$2"
  php -r '$d=json_decode(file_get_contents($argv[1]), true); if (json_last_error() !== JSON_ERROR_NONE) { exit(3); } '"$code" "$file"
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

request_with_session() {
  local method="$1"
  local path="$2"
  local payload="${3:-}"
  local accept_header="${4:-application/json}"

  LAST_BODY_FILE="$TMP_DIR/body-${RANDOM}.txt"
  LAST_HEADERS_FILE="$TMP_DIR/headers-${RANDOM}.txt"
  LAST_LOCATION=""

  local curl_args=(
    --silent
    --show-error
    --connect-timeout "$CURL_CONNECT_TIMEOUT"
    --max-time "$CURL_MAX_TIME"
    --cookie "$COOKIE_JAR"
    --cookie-jar "$COOKIE_JAR"
    --request "$method"
    --dump-header "$LAST_HEADERS_FILE"
    --output "$LAST_BODY_FILE"
    --write-out "%{http_code}"
    --header "Accept: $accept_header"
    "$BASE_URL$path"
  )

  if [[ -n "$payload" ]]; then
    curl_args+=(--header "Content-Type: application/json" --data "$payload")
  fi

  LAST_HTTP_CODE="$(curl "${curl_args[@]}" 2>"$TMP_DIR/curl.err")" || LAST_HTTP_CODE="000"
  LAST_HTTP_CODE="$(trim "$LAST_HTTP_CODE")"
  LAST_LOCATION="$(awk 'BEGIN{IGNORECASE=1} /^Location:/ {sub(/\r$/, "", $2); print $2; exit}' "$LAST_HEADERS_FILE" 2>/dev/null || true)"
}

request_form_with_session() {
  local method="$1"
  local path="$2"
  shift 2

  LAST_BODY_FILE="$TMP_DIR/body-${RANDOM}.txt"
  LAST_HEADERS_FILE="$TMP_DIR/headers-${RANDOM}.txt"
  LAST_LOCATION=""

  local curl_args=(
    --silent
    --show-error
    --connect-timeout "$CURL_CONNECT_TIMEOUT"
    --max-time "$CURL_MAX_TIME"
    --cookie "$COOKIE_JAR"
    --cookie-jar "$COOKIE_JAR"
    --request "$method"
    --dump-header "$LAST_HEADERS_FILE"
    --output "$LAST_BODY_FILE"
    --write-out "%{http_code}"
    "$BASE_URL$path"
  )

  local pair
  for pair in "$@"; do
    curl_args+=(--data-urlencode "$pair")
  done

  LAST_HTTP_CODE="$(curl "${curl_args[@]}" 2>"$TMP_DIR/curl.err")" || LAST_HTTP_CODE="000"
  LAST_HTTP_CODE="$(trim "$LAST_HTTP_CODE")"
  LAST_LOCATION="$(awk 'BEGIN{IGNORECASE=1} /^Location:/ {sub(/\r$/, "", $2); print $2; exit}' "$LAST_HEADERS_FILE" 2>/dev/null || true)"
}

extract_csrf_token() {
  local file="$1"
  php -r '$html=file_get_contents($argv[1]); if (preg_match("/name=\"_token\" value=\"([^\"]+)\"/", $html, $m)) { echo $m[1]; }' "$file"
}

build_tms_payload() {
  local suffix="$1"
  local now_utc
  local pickup_start
  local dropoff_start
  now_utc="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  pickup_start="$(date -u -d '+1 day' +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || date -u +%Y-%m-%dT%H:%M:%SZ)"
  dropoff_start="$(date -u -d '+2 day' +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || date -u +%Y-%m-%dT%H:%M:%SZ)"

  cat <<EOF
{
  "tms_load_id": "BASH-${RUN_ID}-${suffix}",
  "tenant_id": "${TMS_TENANT_ID}",
  "external_handoff_id": "EXT-${RUN_ID}-${suffix}",
  "party_type": "shipper",
  "freight_mode": "Full Truckload",
  "equipment_type": "Dry Van",
  "commodity_description": "Bash smoke freight ${suffix}",
  "weight": 42000,
  "weight_unit": "LBS",
  "piece_count": 12,
  "is_hazardous": false,
  "pickup_city": "Dallas",
  "pickup_state": "TX",
  "pickup_zip": "75001",
  "pickup_country": "United States",
  "pickup_address": "100 Bash Origin Ln, Dallas, TX",
  "pickup_window_start": "${pickup_start}",
  "pickup_window_end": "${pickup_start}",
  "pickup_instructions": "Created by php_api_chain_smoke.sh",
  "dropoff_city": "Houston",
  "dropoff_state": "TX",
  "dropoff_zip": "77001",
  "dropoff_country": "United States",
  "dropoff_address": "200 Bash Destination Ave, Houston, TX",
  "dropoff_window_start": "${dropoff_start}",
  "dropoff_window_end": "${dropoff_start}",
  "dropoff_instructions": "Created by php_api_chain_smoke.sh",
  "board_rate": 2500,
  "rate_currency": "USD",
  "bid_type": "Open",
  "compliance_passed": true,
  "readiness": "ready",
  "pushed_by": "${LOGIN_EMAIL}",
  "push_reason": "Bash chain smoke run ${RUN_ID}",
  "source_module": "php_api_chain_smoke",
  "payload_version": "1.0",
  "generated_at": "${now_utc}"
}
EOF
}

create_published_helper() {
  local suffix="$1"
  local payload
  payload="$(build_tms_payload "$suffix")"

  request_with_session "POST" "/api/stloads/push" "$payload"
  if ! http_in "$LAST_HTTP_CODE" 200 201; then
    echo ""
    return 1
  fi

  local handoff_id
  handoff_id="$(json_string "$LAST_BODY_FILE" 'echo $d["handoff_id"] ?? "";')"
  echo "$handoff_id"
}

echo "PHP API chain smoke"
echo "Base URL: $BASE_URL"
echo "Run ID:   $RUN_ID"
echo

ROUTE_COUNT="$(php artisan route:list --path=api --json 2>/dev/null | php -r '$r=json_decode(stream_get_contents(STDIN), true); echo is_array($r) ? count($r) : 0;' 2>/dev/null || echo 0)"
echo "Current Laravel api route count: $ROUTE_COUNT"

echo
echo "Authenticating web session..."
request_with_session "GET" "/normal-login?id=${LOGIN_ROLE_ID}" ""
if ! http_in "$LAST_HTTP_CODE" 200; then
  echo "Failed to load login page: HTTP $LAST_HTTP_CODE" >&2
  exit 1
fi

CSRF_TOKEN="$(extract_csrf_token "$LAST_BODY_FILE")"
if [[ -z "$CSRF_TOKEN" ]]; then
  echo "Failed to extract CSRF token from login page." >&2
  exit 1
fi

request_form_with_session "POST" "/login" \
  "_token=${CSRF_TOKEN}" \
  "email=${LOGIN_EMAIL}" \
  "password=${LOGIN_PASSWORD}" \
  "id=${LOGIN_ROLE_ID}"

if ! http_in "$LAST_HTTP_CODE" 302; then
  echo "Login failed: expected HTTP 302, got HTTP $LAST_HTTP_CODE" >&2
  exit 1
fi

echo "Session established. Redirected to: ${LAST_LOCATION:-unknown}"
echo

# 1. GET /api/user
request_with_session "GET" "/api/user"
if http_in "$LAST_HTTP_CODE" 200; then
  summary="User fetched: $(json_string "$LAST_BODY_FILE" 'echo ($d["email"] ?? "unknown") . " (id " . ($d["id"] ?? "?") . ")";')"
  record_result "Current Auth User" "GET" "/api/user" "$LAST_HTTP_CODE" "SUCCESS" "$summary"
else
  record_result "Current Auth User" "GET" "/api/user" "$LAST_HTTP_CODE" "FAILED" "Failed to fetch authenticated user."
fi

# 2. POST /api/carrier/connect
request_with_session "POST" "/api/carrier/connect" "{}"
if http_in "$LAST_HTTP_CODE" 200; then
  account_id="$(json_string "$LAST_BODY_FILE" 'echo $d["account_id"] ?? "unknown";')"
  onboarding_url="$(json_string "$LAST_BODY_FILE" 'echo $d["onboarding_url"] ?? "";')"
  summary="Carrier connect onboarding link created for account ${account_id}${onboarding_url:+. Link host: ${onboarding_url%%/*}//}"
  record_result "Carrier Connect Create/Link" "POST" "/api/carrier/connect" "$LAST_HTTP_CODE" "SUCCESS" "$summary"
else
  record_result "Carrier Connect Create/Link" "POST" "/api/carrier/connect" "$LAST_HTTP_CODE" "FAILED" "Carrier connect link was not created."
fi

# 3. GET /api/carrier/connect/refresh
request_with_session "GET" "/api/carrier/connect/refresh" "" "*/*"
if http_in "$LAST_HTTP_CODE" 302; then
  summary="Carrier connect refresh redirect generated to ${LAST_LOCATION:-unknown}."
  record_result "Carrier Connect Refresh" "GET" "/api/carrier/connect/refresh" "$LAST_HTTP_CODE" "SUCCESS" "$summary"
else
  record_result "Carrier Connect Refresh" "GET" "/api/carrier/connect/refresh" "$LAST_HTTP_CODE" "FAILED" "Carrier connect refresh did not return the expected redirect."
fi

# 4. GET /api/countries/{country}/cities
request_with_session "GET" "/api/countries/${COUNTRY_ID}/cities"
if http_in "$LAST_HTTP_CODE" 200; then
  city_count="$(json_string "$LAST_BODY_FILE" 'echo is_array($d) ? count($d) : 0;')"
  summary="Cities list was fetched of ${city_count} item(s) for country ${COUNTRY_ID}."
  record_result "Cities By Country" "GET" "/api/countries/${COUNTRY_ID}/cities" "$LAST_HTTP_CODE" "SUCCESS" "$summary"
else
  record_result "Cities By Country" "GET" "/api/countries/${COUNTRY_ID}/cities" "$LAST_HTTP_CODE" "FAILED" "City list could not be fetched for country ${COUNTRY_ID}."
fi

# 5. POST /api/legs/{leg}/escrow/release
if [[ -n "$ESCROW_RELEASE_PARAM" ]]; then
  request_with_session "POST" "/api/legs/${ESCROW_RELEASE_PARAM}/escrow/release" "{}" "*/*"
  if http_in "$LAST_HTTP_CODE" 200 302; then
    summary="Escrow release endpoint responded for path parameter ${ESCROW_RELEASE_PARAM}."
    record_result "Escrow Release" "POST" "/api/legs/${ESCROW_RELEASE_PARAM}/escrow/release" "$LAST_HTTP_CODE" "SUCCESS" "$summary"
  else
    record_result "Escrow Release" "POST" "/api/legs/${ESCROW_RELEASE_PARAM}/escrow/release" "$LAST_HTTP_CODE" "FAILED" "Escrow release failed for path parameter ${ESCROW_RELEASE_PARAM}."
  fi
else
  record_result "Escrow Release" "POST" "/api/legs/{leg}/escrow/release" "PRECHECK" "FAILED" "Skipped: set PHP_API_ESCROW_RELEASE_PARAM to a valid release candidate id."
fi

# 6. POST /api/stloads/queue
QUEUE_PAYLOAD="$(build_tms_payload "queue")"
request_with_session "POST" "/api/stloads/queue" "$QUEUE_PAYLOAD"
QUEUE_HANDOFF_ID=""
if http_in "$LAST_HTTP_CODE" 201; then
  QUEUE_HANDOFF_ID="$(json_string "$LAST_BODY_FILE" 'echo $d["handoff_id"] ?? "";')"
  summary="Queued handoff created with id ${QUEUE_HANDOFF_ID}."
  record_result "TMS Queue" "POST" "/api/stloads/queue" "$LAST_HTTP_CODE" "SUCCESS" "$summary"
else
  record_result "TMS Queue" "POST" "/api/stloads/queue" "$LAST_HTTP_CODE" "FAILED" "Queue endpoint did not create a handoff."
fi

# 7. POST /api/stloads/push
PUSH_PAYLOAD="$(build_tms_payload "push-main")"
request_with_session "POST" "/api/stloads/push" "$PUSH_PAYLOAD"
PUSH_TMS_LOAD_ID="BASH-${RUN_ID}-push-main"
PUSH_HANDOFF_ID=""
if http_in "$LAST_HTTP_CODE" 201; then
  PUSH_HANDOFF_ID="$(json_string "$LAST_BODY_FILE" 'echo $d["handoff_id"] ?? "";')"
  PUSH_LOAD_ID="$(json_string "$LAST_BODY_FILE" 'echo $d["load_id"] ?? "";')"
  summary="Published handoff ${PUSH_HANDOFF_ID} as load ${PUSH_LOAD_ID}."
  record_result "TMS Push" "POST" "/api/stloads/push" "$LAST_HTTP_CODE" "SUCCESS" "$summary"
else
  record_result "TMS Push" "POST" "/api/stloads/push" "$LAST_HTTP_CODE" "FAILED" "Push endpoint did not publish a load."
fi

# 8. POST /api/stloads/requeue
if [[ -n "$REQUEUE_HANDOFF_ID" ]]; then
  REQUEUE_PAYLOAD="{\"handoff_id\":${REQUEUE_HANDOFF_ID}}"
  request_with_session "POST" "/api/stloads/requeue" "$REQUEUE_PAYLOAD"
  if http_in "$LAST_HTTP_CODE" 200; then
    summary="Requeued handoff $(json_string "$LAST_BODY_FILE" 'echo $d["handoff_id"] ?? "";') and published load $(json_string "$LAST_BODY_FILE" 'echo $d["load_id"] ?? "";')."
    record_result "TMS Requeue" "POST" "/api/stloads/requeue" "$LAST_HTTP_CODE" "SUCCESS" "$summary"
  else
    record_result "TMS Requeue" "POST" "/api/stloads/requeue" "$LAST_HTTP_CODE" "FAILED" "Requeue failed for handoff ${REQUEUE_HANDOFF_ID}."
  fi
elif [[ -n "$QUEUE_HANDOFF_ID" ]]; then
  record_result "TMS Requeue" "POST" "/api/stloads/requeue" "PRECHECK" "FAILED" "Queued handoff ${QUEUE_HANDOFF_ID} exists, but requeue needs an already failed or requeue_required handoff. Set PHP_API_REQUEUE_HANDOFF_ID."
else
  record_result "TMS Requeue" "POST" "/api/stloads/requeue" "PRECHECK" "FAILED" "Skipped: set PHP_API_REQUEUE_HANDOFF_ID to a handoff already in push_failed or requeue_required state."
fi

# 9. POST /api/stloads/withdraw
WITHDRAW_HANDOFF_ID="$(create_published_helper "withdraw-helper" || true)"
if [[ -n "$WITHDRAW_HANDOFF_ID" ]]; then
  request_with_session "POST" "/api/stloads/withdraw" "{\"handoff_id\":${WITHDRAW_HANDOFF_ID},\"reason\":\"Bash withdraw test ${RUN_ID}\"}"
  if http_in "$LAST_HTTP_CODE" 200; then
    summary="Withdrawn handoff $(json_string "$LAST_BODY_FILE" 'echo $d["handoff_id"] ?? "";')."
    record_result "TMS Withdraw" "POST" "/api/stloads/withdraw" "$LAST_HTTP_CODE" "SUCCESS" "$summary"
  else
    record_result "TMS Withdraw" "POST" "/api/stloads/withdraw" "$LAST_HTTP_CODE" "FAILED" "Withdraw failed for helper handoff ${WITHDRAW_HANDOFF_ID}."
  fi
else
  record_result "TMS Withdraw" "POST" "/api/stloads/withdraw" "PRECHECK" "FAILED" "Could not prepare a published handoff for withdraw."
fi

# 10. POST /api/stloads/close
CLOSE_HANDOFF_ID="$(create_published_helper "close-helper" || true)"
if [[ -n "$CLOSE_HANDOFF_ID" ]]; then
  request_with_session "POST" "/api/stloads/close" "{\"handoff_id\":${CLOSE_HANDOFF_ID},\"reason\":\"Bash close test ${RUN_ID}\"}"
  if http_in "$LAST_HTTP_CODE" 200; then
    summary="Closed handoff $(json_string "$LAST_BODY_FILE" 'echo $d["handoff_id"] ?? "";')."
    record_result "TMS Close" "POST" "/api/stloads/close" "$LAST_HTTP_CODE" "SUCCESS" "$summary"
  else
    record_result "TMS Close" "POST" "/api/stloads/close" "$LAST_HTTP_CODE" "FAILED" "Close failed for helper handoff ${CLOSE_HANDOFF_ID}."
  fi
else
  record_result "TMS Close" "POST" "/api/stloads/close" "PRECHECK" "FAILED" "Could not prepare a published handoff for close."
fi

# 11. POST /api/stloads/webhook/status
request_with_session "POST" "/api/stloads/webhook/status" "{\"tms_load_id\":\"${PUSH_TMS_LOAD_ID}\",\"tenant_id\":\"${TMS_TENANT_ID}\",\"tms_status\":\"in_transit\",\"pushed_by\":\"${LOGIN_EMAIL}\",\"source_module\":\"php_api_chain_smoke\",\"detail\":\"Status webhook from bash chain\"}"
if http_in "$LAST_HTTP_CODE" 200; then
  summary="Status webhook accepted for handoff $(json_string "$LAST_BODY_FILE" 'echo $d["handoff_id"] ?? "";'), TMS status $(json_string "$LAST_BODY_FILE" 'echo $d["tms_status"] ?? "";')."
  record_result "TMS Webhook Status" "POST" "/api/stloads/webhook/status" "$LAST_HTTP_CODE" "SUCCESS" "$summary"
else
  record_result "TMS Webhook Status" "POST" "/api/stloads/webhook/status" "$LAST_HTTP_CODE" "FAILED" "Status webhook was not accepted for ${PUSH_TMS_LOAD_ID}."
fi

# 12. POST /api/stloads/webhook/bulk-status
request_with_session "POST" "/api/stloads/webhook/bulk-status" "{\"updates\":[{\"tms_load_id\":\"${PUSH_TMS_LOAD_ID}\",\"tenant_id\":\"${TMS_TENANT_ID}\",\"tms_status\":\"delivered\",\"pushed_by\":\"${LOGIN_EMAIL}\"},{\"tms_load_id\":\"BASH-${RUN_ID}-missing\",\"tenant_id\":\"${TMS_TENANT_ID}\",\"tms_status\":\"dispatched\",\"pushed_by\":\"${LOGIN_EMAIL}\"}]}"
if http_in "$LAST_HTTP_CODE" 200; then
  summary="Bulk status completed: processed $(json_string "$LAST_BODY_FILE" 'echo $d["processed"] ?? 0;'), skipped $(json_string "$LAST_BODY_FILE" 'echo $d["skipped"] ?? 0;'), errors $(json_string "$LAST_BODY_FILE" 'echo $d["errors"] ?? 0;')."
  record_result "TMS Webhook Bulk Status" "POST" "/api/stloads/webhook/bulk-status" "$LAST_HTTP_CODE" "SUCCESS" "$summary"
else
  record_result "TMS Webhook Bulk Status" "POST" "/api/stloads/webhook/bulk-status" "$LAST_HTTP_CODE" "FAILED" "Bulk status webhook did not complete successfully."
fi

# 13. POST /api/stloads/webhook/cancel
CANCEL_HANDOFF_ID="$(create_published_helper "cancel-helper" || true)"
if [[ -n "$CANCEL_HANDOFF_ID" ]]; then
  CANCEL_TMS_LOAD_ID="BASH-${RUN_ID}-cancel-helper"
  request_with_session "POST" "/api/stloads/webhook/cancel" "{\"tms_load_id\":\"${CANCEL_TMS_LOAD_ID}\",\"tenant_id\":\"${TMS_TENANT_ID}\",\"reason\":\"Bash cancel webhook ${RUN_ID}\",\"pushed_by\":\"${LOGIN_EMAIL}\"}"
  if http_in "$LAST_HTTP_CODE" 200; then
    summary="Cancel webhook result: $(json_string "$LAST_BODY_FILE" 'echo $d["status"] ?? "unknown";') for handoff $(json_string "$LAST_BODY_FILE" 'echo $d["handoff_id"] ?? "";')."
    record_result "TMS Webhook Cancel" "POST" "/api/stloads/webhook/cancel" "$LAST_HTTP_CODE" "SUCCESS" "$summary"
  else
    record_result "TMS Webhook Cancel" "POST" "/api/stloads/webhook/cancel" "$LAST_HTTP_CODE" "FAILED" "Cancel webhook failed for helper handoff ${CANCEL_HANDOFF_ID}."
  fi
else
  record_result "TMS Webhook Cancel" "POST" "/api/stloads/webhook/cancel" "PRECHECK" "FAILED" "Could not prepare a published handoff for cancel webhook."
fi

# 14. POST /api/stloads/webhook/close
ARCHIVE_HANDOFF_ID="$(create_published_helper "archive-helper" || true)"
if [[ -n "$ARCHIVE_HANDOFF_ID" ]]; then
  ARCHIVE_TMS_LOAD_ID="BASH-${RUN_ID}-archive-helper"
  request_with_session "POST" "/api/stloads/webhook/close" "{\"tms_load_id\":\"${ARCHIVE_TMS_LOAD_ID}\",\"tenant_id\":\"${TMS_TENANT_ID}\",\"reason\":\"Bash archive webhook ${RUN_ID}\",\"pushed_by\":\"${LOGIN_EMAIL}\"}"
  if http_in "$LAST_HTTP_CODE" 200; then
    summary="Archive close webhook result: $(json_string "$LAST_BODY_FILE" 'echo $d["status"] ?? "unknown";') for handoff $(json_string "$LAST_BODY_FILE" 'echo $d["handoff_id"] ?? "";')."
    record_result "TMS Webhook Close" "POST" "/api/stloads/webhook/close" "$LAST_HTTP_CODE" "SUCCESS" "$summary"
  else
    record_result "TMS Webhook Close" "POST" "/api/stloads/webhook/close" "$LAST_HTTP_CODE" "FAILED" "Archive close webhook failed for helper handoff ${ARCHIVE_HANDOFF_ID}."
  fi
else
  record_result "TMS Webhook Close" "POST" "/api/stloads/webhook/close" "PRECHECK" "FAILED" "Could not prepare a published handoff for archive close webhook."
fi

echo
printf '%-28s %-6s %-38s %-8s %s\n' "API" "METHOD" "PATH" "STATUS" "SUMMARY"
printf '%-28s %-6s %-38s %-8s %s\n' "----------------------------" "------" "--------------------------------------" "--------" "----------------------------------------------"
for i in "${!RESULT_NAMES[@]}"; do
  printf '%-28s %-6s %-38s %-8s %s\n' \
    "${RESULT_NAMES[$i]}" \
    "${RESULT_METHODS[$i]}" \
    "${RESULT_PATHS[$i]}" \
    "${RESULT_STATES[$i]}" \
    "${RESULT_SUMMARIES[$i]}"
done

echo
echo "Final summary"
echo "  API routes discovered: $ROUTE_COUNT"
echo "  API routes exercised:  $TOTAL_RUN"
echo "  Successful:            $TOTAL_SUCCESS"
echo "  Failed:                $TOTAL_FAILED"

if [[ "$TOTAL_FAILED" -gt 0 ]]; then
  exit 1
fi

