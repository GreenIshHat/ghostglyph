use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io::{self, Read};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Severity {
    Info,
    Warn,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Strict,
    Compat,
}

#[derive(Debug, Clone)]
struct Finding {
    byte_offset: usize,
    ch: char,
    codepoint: u32,
    name: &'static str,
    short: &'static str,
    class: &'static str,
    severity: Severity,
}

fn main() {
    let mut args: Vec<String> = env::args().skip(1).collect();

    if args.iter().any(|a| a == "-h" || a == "--help" || a == "help") {
        print_help();
        return;
    }

    let mut mode = Mode::Strict;
    if let Some(i) = args.iter().position(|a| a == "--mode") {
        if i + 1 >= args.len() {
            eprintln!("ghostglyph: --mode requires strict or compat");
            std::process::exit(2);
        }
        mode = match args[i + 1].as_str() {
            "strict" | "paranoid" => Mode::Strict,
            "compat" | "compatible" => Mode::Compat,
            other => {
                eprintln!("ghostglyph: unknown mode: {other}");
                std::process::exit(2);
            }
        };
        args.drain(i..=i + 1);
    }

    let command = args.get(0).map(String::as_str).unwrap_or("scan");
    let path_or_text = args.get(1).map(String::as_str);

    if command == "encode-zw" {
        let payload = match path_or_text {
            Some(text) => text.to_string(),
            None => match read_stdin() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("ghostglyph: stdin error: {e}");
                    std::process::exit(2);
                }
            },
        };
        print!("{}", encode_zero_width_bits(payload.trim_end_matches('\n')));
        return;
    }

    let input = match read_input(path_or_text) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("ghostglyph: input error: {e}");
            std::process::exit(2);
        }
    };

    let findings = scan(&input, mode);
    let escaped_findings = scan_source_escapes(&input);

    match command {
        "scan" => {
            if findings.is_empty() && escaped_findings.is_empty() {
                println!("OK: no suspicious invisible/control Unicode found");
                std::process::exit(0);
            }

            print_findings(&findings);
            print_escape_findings(&escaped_findings);

            print_zero_width_candidate_stderr(&input);

            if let Some(decoded) = decode_unicode_tags(&input) {
                eprintln!();
                eprintln!("decoded Unicode Tags candidate:");
                eprintln!("{decoded}");
            }

            std::process::exit(1);
        }

        "summary" => {
            print_summary(&input, &findings, &escaped_findings);

            if findings.is_empty() && escaped_findings.is_empty() {
                std::process::exit(0);
            } else {
                std::process::exit(1);
            }
        }

        "reveal" => {
            print_revealed(&input, mode);
            std::process::exit(if findings.is_empty() { 0 } else { 1 });
        }

        "strip" => {
            print!("{}", strip_suspicious(&input, mode));
            std::process::exit(if findings.is_empty() { 0 } else { 1 });
        }

        "decode-zw" => {
            match decode_zero_width_payload(&input) {
                Some(payload) => {
                    match payload.as_text() {
                        Some(text) => println!("{text}"),
                        None => {
                            eprintln!("zero-width payload is not printable UTF-8; emitting hex fallback");
                            println!("{}", format_hex_prefixed(&payload.bytes));
                        }
                    }
                    std::process::exit(1);
                }
                None => {
                    eprintln!("No decodable U+200B/U+200C binary payload found");
                    std::process::exit(0);
                }
            }
        }

        "decode-zw-hex" => {
            match extract_zero_width_bytes(&input) {
                Some(bytes) => {
                    println!("{}", format_hex_prefixed(&bytes));
                    std::process::exit(1);
                }
                None => {
                    eprintln!("No decodable U+200B/U+200C binary payload found");
                    std::process::exit(0);
                }
            }
        }

        "decode-tags" => {
            match decode_unicode_tags(&input) {
                Some(decoded) => {
                    println!("{decoded}");
                    std::process::exit(1);
                }
                None => {
                    eprintln!("No decodable Unicode Tags payload found");
                    std::process::exit(0);
                }
            }
        }

        _ => {
            eprintln!("ghostglyph: unknown command: {command}");
            print_help();
            std::process::exit(2);
        }
    }
}

fn print_help() {
    eprintln!("ghostglyph — reveal invisible Unicode before your tools consume it");
    eprintln!();
    eprintln!("usage:");
    eprintln!("  ghostglyph scan [file]");
    eprintln!("  ghostglyph summary [file]");
    eprintln!("  ghostglyph reveal [file]");
    eprintln!("  ghostglyph strip [file]");
    eprintln!("  ghostglyph decode-zw [file]");
    eprintln!("  ghostglyph decode-zw-hex [file]");
    eprintln!("  ghostglyph decode-tags [file]");
    eprintln!("  ghostglyph encode-zw [text]");
    eprintln!();
    eprintln!("options:");
    eprintln!("  --mode strict      default; paranoid about invisible/control text");
    eprintln!("  --mode compat      quieter for common text/emoji cases");
    eprintln!();
    eprintln!("exit codes:");
    eprintln!("  0 clean");
    eprintln!("  1 suspicious content found");
    eprintln!("  2 input/scanner error");
}

