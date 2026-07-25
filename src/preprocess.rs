//! single pre-parse pass over the raw input that
//!
//! 1. expands legacy `<isindex>` start tags (modern parsers dropped the
//!    behavior but the vendored dompurify fixtures expect it), and
//! 2. records where `<svg>`/`<math>` start tags carried an `xmlns` attribute,
//!    because html5ever consumes namespace declarations while browsers keep
//!    them as serializable attributes.

/// prompt text mandated by the old html spec.
const PROMPT: &str = "This is a searchable index. Enter search keywords: ";

/// rawtext elements whose content must never be scanned or expanded.
const RAW_TEXT: &[&str] = &[
    "script",
    "style",
    "textarea",
    "title",
    "xmp",
    "iframe",
    "noembed",
    "noframes",
    "plaintext",
];

/// result of scanning the dirty input before parsing.
pub struct PreScan {
    /// input with isindex tags expanded, when any were present.
    pub html: Option<String>,
    /// `(nth svg/math start tag, index of its xmlns attribute)` pairs.
    pub xmlns_spots: Vec<(u32, u16)>,
    /// how many svg/math start tags were seen, for ordinal bookkeeping i guess.
    foreign_seen: u32,
}

/// scans dirty n returns owned html only when isindex expansion kicked in.
pub fn prescan(dirty: &str) -> PreScan {
    let mut scan = PreScan {
        html: None,
        xmlns_spots: Vec::new(),
        foreign_seen: 0,
    };
    if !needs_scan(dirty) {
        return scan;
    }
    let mut out = String::with_capacity(dirty.len());
    let mut expanded = false;
    let mut rest = dirty;
    while let Some(pos) = rest.find('<') {
        out.push_str(&rest[..pos]);
        let after = &rest[pos..];
        rest = match action_for(after) {
            Action::IsIndex(attrs) => {
                expand_isindex(&mut out, attrs);
                expanded = true;
                &after[tag_len(&after["<isindex".len()..]) + "<isindex".len()..]
            }
            Action::Foreign(name_end, total) => {
                record_xmlns_spot(&mut scan, after, name_end, total);
                out.push_str(&after[..total]);
                &after[total..]
            }
            Action::RawText(consumed) => {
                copy_raw_text(&mut out, after, consumed);
                &after[consumed..]
            }
            Action::Copy(consumed) => {
                out.push_str(&after[..consumed]);
                &after[consumed..]
            }
            Action::Literal => {
                out.push('<');
                &after[1..]
            }
        };
    }
    out.push_str(rest);
    if expanded {
        scan.html = Some(out);
    }
    scan
}

fn needs_scan(dirty: &str) -> bool {
    has_ci(dirty, b"<isindex") || has_ci(dirty, b"<svg") || has_ci(dirty, b"<math")
}

fn has_ci(haystack: &str, needle: &[u8]) -> bool {
    haystack
        .as_bytes()
        .windows(needle.len())
        .any(|w| w.eq_ignore_ascii_case(needle))
}

enum Action<'a> {
    /// expand an isindex tag; payload is the attribute source slice.
    IsIndex(&'a str),
    /// svg/math start tag: name-end and total lengths for xmlns bookkeeping.
    Foreign(usize, usize),
    /// copy a rawtext start tag of this total length plus verbatim content.
    RawText(usize),
    /// copy an already-parsed construct of this total length.
    Copy(usize),
    /// lone `<` that opens nothing.
    Literal,
}

fn action_for(after: &str) -> Action<'_> {
    let Some(second) = after.as_bytes().get(1).copied() else {
        return Action::Literal;
    };
    match second {
        b'!' if after.starts_with("<!--") => Action::Copy(comment_len(after)),
        b'!' | b'?' => Action::Copy(bogus_len(after)),
        b'/' => Action::Copy(tag_len(&after[1..]) + 1),
        c if c.is_ascii_alphabetic() => start_tag_action(after),
        _ => Action::Literal,
    }
}

fn start_tag_action(after: &str) -> Action<'_> {
    let Some(name_len) = tag_name_len(&after[1..]) else {
        return Action::Literal;
    };
    let total = tag_len(after);
    let name = &after[1..1 + name_len];
    if name.eq_ignore_ascii_case("isindex") {
        return Action::IsIndex(&after[1 + name_len..total - 1]);
    }
    let foreign = name.eq_ignore_ascii_case("svg") || name.eq_ignore_ascii_case("math");
    let raw = RAW_TEXT
        .iter()
        .find(|candidate| name.eq_ignore_ascii_case(candidate));
    match (foreign, raw) {
        (true, _) => Action::Foreign(1 + name_len, total),
        (_, Some(_)) if !self_closing(after, total) => Action::RawText(total),
        _ => Action::Copy(total),
    }
}

fn self_closing(tag: &str, total: usize) -> bool {
    tag[total.saturating_sub(2)..total].eq_ignore_ascii_case("/>")
}

