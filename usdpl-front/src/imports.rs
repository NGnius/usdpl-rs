use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[cfg(feature = "debug")]
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    pub fn console_log(s: &str);

    #[cfg(feature = "debug")]
    #[wasm_bindgen(js_namespace = console, js_name = warn)]
    pub fn console_warn(s: &str);

    #[cfg(feature = "debug")]
    #[wasm_bindgen(js_namespace = console, js_name = error)]
    pub fn console_error(s: &str);
}