fn read_input(path: Option<&str>) -> io::Result<String> {
    match path {
        Some(p) => fs::read_to_string(p),
        None => read_stdin(),
    }
}

fn read_stdin() -> io::Result<String> {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;
    Ok(buf)
}

fn scan(input: &str, mode: Mode) -> Vec<Finding> {
    let mut out = Vec::new();

    for (byte_offset, ch) in input.char_indices() {
        if let Some((name, short, class, severity)) = classify(ch, mode) {
            out.push(Finding {
                byte_offset,
                ch,
                codepoint: ch as u32,
                name,
                short,
                class,
                severity,
            });
        }
    }

    out
}

fn classify(ch: char, mode: Mode) -> Option<(&'static str, &'static str, &'static str, Severity)> {
    let cp = ch as u32;

    match cp {
        // Zero-width / invisible format characters.
        0x200B => Some(("ZERO WIDTH SPACE", "ZWSP", "zero-width", Severity::High)),
        0x200C => Some(("ZERO WIDTH NON-JOINER", "ZWNJ", "zero-width", Severity::High)),
        // strict == paranoid, because bitches hide bytes
        // compat == do not scream at normal text/emoji as much
        0x200D => Some(("ZERO WIDTH JOINER", "ZWJ", "zero-width", match mode { Mode::Strict => Severity::Warn, Mode::Compat => Severity::Info })),
        0x2060 => Some(("WORD JOINER", "WJ", "zero-width", Severity::High)),
        0xFEFF => Some(("ZERO WIDTH NO-BREAK SPACE / BOM", "BOM", "zero-width", Severity::High)),

        // Bidi controls: Trojan Source family.
        0x202A => Some(("LEFT-TO-RIGHT EMBEDDING", "LRE", "bidi", Severity::Critical)),
        0x202B => Some(("RIGHT-TO-LEFT EMBEDDING", "RLE", "bidi", Severity::Critical)),
        0x202C => Some(("POP DIRECTIONAL FORMATTING", "PDF", "bidi", Severity::Critical)),
        0x202D => Some(("LEFT-TO-RIGHT OVERRIDE", "LRO", "bidi", Severity::Critical)),
        0x202E => Some(("RIGHT-TO-LEFT OVERRIDE", "RLO", "bidi", Severity::Critical)),
        0x2066 => Some(("LEFT-TO-RIGHT ISOLATE", "LRI", "bidi", Severity::Critical)),
        0x2067 => Some(("RIGHT-TO-LEFT ISOLATE", "RLI", "bidi", Severity::Critical)),
        0x2068 => Some(("FIRST STRONG ISOLATE", "FSI", "bidi", Severity::Critical)),
        0x2069 => Some(("POP DIRECTIONAL ISOLATE", "PDI", "bidi", Severity::Critical)),
        0x200E => Some(("LEFT-TO-RIGHT MARK", "LRM", "bidi", match mode { Mode::Strict => Severity::Warn, Mode::Compat => Severity::Info })),
        0x200F => Some(("RIGHT-TO-LEFT MARK", "RLM", "bidi", match mode { Mode::Strict => Severity::Warn, Mode::Compat => Severity::Info })),
        0x061C => Some(("ARABIC LETTER MARK", "ALM", "bidi", match mode { Mode::Strict => Severity::Warn, Mode::Compat => Severity::Info })),

        // Unicode tags: ASCII smuggling / hidden payload range.
        0xE0000..=0xE007F => Some(("UNICODE TAG CHARACTER", "TAG", "tag-smuggling", Severity::Critical)),

        // Variation selectors.
        0xFE00..=0xFE0F => Some(("VARIATION SELECTOR", "VS", "variation-selector", match mode { Mode::Strict => Severity::High, Mode::Compat => Severity::Info })),
        0xE0100..=0xE01EF => Some(("VARIATION SELECTOR SUPPLEMENT", "VSS", "variation-selector", match mode { Mode::Strict => Severity::High, Mode::Compat => Severity::Warn })),

        // Line/paragraph separators.
        0x2028 => Some(("LINE SEPARATOR", "LS", "separator", Severity::Warn)),
        0x2029 => Some(("PARAGRAPH SEPARATOR", "PS", "separator", Severity::Warn)),

        // ASCII / C0 controls, except common whitespace: TAB LF CR.
        0x00..=0x08 => Some(("ASCII CONTROL", "CTRL", "control", Severity::High)),
        0x0B..=0x0C => Some(("ASCII CONTROL", "CTRL", "control", Severity::High)),
        0x0E..=0x1F => Some(("ASCII CONTROL", "CTRL", "control", Severity::High)),
        0x7F => Some(("DELETE CONTROL", "DEL", "control", Severity::High)),

        // C1 controls.
        0x80..=0x9F => Some(("C1 CONTROL", "C1", "control", Severity::High)),

        _ => None,
    }
}

