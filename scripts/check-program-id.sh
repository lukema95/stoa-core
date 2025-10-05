#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LIB="$ROOT/programs/stoa/src/lib.rs"
ANCHOR="$ROOT/Anchor.toml"
IDL="$ROOT/target/idl/stoa.json"

DECLARED="$(sed -n 's/.*declare_id!("\([^"]*\)").*/\1/p' "$LIB" | head -1)"
ANCHOR_ID="$(awk -F'"' '/^\[programs\.(localnet|devnet)\]/{found=1; next} /^\[/{found=0} found && /^stoa[[:space:]]*=/{print $2; exit}' "$ANCHOR")"

if [[ -z "$DECLARED" || -z "$ANCHOR_ID" ]]; then
  echo "error: could not read program ID from $LIB or $ANCHOR" >&2
  exit 1
fi

if [[ "$DECLARED" != "$ANCHOR_ID" ]]; then
  echo "error: declare_id ($DECLARED) != Anchor.toml ($ANCHOR_ID)" >&2
  exit 1
fi

if [[ -f "$IDL" ]]; then
  IDL_ID="$(node -p "require('$IDL').address")"
  if [[ "$DECLARED" != "$IDL_ID" ]]; then
    echo "error: declare_id ($DECLARED) != IDL address ($IDL_ID)" >&2
    exit 1
  fi
fi

echo "program ID ok: $DECLARED"
