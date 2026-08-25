#!/usr/bin/env bash
set -euo pipefail

BIN="${GHOSTGLYPH_BIN:-./target/release/ghostglyph}"

cat fixtures/haunted_hello_witchers.txt | "$BIN" summary || true
cat fixtures/haunted_hello_witchers.txt | "$BIN" decode-zw || true
cat fixtures/haunted_hello_witchers.txt | "$BIN" strip