fn scan_source_escapes(input: &str) -> Vec<(usize, &'static str, &'static str)> {
    let lower = input.to_ascii_lowercase();
    let patterns = [
        ("\\u000a", "JAVA_UNICODE_ESCAPE_LF", "Unicode escape may become linefeed early in Java"),
        ("\\u000d", "JAVA_UNICODE_ESCAPE_CR", "Unicode escape may become carriage return early in Java"),
        ("\\u202e", "BIDI_ESCAPE_RLO", "escaped right-to-left override"),
        ("\\u202d", "BIDI_ESCAPE_LRO", "escaped left-to-right override"),
        ("\\u200b", "ZWSP_ESCAPE", "escaped zero width space"),
        ("\\u200c", "ZWNJ_ESCAPE", "escaped zero width non-joiner"),
        ("\\u200d", "ZWJ_ESCAPE", "escaped zero width joiner"),
    ];

    let mut out = Vec::new();

    for (pat, code, msg) in patterns {
        let mut start = 0;
        while let Some(pos) = lower[start..].find(pat) {
            let absolute = start + pos;
            out.push((absolute, code, msg));
            start = absolute + pat.len();
        }
    }

    out
}

fn print_findings(findings: &[Finding]) {
    for f in findings {
        eprintln!(
            "{:?}: byte={} U+{:04X} {} ({}) class={} char={:?}",
            f.severity,
            f.byte_offset,
            f.codepoint,
            f.name,
            f.short,
            f.class,
            f.ch
        );
    }
}

fn print_escape_findings(findings: &[(usize, &'static str, &'static str)]) {
    for (offset, code, msg) in findings {
        eprintln!("High: byte={offset} {code} {msg}");
    }
}

fn print_summary(
    input: &str,
    findings: &[Finding],
    escaped_findings: &[(usize, &'static str, &'static str)],
) {
    if findings.is_empty() && escaped_findings.is_empty() {
        println!("OK: no suspicious invisible/control Unicode found");
        return;
    }

    println!("SUSPICIOUS: {} Unicode finding(s), {} escaped source finding(s)", findings.len(), escaped_findings.len());

    let mut by_class: BTreeMap<&'static str, usize> = BTreeMap::new();
    let mut by_scalar: BTreeMap<(u32, &'static str, &'static str), usize> = BTreeMap::new();

    for finding in findings {
        *by_class.entry(finding.class).or_insert(0) += 1;
        *by_scalar
            .entry((finding.codepoint, finding.short, finding.name))
            .or_insert(0) += 1;
    }

    if !by_class.is_empty() {
        println!();
        println!("by class:");
        for (class, count) in by_class {
            println!("  {class}: {count}");
        }
    }

    if !by_scalar.is_empty() {
        println!();
        println!("by code point:");
        for ((codepoint, short, name), count) in by_scalar {
            println!("  U+{codepoint:04X} {short:<5} {count:>4}  {name}");
        }
    }

    if !escaped_findings.is_empty() {
        println!();
        println!("escaped source patterns:");
        for (offset, code, msg) in escaped_findings {
            println!("  byte={offset:<5} {code:<24} {msg}");
        }
    }

    print_zero_width_candidate_stdout(input);

    if let Some(decoded) = decode_unicode_tags(input) {
        println!();
        println!("decoded Unicode Tags candidate:");
        println!("{decoded}");
    }
}

fn print_revealed(input: &str, mode: Mode) {
    for (_, ch) in input.char_indices() {
        if let Some((_, short, _, _)) = classify(ch, mode) {
            print!("<U+{:04X}:{short}>", ch as u32);
        } else {
            print!("{ch}");
        }
    }
}

fn strip_suspicious(input: &str, mode: Mode) -> String {
    input.chars().filter(|&ch| classify(ch, mode).is_none()).collect()
}

fn encode_zero_width_bits(payload: &str) -> String {
    encode_zero_width_bytes(payload.as_bytes())
}

