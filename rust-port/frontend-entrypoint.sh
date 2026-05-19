#!/bin/sh
set -eu

: "${BACKEND_UPSTREAM:=https://stloads-rust-backend.28hm0zrfwqqw.us-south.codeengine.appdomain.cloud}"
: "${STLOADS_API_BASE_URL:=${BACKEND_API_BASE_URL:-}}"
: "${STLOADS_WS_BASE_URL:=}"
: "${BACKEND_API_BASE_URL:=}"
: "${GOOGLE_MAPS_API_KEY:=}"
: "${PUBLIC_STLOADS_URL:=${FRONTEND_PUBLIC_URL:-https://stloads-rust-frontend.28hm0zrfwqqw.us-south.codeengine.appdomain.cloud}}"
: "${FRONTEND_PUBLIC_URL:=${PUBLIC_STLOADS_URL}}"

if [ -z "${STLOADS_API_BASE_URL}" ]; then
  STLOADS_API_BASE_URL=""
fi

if [ -z "${BACKEND_API_BASE_URL}" ]; then
  BACKEND_API_BASE_URL="${STLOADS_API_BASE_URL}"
fi

export BACKEND_UPSTREAM STLOADS_API_BASE_URL STLOADS_WS_BASE_URL BACKEND_API_BASE_URL GOOGLE_MAPS_API_KEY PUBLIC_STLOADS_URL FRONTEND_PUBLIC_URL

envsubst '${BACKEND_UPSTREAM}' < /etc/nginx/templates/default.conf.template > /etc/nginx/conf.d/default.conf
envsubst '${STLOADS_API_BASE_URL} ${STLOADS_WS_BASE_URL} ${GOOGLE_MAPS_API_KEY} ${PUBLIC_STLOADS_URL}' < /app/templates/runtime-config.js.template > /usr/share/nginx/html/runtime-config.js

exec nginx -g 'daemon off;'
