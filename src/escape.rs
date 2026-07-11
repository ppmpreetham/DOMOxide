//! browser `innerHTML` escaping

/// elements whose text children are emitted verbatim instead of escaped.
pub fn is_raw_text(tag_lower: &str) -> bool {
    crate::lists::RAW_TEXT_SERIALIZE.contains(&tag_lower)
}

/// escapes text content: `&`, `<`, `>` and nbsp.
pub fn escape_text(input: &str, out: &mut String) {
    escape(input, out, Escape::Text)
}

/// escapes attribute values: `&`, `"` and nbsp.
pub fn escape_attribute(input: &str, out: &mut String) {
    escape(input, out, Escape::Attribute)
}

#[derive(Clone, Copy)]
enum Escape {
    Text,
    Attribute,
}

/// single-pass escaper that copies untouched slices verbatim.
fn escape(input: &str, out: &mut String, mode: Escape) {
    let mut chunk = 0;
    for (i, c) in input.char_indices() {
        let entity = match (mode, c) {
            (_, '&') => "&amp;",
            (_, '\u{a0}') => "&nbsp;",
            (Escape::Text, '<') => "&lt;",
            (Escape::Text, '>') => "&gt;",
            (Escape::Attribute, '"') => "&quot;",
            _ => continue,
        };
        out.push_str(&input[chunk..i]);
        out.push_str(entity);
        chunk = i + c.len_utf8();
    }
    out.push_str(&input[chunk..]);
}

/// javascript `String.prototype.trim` semantics (`\u{feff}` is whitespace there).
pub fn js_trim(input: &str) -> &str {
    input.trim_matches(|c: char| c.is_whitespace() || c == '\u{feff}')
}

/// true when `data` contains `<` followed by `/`, a word character or `!`
/// (dompurify's `/<[/\w!]/g` mXss probe)!!!!!!!!!!
pub fn has_mxss_marker(data: &str) -> bool {
    let bytes = data.as_bytes();
    bytes.windows(2).any(|w| {
        w[0] == b'<'
            && (w[1] == b'/' || w[1] == b'!' || w[1].is_ascii_alphanumeric() || w[1] == b'_')
    })
}

pub fn has_noscript_breakout(data: &str) -> bool {
    let lower = data.to_ascii_lowercase();
    ["</noscript", "</noembed", "</noframes"]
        .iter()
        .any(|marker| lower.contains(marker))
}
