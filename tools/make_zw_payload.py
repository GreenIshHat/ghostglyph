#!/usr/bin/env python3
"""
Minimal no-dependency helper for generating Ghostglyph zero-width fixtures.

Mapping:
    U+200B ZERO WIDTH SPACE     = 0
    U+200C ZERO WIDTH NON-JOINER = 1
"""

import sys

ZERO = "\u200b"
ONE = "\u200c"

def encode(payload: str) -> str:
    out = []
    for b in payload.encode("utf-8"):
        for bit in range(7, -1, -1):
            out.append(ONE if ((b >> bit) & 1) else ZERO)
    return "".join(out)

def decode(text: str) -> str:
    bits = "".join("0" if ch == ZERO else "1" for ch in text if ch in (ZERO, ONE))
    if len(bits) % 8:
        raise SystemExit(f"bit length is not divisible by 8: {len(bits)}")
    data = bytes(int(bits[i:i+8], 2) for i in range(0, len(bits), 8))
    return data.decode("utf-8")

def main() -> None:
    if len(sys.argv) < 2 or sys.argv[1] not in {"encode", "decode"}:
        print("usage: make_zw_payload.py encode TEXT | decode [FILE]", file=sys.stderr)
        raise SystemExit(2)

    if sys.argv[1] == "encode":
        payload = " ".join(sys.argv[2:]) if len(sys.argv) > 2 else sys.stdin.read()
        print(encode(payload), end="")
    else:
        text = open(sys.argv[2], encoding="utf-8").read() if len(sys.argv) > 2 else sys.stdin.read()
        print(decode(text))

if __name__ == "__main__":
    main()
