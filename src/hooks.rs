use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HookAction {
    Continue,
    ForceRemove,
}

pub type ElementHook = dyn Fn(&str, &mut HashMap<String, String>) -> HookAction;
pub type AttributeHook = dyn Fn(&str, &str, &mut String) -> HookAction;

#[derive(Default)]
pub struct Hooks {
    pub before_sanitize_elements: Option<Box<ElementHook>>,
    pub upon_sanitize_element: Option<Box<ElementHook>>,
    pub after_sanitize_elements: Option<Box<ElementHook>>,
    pub before_sanitize_attributes: Option<Box<AttributeHook>>,
    pub upon_sanitize_attribute: Option<Box<AttributeHook>>,
}
