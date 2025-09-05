#!/usr/bin/env bash
set -euo pipefail

# Copy static files to volume
rm -rf /static/*
cp -Rv /flugbuech/static/* /static/
find /static -type f -exec chmod a+r {} \;
find /static -type d -exec chmod a+rx {} \;

# Run API
exec ./flugbuech-api --migrate
