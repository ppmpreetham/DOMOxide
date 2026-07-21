//! attribute sanitization: a faithful port of dompurify's
//! `_sanitizeAttributes` and `_isValidAttribute`.

use crate::escape::js_trim;
use crate::hooks::{AttributeHook, HookAction, Hooks};
use crate::lists as l;
use crate::policy::Policy;
use crate::uri;

pub struct SanitizedAttr {
    pub name: String,
    pub value: String,
}

pub(crate) fn sanitize(
    policy: &Policy,
    hooks: &Hooks,
    tag_lower: &str,
    attrs: Vec<crate::sanitize::ParsedAttr>,
    scratch: &mut String,
) -> Vec<SanitizedAttr> {
    let mut kept = Vec::with_capacity(attrs.len());
    for crate::sanitize::ParsedAttr {
        name,
        value: raw_value,
        ..
    } in attrs
    {
        let name_lower = name.to_lowercase();
        let mut value = if name == "value" {
            raw_value.clone()
        } else {
            js_trim(&raw_value).to_owned()
        };

        if run_attr_hooks(hooks, tag_lower, &name_lower, &mut value)
            || policy.safe_for_xml && has_comment_breakout(&value)
            || name_lower == "attributename" && value.contains("href")
            || !is_valid_attribute(policy, tag_lower, &name_lower, &value, scratch)
        {
            if name_lower == "is" {
                kept.push(SanitizedAttr {
                    name,
                    value: String::new(),
                });
            }
            continue;
        }
        kept.push(SanitizedAttr { name, value });
    }
    kept
}

fn run_attr_hooks(hooks: &Hooks, tag: &str, attr: &str, value: &mut String) -> bool {
    let mut removed = |hook: Option<&Box<AttributeHook>>| {
        hook.is_some_and(|hook| hook(tag, attr, value) == HookAction::ForceRemove)
    };
    removed(hooks.before_sanitize_attributes.as_ref())
        || removed(hooks.upon_sanitize_attribute.as_ref())
}
fn has_comment_breakout(value: &str) -> bool {
    const TAGS: &[&str] = &[
        "style", "script", "title", "xmp", "textarea", "noscript", "iframe", "noembed", "noframes",
    ];
    let bytes = value.as_bytes();
    (0..bytes.len()).any(|i| match bytes[i] {
        b'-' | b']' => {
            value[i..].starts_with("-->")
                || value[i..].starts_with("--!>")
                || value[i..].starts_with("]>")
        }
        b'<' if bytes.get(i + 1) == Some(&b'/') => TAGS.iter().any(|tag| {
            value[i + 2..].len() >= tag.len()
                && value[i + 2..i + 2 + tag.len()].eq_ignore_ascii_case(tag)
        }),
        _ => false,
    })
}

fn is_valid_attribute(
    policy: &Policy,
    tag_lower: &str,
    name_lower: &str,
    value: &str,
    scratch: &mut String,
) -> bool {
    if policy.forbids_attr(name_lower) {
        return false;
    }
    if policy.sanitize_dom && (name_lower == "id" || name_lower == "name") && clobbers(value) {
        return false;
    }
    if policy.allow_data_attr && is_data_attr(name_lower)
        || policy.allow_aria_attr && is_aria_attr(name_lower)
    {
        return true;
    }
    if !policy.allows_attr(name_lower) {
        return allowed_via_custom_elements(policy, tag_lower, name_lower, value);
    }
    uri::is_uri_safe_attribute(name_lower)
        || uri::is_allowed_uri(value, scratch)
        || uri::is_allowed_data_uri(tag_lower, name_lower, value)
        || value.is_empty()
}

/// `/^data-[\-\w.\u{b7}-\u{10ffff}]+$/`
