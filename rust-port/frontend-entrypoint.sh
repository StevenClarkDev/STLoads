#!/bin/sh
set -eu

: "${BACKEND_UPSTREAM:=https://stloads-rust-backend.28hm0zrfwqqw.us-south.codeengine.appdomain.cloud}"
: "${BACKEND_API_BASE_URL:=}"
: "${GOOGLE_MAPS_API_KEY:=}"
: "${FRONTEND_PUBLIC_URL:=}"

if [ -z "${BACKEND_API_BASE_URL}" ]; then
  BACKEND_API_BASE_URL=""
fi

export BACKEND_UPSTREAM BACKEND_API_BASE_URL GOOGLE_MAPS_API_KEY FRONTEND_PUBLIC_URL

envsubst '${BACKEND_UPSTREAM}' < /etc/nginx/templates/default.conf.template > /etc/nginx/conf.d/default.conf
envsubst '${BACKEND_API_BASE_URL} ${GOOGLE_MAPS_API_KEY} ${FRONTEND_PUBLIC_URL}' < /app/templates/runtime-config.js.template > /usr/share/nginx/html/runtime-config.js

exec nginx -g 'daemon off;'
