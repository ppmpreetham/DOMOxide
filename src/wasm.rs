use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn sanitize_wasm(dirty: &str, config: JsValue) -> Result<String, JsValue> {
    let parsed = if config.is_null() || config.is_undefined() {
        crate::Config::default()
    } else {
        serde_wasm_bindgen::from_value(config)
            .map_err(|error| JsValue::from_str(&error.to_string()))?
    };
    Ok(parsed.clean(dirty))
}