fn encode_zero_width_bytes(bytes: &[u8]) -> String {
    let mut out = String::new();

    for byte in bytes {
        for bit in (0..8).rev() {
            if (byte >> bit) & 1 == 0 {
                out.push('\u{200B}');
            } else {
                out.push('\u{200C}');
            }
        }
    }

    out
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DecodedPayload {
    bytes: Vec<u8>,
}

impl DecodedPayload {
    fn as_text(&self) -> Option<String> {
        let decoded = String::from_utf8(self.bytes.clone()).ok()?;
        if decoded.chars().all(|c| c == '\n' || c == '\r' || c == '\t' || !c.is_control()) {
            Some(decoded)
        } else {
            None
        }
    }
}

fn extract_zero_width_bytes(input: &str) -> Option<Vec<u8>> {
    let bits: String = input
        .chars()
        .filter_map(|ch| match ch {
            '\u{200B}' => Some('0'),
            '\u{200C}' => Some('1'),
            _ => None,
        })
        .collect();

    if bits.len() < 8 || bits.len() % 8 != 0 {
        return None;
    }

    let mut bytes = Vec::new();

    for chunk in bits.as_bytes().chunks(8) {
        let s = std::str::from_utf8(chunk).ok()?;
        let b = u8::from_str_radix(s, 2).ok()?;
        bytes.push(b);
    }

    Some(bytes)
}

fn decode_zero_width_payload(input: &str) -> Option<DecodedPayload> {
    extract_zero_width_bytes(input).map(|bytes| DecodedPayload { bytes })
}

fn decode_zero_width_bits(input: &str) -> Option<String> {
    decode_zero_width_payload(input)?.as_text()
}

fn format_hex_prefixed(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("0x{b:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn print_zero_width_candidate_stdout(input: &str) {
    if let Some(payload) = decode_zero_width_payload(input) {
        println!();
        println!("decoded zero-width candidate:");

        match payload.as_text() {
            Some(decoded) => println!("{decoded}"),
            None => {
                println!("utf8: <invalid or non-printable>");
                println!("hex: {}", format_hex_prefixed(&payload.bytes));
                println!("bytes: {}", payload.bytes.len());
            }
        }
    }
}

fn print_zero_width_candidate_stderr(input: &str) {
    if let Some(payload) = decode_zero_width_payload(input) {
        eprintln!();
        eprintln!("decoded zero-width candidate:");

        match payload.as_text() {
            Some(decoded) => eprintln!("{decoded}"),
            None => {
                eprintln!("utf8: <invalid or non-printable>");
                eprintln!("hex: {}", format_hex_prefixed(&payload.bytes));
                eprintln!("bytes: {}", payload.bytes.len());
            }
        }
    }
}

fn decode_unicode_tags(input: &str) -> Option<String> {
    let mut out = String::new();

    for ch in input.chars() {
        let cp = ch as u32;

        // U+E0020..U+E007E correspond to tag versions of printable ASCII.
        if (0xE0020..=0xE007E).contains(&cp) {
            let ascii = (cp - 0xE0000) as u8 as char;
            out.push(ascii);
        }
    }

    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_zero_width_payload_chars() {
        let s = "hello\u{200B}\u{200C}assistant";
        let findings = scan(s, Mode::Strict);
        assert_eq!(findings.len(), 2);
        assert_eq!(findings[0].codepoint, 0x200B);
        assert_eq!(findings[1].codepoint, 0x200C);
    }

    #[test]
    fn encodes_and_decodes_basic_zw_binary() {
        let encoded = encode_zero_width_bits("skylimit");
        assert_eq!(decode_zero_width_bits(&encoded).unwrap(), "skylimit");
    }

    #[test]
    fn extracts_non_utf8_zero_width_payload_as_hex() {
        let encoded = encode_zero_width_bytes(&[0xFF, 0x00, 0x41]);
        let payload = decode_zero_width_payload(&encoded).unwrap();
        assert_eq!(payload.as_text(), None);
        assert_eq!(format_hex_prefixed(&payload.bytes), "0xFF 0x00 0x41");
    }

    #[test]
    fn strips_suspicious_chars() {
        let s = "he\u{200B}llo\u{202E}";
        assert_eq!(strip_suspicious(s, Mode::Strict), "hello");
    }

    #[test]
    fn detects_java_hidden_newline_escape() {
        let s = "String x = \"safe\"; // \\u000a System.out.println(\"oops\");";
        let findings = scan_source_escapes(s);
        assert!(findings.iter().any(|(_, code, _)| *code == "JAVA_UNICODE_ESCAPE_LF"));
    }

    #[test]
    fn detects_unicode_tag_range() {
        let s = "hello\u{E0061}world";
        let findings = scan(s, Mode::Strict);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].class, "tag-smuggling");
    }

    #[test]
    fn decodes_unicode_tags() {
        let s = "normal\u{E0067}\u{E0067}";
        assert_eq!(decode_unicode_tags(s).unwrap(), "gg");
    }
}
