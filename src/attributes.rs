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
