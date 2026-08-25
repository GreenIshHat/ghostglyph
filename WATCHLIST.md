# Ghostglyph Watchlist

Current watchlist for Ghostglyph v0.2.

## Zero-width and invisible joiners

    U+200B  ZERO WIDTH SPACE
    U+200C  ZERO WIDTH NON-JOINER
    U+200D  ZERO WIDTH JOINER
    U+2060  WORD JOINER
    U+FEFF  ZERO WIDTH NO-BREAK SPACE / BOM

## Bidirectional controls

    U+202A  LEFT-TO-RIGHT EMBEDDING
    U+202B  RIGHT-TO-LEFT EMBEDDING
    U+202C  POP DIRECTIONAL FORMATTING
    U+202D  LEFT-TO-RIGHT OVERRIDE
    U+202E  RIGHT-TO-LEFT OVERRIDE
    U+2066  LEFT-TO-RIGHT ISOLATE
    U+2067  RIGHT-TO-LEFT ISOLATE
    U+2068  FIRST STRONG ISOLATE
    U+2069  POP DIRECTIONAL ISOLATE
    U+200E  LEFT-TO-RIGHT MARK
    U+200F  RIGHT-TO-LEFT MARK
    U+061C  ARABIC LETTER MARK

## Unicode Tags

    U+E0000..U+E007F  tag characters

Ghostglyph can decode printable ASCII payloads from U+E0020..U+E007E.

## Variation selectors

    U+FE00..U+FE0F    variation selectors
    U+E0100..U+E01EF  variation selectors supplement

## Controls and separators

    U+0000..U+0008
    U+000B..U+000C
    U+000E..U+001F
    U+007F
    U+0080..U+009F
    U+2028  LINE SEPARATOR
    U+2029  PARAGRAPH SEPARATOR

Common TAB, LF, and CR are not flagged by default.

## Escaped source patterns

    \u000a
    \u000d
    \u202e
    \u202d
    \u200b
    \u200c
    \u200d

## v0.2.2

- Zero-width decoder now has hex fallback for non-UTF-8 payloads.
- Added `decode-zw-hex` for 0x-prefixed byte output.
- Added `fixtures/haunted_binary_ff.txt`.
