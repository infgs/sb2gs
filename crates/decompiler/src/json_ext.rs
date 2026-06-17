use serde_json::{Map, Value};

pub fn object(value: &Value) -> &Map<String, Value> {
    value
        .as_object()
        .unwrap_or_else(|| panic!("expected object, got {value:?}"))
}

pub fn object_mut(value: &mut Value) -> &mut Map<String, Value> {
    match value.as_object_mut() {
        Some(value) => value,
        None => panic!("expected object"),
    }
}

pub fn array(value: &Value) -> &[Value] {
    value
        .as_array()
        .unwrap_or_else(|| panic!("expected array, got {value:?}"))
}

pub fn string(value: &Value) -> &str {
    value
        .as_str()
        .unwrap_or_else(|| panic!("expected string, got {value:?}"))
}

pub fn key<'a>(value: &'a Value, name: &str) -> &'a Value {
    object(value)
        .get(name)
        .unwrap_or_else(|| panic!("missing key {name} in {value:?}"))
}

pub fn key_mut<'a>(value: &'a mut Value, name: &str) -> &'a mut Value {
    object_mut(value)
        .get_mut(name)
        .unwrap_or_else(|| panic!("missing key {name}"))
}

pub fn opt_key<'a>(value: &'a Value, name: &str) -> Option<&'a Value> {
    object(value).get(name)
}

pub fn string_key<'a>(value: &'a Value, name: &str) -> &'a str {
    string(key(value, name))
}

pub fn bool_key(value: &Value, name: &str) -> bool {
    key(value, name)
        .as_bool()
        .unwrap_or_else(|| panic!("expected bool key {name}"))
}

pub fn f64_key(value: &Value, name: &str) -> f64 {
    key(value, name)
        .as_f64()
        .unwrap_or_else(|| panic!("expected number key {name}"))
}

pub fn i64_key(value: &Value, name: &str) -> i64 {
    key(value, name)
        .as_i64()
        .unwrap_or_else(|| panic!("expected integer key {name}"))
}
