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

current_js="$(find /usr/share/nginx/html -maxdepth 1 -name 'frontend-leptos-*.js' | sort | tail -n 1 || true)"
current_wasm="$(find /usr/share/nginx/html -maxdepth 1 -name 'frontend-leptos-*_bg.wasm' | sort | tail -n 1 || true)"
if [ -n "${current_js}" ] && [ -n "${current_wasm}" ]; then
  asset_token="$(sha256sum "${current_js}" | awk '{ print substr($1, 1, 12) }')"
  current_js_base="$(basename "${current_js}")"
  current_wasm_base="$(basename "${current_wasm}")"
  public_js_base="frontend-leptos-${asset_token}.js"
  public_wasm_base="frontend-leptos-${asset_token}_bg.wasm"
  public_js="/usr/share/nginx/html/${public_js_base}"
  public_wasm="/usr/share/nginx/html/${public_wasm_base}"

  cp "${current_js}" "${public_js}"
  cp "${current_wasm}" "${public_wasm}"
  sed -i "s#/${current_wasm_base}#/${public_wasm_base}#g" "${public_js}"

  snippet_dir="$(find /usr/share/nginx/html/snippets -mindepth 1 -maxdepth 1 -type d -name 'frontend-leptos-*' | sort | tail -n 1 || true)"
  if [ -n "${snippet_dir}" ]; then
    snippet_base="$(basename "${snippet_dir}")"
    public_snippet_base="${snippet_base}-${asset_token}"
    public_snippet_dir="/usr/share/nginx/html/snippets/${public_snippet_base}"
    rm -rf "${public_snippet_dir}"
    cp -R "${snippet_dir}" "${public_snippet_dir}"
    sed -i "s#snippets/${snippet_base}/#snippets/${public_snippet_base}/#g" "${public_js}"
    for inline_index in 0 1 2; do
      sed -i "s#\"./snippets/${public_snippet_base}/inline${inline_index}.js\":#\"./snippets/${snippet_base}/inline${inline_index}.js\":#g" "${public_js}"
    done
    sed -i "s#snippets/${snippet_base}/#snippets/${public_snippet_base}/#g" /usr/share/nginx/html/index.html
  fi

  sed -i "s#/${current_js_base}#/${public_js_base}#g" /usr/share/nginx/html/index.html
  sed -i "s#/${current_wasm_base}#/${public_wasm_base}#g" /usr/share/nginx/html/index.html
fi
for stale_hash in 69f399d7c1135858 5e7db52fa393058 9c4bcbb11e220425 1382f083b29e05c0; do
  if [ -n "${current_js}" ] && [ ! -e "/usr/share/nginx/html/frontend-leptos-${stale_hash}.js" ]; then
    cp "${current_js}" "/usr/share/nginx/html/frontend-leptos-${stale_hash}.js"
  fi
  if [ -n "${current_wasm}" ] && [ ! -e "/usr/share/nginx/html/frontend-leptos-${stale_hash}_bg.wasm" ]; then
    cp "${current_wasm}" "/usr/share/nginx/html/frontend-leptos-${stale_hash}_bg.wasm"
  fi
done

envsubst '${BACKEND_UPSTREAM}' < /etc/nginx/templates/default.conf.template > /etc/nginx/conf.d/default.conf
envsubst '${BACKEND_API_BASE_URL} ${GOOGLE_MAPS_API_KEY} ${FRONTEND_PUBLIC_URL}' < /app/templates/runtime-config.js.template > /usr/share/nginx/html/runtime-config.js
sed -i -E 's/ integrity="sha384-[^"]+"//g' /usr/share/nginx/html/index.html

exec nginx -g 'daemon off;'
