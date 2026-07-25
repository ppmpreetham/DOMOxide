//! domoxide: a rust html sanitizer shaped around dompurify compatibility.
//!
//! remove those dirty lil XSS idiots outta your codebase!

mod attributes;
mod config;
mod dom;
mod escape;
mod hooks;
pub(crate) mod lists;
mod policy;
mod preprocess;
mod sanitize;
mod uri;

#[cfg(feature = "wasm")]
mod wasm;

pub use config::{Config, CustomElementHandling, UseProfiles};
pub use hooks::{AttributeHook, ElementHook, HookAction, Hooks};

/// sanitizes untrusted `dirty` html with default settings.
pub fn sanitize(dirty: &str) -> String {
    Config::default().clean(dirty)
}
