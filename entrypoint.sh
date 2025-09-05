#!/usr/bin/env bash
set -euo pipefail

# Copy static files to volume
rm -rf /static/*
cp -Rv /flugbuech/static/* /static/

# Run API
exec ./flugbuech-api --migrate
