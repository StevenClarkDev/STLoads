#[cfg(target_arch = "wasm32")]
fn read_string_property(property: &str) -> Option<String> {
    use wasm_bindgen::JsValue;

    let window = web_sys::window()?;
    let config = js_sys::Reflect::get(&window, &JsValue::from_str("__STLOADS_CONFIG")).ok()?;
    if config.is_undefined() || config.is_null() {
        return None;
    }

    js_sys::Reflect::get(&config, &JsValue::from_str(property))
        .ok()
        .and_then(|value| value.as_string())
        .map(|value| value.trim().trim_end_matches('/').to_string())
        .filter(|value| !value.is_empty())
}

#[cfg(not(target_arch = "wasm32"))]
fn read_string_property(_property: &str) -> Option<String> {
    None
}

pub fn backend_api_base_url() -> Option<String> {
    read_string_property("backendApiBaseUrl")
}

pub fn google_maps_api_key() -> Option<String> {
    read_string_property("googleMapsApiKey")
}
