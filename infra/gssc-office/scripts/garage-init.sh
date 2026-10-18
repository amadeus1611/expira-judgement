#!/usr/bin/env sh
# One-time Garage setup: assign this node, create the mail bucket and a key.
# Usage: ./scripts/garage-init.sh <capacity, e.g. 500G>
set -eu
cd "$(dirname "$0")/.."
cap="${1:?capacity, e.g. 500G}"
g() { docker compose exec -T garage /garage "$@"; }
node=$(g node id -q | cut -d@ -f1)
g layout assign -z office -c "$cap" "$node"
g layout apply --version 1
g bucket create stalwart
g key create stalwart-key
g bucket allow --read --write --owner stalwart --key stalwart-key
echo "Put the key ID/secret above into Stalwart: Settings > Storage > Blob > S3"
echo "endpoint http://garage:3900, region garage, bucket stalwart"
