//! Mixed `string`/`number` tolerant readers shared by every decoder.

use serde_json::Value;

pub(crate) fn as_u64(value: Option<&Value>) -> u64 {
    value
        .and_then(|v| {
            v.as_u64()
                .or_else(|| v.as_str().and_then(|s| s.parse::<u64>().ok()))
                .or_else(|| v.as_f64().map(|f| f.max(0.0) as u64))
        })
        .unwrap_or(0)
}

pub(crate) fn as_f64(value: Option<&Value>) -> f64 {
    value
        .and_then(|v| {
            v.as_f64()
                .or_else(|| v.as_str().and_then(|s| s.parse::<f64>().ok()))
        })
        .unwrap_or(0.0)
}

pub(crate) fn str_of<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("")
}

pub(crate) fn rows<'a>(body: &'a Value, key: &str) -> &'a [Value] {
    body.get(key)
        .and_then(Value::as_array)
        .map(|v| v.as_slice())
        .unwrap_or(&[])
}
