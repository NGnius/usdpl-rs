use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern {
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    pub fn console_log(s: &str);

    #[wasm_bindgen(js_namespace = console, js_name = warn)]
    pub fn console_warn(s: &str);

    #[wasm_bindgen(js_namespace = console, js_name = error)]
    pub fn console_error(s: &str);
}
