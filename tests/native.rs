use domoxide::{Config, sanitize};
use pretty_assertions::assert_eq;

#[test]
fn removes_script_and_event_handlers() {
    let clean = sanitize(r#"<p onclick="evil()">ok</p><script>alert(1)</script>"#);

    assert_eq!(clean, "<p>ok</p>");
}

#[test]
fn removes_javascript_urls() {
    let clean = sanitize(r#"<a href="javascript:alert(1)">click</a>"#);

    assert_eq!(clean, "<a>click</a>");
}

#[test]
fn supports_add_and_forbid_options() {
    let clean = Config::html_profile()
        .add_tags(["custom-card"])
        .add_attr(["data-id"])
        .forbid_tags(["img"])
        .clean(r#"<custom-card data-id="7"><img src="x">ok</custom-card>"#);

    assert_eq!(clean, r#"<custom-card data-id="7">ok</custom-card>"#);
}
