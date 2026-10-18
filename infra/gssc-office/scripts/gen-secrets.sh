#!/usr/bin/env sh
# Writes .env with fresh secrets; refuses to overwrite an existing one.
set -eu
cd "$(dirname "$0")/.."
[ -e .env ] && { echo ".env exists; not overwriting" >&2; exit 1; }
umask 077
{
  echo "GARAGE_RPC_SECRET=$(openssl rand -hex 32)"
  echo "GARAGE_ADMIN_TOKEN=$(openssl rand -base64 32)"
} > .env
echo "wrote .env"
