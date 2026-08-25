#!/usr/bin/env sh
set -eu

BIN="${GHOSTGLYPH_BIN:-ghostglyph}"

need_bin() {
  command -v "$BIN" >/dev/null 2>&1 || {
    echo "ghostglyph binary not found: $BIN" >&2
    echo "set GHOSTGLYPH_BIN=./target/release/ghostglyph or install gg" >&2
    exit 2
  }
}

paste_clipboard() {
  if command -v wl-paste >/dev/null 2>&1; then
    wl-paste
  elif command -v xclip >/dev/null 2>&1; then
    xclip -selection clipboard -o
  elif command -v xsel >/dev/null 2>&1; then
    xsel --clipboard --output
  elif command -v pbpaste >/dev/null 2>&1; then
    pbpaste
  elif command -v powershell.exe >/dev/null 2>&1; then
    powershell.exe -NoProfile -Command Get-Clipboard
  else
    echo "no clipboard reader found: install wl-clipboard, xclip, xsel, or run on macOS with pbpaste" >&2
    exit 2
  fi
}

need_bin
paste_clipboard | "$BIN" summary
