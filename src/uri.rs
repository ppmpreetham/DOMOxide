use crate::lists::{DATA_URI_TAGS, URI_SAFE_ATTRS};

/// accepted by `IS_ALLOWED_URI` before the `:` separator.
const SCHEMES: &[&str] = &[
    "ftp", "ftps", "http", "https", "mailto", "tel", "callto", "sms", "cid", "xmpp", "matrix",
];

/// characters stripped before scheme test
fn is_attr_whitespace(c: char) -> bool {
    matches!(
        c,
        '\u{0}'..='\u{20}'
            | '\u{a0}'
            | '\u{1680}'
            | '\u{180e}'
            | '\u{2000}'..='\u{2029}'
            | '\u{205f}'
            | '\u{3000}'
    )
}

pub fn is_uri_safe_attribute(attr_lower: &str) -> bool {
    URI_SAFE_ATTRS.contains(&attr_lower)
}

/// `/^(?:(?:(?:f|ht)tps?|mailto|tel|callto|sms|cid|xmpp|matrix):|[^a-z]|[a-z+.\-]+(?:[^a-z+.\-:]|$))/i`
/// evaluated against `value` with attribute whitespace removed, `scratch` is
/// reused storage so the common case never allocates.
pub fn is_allowed_uri(value: &str, scratch: &mut String) -> bool {
    let check: &str = if value.chars().any(is_attr_whitespace) {
        scratch.clear();
        scratch.extend(value.chars().filter(|c| !is_attr_whitespace(*c)));
        scratch.as_str()
    } else {
        value
    };
    match_allowed_uri(check)
}

/// anchored evaluation of the three regex alternatives against clean text.
fn match_allowed_uri(check: &str) -> bool {
    if starts_with_scheme(check) {
        return true;
    }
    let mut chars = check.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_alphabetic() {
        return true; // `[^a-z]` alternative
    }
    // `[a-z+.\-]+` run must end at the string end or on a char outside
    // `[a-z+.\-:]` for the third alternative to match.
    for c in chars {
        if c.is_ascii_alphabetic() || matches!(c, '+' | '.' | '-') {
            continue;
        }
        return c != ':';
    }
    true
}

/// `(?:f|ht)tps?|mailto|tel|callto|sms|cid|xmpp|matrix` followed by `:`.
fn starts_with_scheme(check: &str) -> bool {
    SCHEMES.iter().any(|scheme| {
        check.len() > scheme.len()
            && check[..scheme.len()].eq_ignore_ascii_case(scheme)
            && check.as_bytes()[scheme.len()] == b':'
    })
}

/// `data:`: allowed element : `src`/`href`/`xlink:href`
pub fn is_allowed_data_uri(tag_lower: &str, attr_lower: &str, trimmed_value: &str) -> bool {
    matches!(attr_lower, "src" | "href" | "xlink:href")
        && tag_lower != "script"
        && trimmed_value.starts_with("data:")
        && DATA_URI_TAGS.contains(&tag_lower)
}