/// records `(ordinal, attribute index)` when the tag carries a plain `xmlns`.
fn record_xmlns_spot(scan: &mut PreScan, tag: &str, name_end: usize, total: usize) {
    let ordinal = scan.foreign_seen;
    scan.foreign_seen += 1;
    let body = &tag[name_end..total.saturating_sub(1)];
    for (index, (name, _)) in IterAttrs::new(body).enumerate() {
        if name.eq_ignore_ascii_case("xmlns") {
            scan.xmlns_spots.push((ordinal, index as u16));
            break;
        }
    }
}

/// appends the expanded isindex form for the collected attribute source.
fn expand_isindex(out: &mut String, attrs: &str) {
    let attrs = attrs.trim_end_matches(['/', ' ', '\t', '\n', '\r', '\u{c}']);
    out.push_str("<form><hr><label>");
    out.push_str(PROMPT);
    out.push_str("<input");
    out.push_str(attrs);
    out.push_str(" name=\"isindex\"></label><hr></form>");
}

/// copies a rawtext element's start tag, content and matching end tag verbatim.
fn copy_raw_text(out: &mut String, start_tag: &str, start_tag_len: usize) {
    out.push_str(&start_tag[..start_tag_len]);
    let body = &start_tag[start_tag_len..];
    let name_len = tag_name_len(&start_tag[1..]).unwrap_or(0);
    let name = &start_tag[1..1 + name_len];
    let mut from = 0;
    while let Some(offset) = body[from..].find("</") {
        let at = from + offset;
        let Some(candidate_len) = tag_name_len(&body[at + 2..]) else {
            from = at + 2;
            continue;
        };
        if name.eq_ignore_ascii_case(&body[at + 2..at + 2 + candidate_len]) {
            let end = at + 2 + tag_len(&body[at + 2..]);
            out.push_str(&body[..end]);
            return;
        }
        from = at + 2;
    }
    out.push_str(body);
}

/// length of a tag name starting at `name`; first byte must be a letter.
fn tag_name_len(name: &str) -> Option<usize> {
    let bytes = name.as_bytes();
    if !bytes.first().copied()?.is_ascii_alphabetic() {
        return None;
    }
    Some(
        bytes[1..]
            .iter()
            .position(|b| b.is_ascii_whitespace() || matches!(b, b'/' | b'>'))
            .map_or(bytes.len(), |i| i + 1),
    )
}

/// distance from `body` start to just past the terminating `>` (quote aware).
fn attr_scan_end(body: &str) -> usize {
    let mut quote = 0u8;
    for (i, b) in body.as_bytes().iter().enumerate() {
        match quote {
            0 if *b == b'>' => return i + 1,
            0 if *b == b'"' || *b == b'\'' => quote = *b,
            q if q == *b => quote = 0,
            _ => {}
        }
    }
    body.len()
}

fn tag_len(after: &str) -> usize {
    attr_scan_end(after)
}

fn bogus_len(after: &str) -> usize {
    after.find('>').map_or(after.len(), |i| i + 1)
}

fn comment_len(after: &str) -> usize {
    after[4..].find("-->").map_or(after.len(), |i| i + 7)
}

/// minimal attribute iterator over a start-tag body (no value decoding).
struct IterAttrs<'a> {
    rest: &'a str,
}

impl<'a> IterAttrs<'a> {
    fn new(body: &'a str) -> Self {
        Self { rest: body }
    }
}

impl<'a> Iterator for IterAttrs<'a> {
    type Item = (&'a str, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        let rest = self
            .rest
            .trim_start_matches([' ', '\t', '\n', '\r', '\u{c}', '/']);
        if rest.is_empty() || rest.starts_with('>') {
            return None;
        }
        let name_end = rest
            .find(|c: char| c.is_ascii_whitespace() || matches!(c, '=' | '>' | '/'))
            .unwrap_or(rest.len());
        let (name, tail) = rest.split_at(name_end);
        let tail = tail.trim_start_matches([' ', '\t', '\n', '\r', '\u{c}']);
        let value = match tail.strip_prefix('=') {
            Some(after_eq) => {
                let after_eq = after_eq.trim_start_matches([' ', '\t', '\n', '\r', '\u{c}']);
                match after_eq.chars().next() {
                    Some(q @ ('"' | '\'')) => {
                        let len = after_eq[1..].find(q).map_or(after_eq.len() - 1, |i| i + 2);
                        self.rest = &after_eq[len..];
                        &after_eq[1..len - 1]
                    }
                    _ => {
                        let len = after_eq
                            .find(|c: char| c.is_ascii_whitespace() || c == '>')
                            .unwrap_or(after_eq.len());
                        self.rest = &after_eq[len..];
                        &after_eq[..len]
                    }
                }
            }
            None => {
                self.rest = tail;
                ""
            }
        };
        Some((name, value))
    }
}
