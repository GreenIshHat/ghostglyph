#!/usr/bin/env bash
set -euo pipefail

BIN="${GHOSTGLYPH_BIN:-./target/release/ghostglyph}"

last_hash=""

while sleep 1; do
  text="$(wl-paste 2>/dev/null || true)"
  hash="$(printf '%s' "$text" | sha256sum | awk '{print $1}')"

  if [ "$hash" = "$last_hash" ]; then
    continue
  fi

  last_hash="$hash"

  if printf '%s' "$text" | "$BIN" summary >/tmp/ghostglyph.clip 2>&1; then
    continue
  fi

  notify-send "Ghostglyph" "Suspicious invisible Unicode in clipboard"
done
