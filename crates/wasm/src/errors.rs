use js_sys::{Object, Reflect};
use wasm_bindgen::JsValue;

pub(crate) fn result_to_js_result(result: eyre::Result<JsValue>) -> JsValue {
    match result {
        Ok(value) => js_result(value, JsValue::UNDEFINED),
        Err(error) => js_result(JsValue::UNDEFINED, JsValue::from_str(&error.to_string())),
    }
}

fn js_result(value: JsValue, error: JsValue) -> JsValue {
    let out = Object::new();
    Reflect::set(&out, &JsValue::from_str("value"), &value)
        .expect("setting value on a new object should not fail");
    Reflect::set(&out, &JsValue::from_str("error"), &error)
        .expect("setting error on a new object should not fail");
    out.into()
}
