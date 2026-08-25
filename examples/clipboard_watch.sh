#!/usr/bin/env sh
set -u

BIN="${GHOSTGLYPH_BIN:-ghostglyph}"
INTERVAL="${GHOSTGLYPH_INTERVAL:-2}"

if ! command -v "$BIN" >/dev/null 2>&1; then
  echo "ghostglyph binary not found: $BIN" >&2
  echo "set GHOSTGLYPH_BIN=./target/release/ghostglyph or install gg" >&2
  exit 2
fi

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
    return 127
  fi
}

hash_file() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | awk '{print $1}'
  else
    cksum "$1" | awk '{print $1 ":" $2}'
  fi
}

notify() {
  title="$1"
  body="$2"
  if command -v notify-send >/dev/null 2>&1; then
    notify-send "$title" "$body"
  else
    printf '%s: %s\n' "$title" "$body"
  fi
}

tmp="$(mktemp)"
trap 'rm -f "$tmp"' EXIT INT TERM

last_hash=""

echo "ghostglyph clipboard watcher running"
echo "interval: ${INTERVAL}s"
echo "bin: $BIN"

while :; do
  if ! paste_clipboard > "$tmp" 2>/dev/null; then
    echo "no clipboard reader found or clipboard unavailable" >&2
    echo "install wl-clipboard/xclip/xsel, or run this on the host instead of a container/toolbox" >&2
    exit 2
  fi

  current_hash="$(hash_file "$tmp")"

  if [ "$current_hash" != "$last_hash" ]; then
    last_hash="$current_hash"

    output="$("$BIN" summary "$tmp" 2>&1)"
    code="$?"

    if [ "$code" = "1" ]; then
      printf '\n%s\n' "$output"
      notify "Ghostglyph" "Clipboard contains suspicious invisible Unicode"
    elif [ "$code" = "2" ]; then
      printf '\n%s\n' "$output" >&2
    fi
  fi

  sleep "$INTERVAL"
done
