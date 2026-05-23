use crate::ports::url_fragment::UrlFragmentPort;

#[derive(Clone, Copy, Default)]
pub(super) struct WasmUrlNavigator;

fn href_without_fragment() -> Result<String, String> {
    let window = web_sys::window().ok_or("no window")?;
    let href = window
        .location()
        .href()
        .map_err(|_| "href unavailable")?;
    Ok(match href.split_once('#') {
        Some((before, _)) => before.to_string(),
        None => href,
    })
}

impl UrlFragmentPort for WasmUrlNavigator {
    fn current_fragment_body(&self) -> Option<String> {
        let window = web_sys::window()?;
        let hash = window.location().hash().ok()?;
        if hash.len() <= 1 {
            return None;
        }
        Some(hash.strip_prefix('#')?.trim().to_string())
    }

    fn replace_fragment_body(&self, fragment_body: &str) -> Result<(), String> {
        let window = web_sys::window().ok_or("no window")?;
        let loc = window.location();
        let base = href_without_fragment()?;
        let new_url = format!("{base}#{fragment_body}");
        let hist = window.history().map_err(|_| "history unavailable")?;
        if hist
            .replace_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(&new_url))
            .is_err()
        {
            loc.set_hash(fragment_body)
                .map_err(|_| "set_hash failed")?;
        }
        Ok(())
    }

    fn fragment_equals(&self, fragment_body: &str) -> bool {
        let Some(current) = self.current_fragment_body() else {
            return false;
        };
        current.starts_with("v=") && current.as_str() == fragment_body
    }
}
