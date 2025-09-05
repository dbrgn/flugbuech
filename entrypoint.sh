#!/usr/bin/env bash
set -euo pipefail

# Copy static files to volume
rm -rf /static/*
cp -Rv /flugbuech/static/* /static/
find /static -type f -exec chmod a+r {} \;
find /static -type d -exec chmod a+rx {} \;

# Enable statistics if enabled through env var
insert_snippet() {
  local file="$1"
  local domain="$2"
  local url="$3"

  local marker="<!--end-of-head-->"
  local line1="<!-- self-hosted, GDPR compliant, non-privacy-invading visitor stats -->"
  local line2="<script defer data-domain=\"${domain}\" src=\"${url}\"></script>"

  sed -i "/${marker}/i $line1" "$file"
  sed -i "s|${marker}|$line2|" "$file"
}
if [[ -n "${PLAUSIBLE_DOMAIN:-}" && -n "${PLAUSIBLE_URL:-}" ]]; then
  insert_snippet "/static/index.html" "$PLAUSIBLE_DOMAIN" "$PLAUSIBLE_URL"
  insert_snippet "/static/fallback.html" "$PLAUSIBLE_DOMAIN" "$PLAUSIBLE_URL"
fi

# Run API
exec ./flugbuech-api --migrate
