#![cfg(target_arch = "wasm32")]

use domoxide::{Config, sanitize};
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn wasm_sanitizes_basic_xss() {
    let clean = sanitize(r#"<img src=x onerror=alert(1)><script>alert(1)</script>"#);

    assert_eq!(clean, r#"<img src="x">"#);
}

#[wasm_bindgen_test]
fn wasm_supports_dompurify_style_config() {
    let clean = Config::html_profile()
        .add_tags(["custom-card"])
        .add_attr(["data-id"])
        .clean(r#"<custom-card data-id="42" onclick="evil()">ok</custom-card>"#);

    assert_eq!(clean, r#"<custom-card data-id="42">ok</custom-card>"#);
}
