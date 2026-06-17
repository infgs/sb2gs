use std::collections::HashMap;
use std::sync::OnceLock;

use regex::Regex;

use crate::signatures::is_signature_name;
use serde_json::Value;

pub struct Identifiers {
    identifiers: HashMap<String, String>,
}

impl Default for Identifiers {
    fn default() -> Self {
        Self::new()
    }
}

impl Identifiers {
    pub fn new() -> Self {
        Self {
            identifiers: HashMap::new(),
        }
    }

    pub fn identifier(&mut self, original: &str) -> String {
        if let Some(identifier) = self.identifiers.get(original) {
            return identifier.clone();
        }
        let mut identifier = whitespace_re().replace_all(original, "_").to_string();
        identifier = invalid_chars_re().replace_all(&identifier, "").to_string();
        identifier = identifier.trim_matches('_').to_lowercase();
        if is_keyword(&identifier) || is_signature_name(&identifier) {
            identifier.push('_');
        }
        if identifier.is_empty() || identifier.as_bytes()[0].is_ascii_digit() {
            identifier.insert(0, '_');
        }
        let base = identifier.clone();
        let mut index = 2;
        while self.identifiers.values().any(|used| used == &identifier) {
            identifier = format!("{base}{index}");
            index += 1;
        }
        self.identifiers
            .insert(original.to_string(), identifier.clone());
        identifier
    }
}

fn whitespace_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"[\s.\-:]+").expect("valid whitespace regex"))
}

fn invalid_chars_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"[^a-zA-Z_0-9]").expect("valid invalid char regex"))
}

fn is_keyword(value: &str) -> bool {
    matches!(
        value,
        "costumes"
            | "sounds"
            | "local"
            | "proc"
            | "func"
            | "return"
            | "nowarp"
            | "on"
            | "onflag"
            | "onkey"
            | "onclick"
            | "onbackdrop"
            | "onloudness"
            | "ontimer"
            | "onclone"
            | "if"
            | "else"
            | "elif"
            | "until"
            | "forever"
            | "repeat"
            | "not"
            | "and"
            | "or"
            | "in"
            | "length"
            | "round"
            | "abs"
            | "floor"
            | "ceil"
            | "sqrt"
            | "sin"
            | "cos"
            | "tan"
            | "asin"
            | "acos"
            | "atan"
            | "ln"
            | "log"
            | "antiln"
            | "antilog"
            | "show"
            | "hide"
            | "add"
            | "to"
            | "delete"
            | "insert"
            | "at"
            | "of"
            | "as"
            | "enum"
            | "struct"
            | "true"
            | "false"
            | "list"
            | "cloud"
            | "set_x"
            | "set_y"
            | "set_size"
            | "point_in_direction"
            | "set_volume"
            | "set_rotation_style_left_right"
            | "set_rotation_style_all_around"
            | "set_rotation_style_do_not_rotate"
            | "var"
    )
}

pub(crate) fn string(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        write_json_char(&mut out, ch);
    }
    out.push('"');
    out
}

fn write_json_char(out: &mut String, ch: char) {
    match ch {
        '"' => out.push_str("\\\""),
        '\\' => out.push_str("\\\\"),
        '\x08' => out.push_str("\\b"),
        '\x0c' => out.push_str("\\f"),
        '\n' => out.push_str("\\n"),
        '\r' => out.push_str("\\r"),
        '\t' => out.push_str("\\t"),
        ch if ch < ' ' => out.push_str(&format!("\\u{:04x}", ch as u32)),
        ch if ch.is_ascii() => out.push(ch),
        ch => write_non_ascii_json_char(out, ch),
    }
}

fn write_non_ascii_json_char(out: &mut String, ch: char) {
    let code = ch as u32;
    if code <= 0xffff {
        out.push_str(&format!("\\u{code:04x}"));
        return;
    }
    let code = code - 0x10000;
    let high = 0xd800 + ((code >> 10) & 0x3ff);
    let low = 0xdc00 + (code & 0x3ff);
    out.push_str(&format!("\\u{high:04x}\\u{low:04x}"));
}

pub(crate) fn number(value: &Value) -> String {
    serde_json::to_string(value).expect("number serializes")
}

pub(crate) fn value(value: &Value) -> String {
    if value.is_number() {
        return number(value);
    }
    let text = value
        .as_str()
        .unwrap_or_else(|| panic!("expected string or number, got {value:?}"));
    if is_goboscript_literal(text) {
        return text.to_string();
    }
    string(text)
}

fn is_goboscript_literal(text: &str) -> bool {
    let Ok(parsed) = serde_json::from_str::<Value>(text) else {
        return false;
    };
    parsed.is_number() && serde_json::to_string(&parsed).is_ok_and(|serialized| serialized == text)
}

pub fn toml_string(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch < ' ' => out.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => out.push(ch),
        }
    }
    out.push('"');
    out
}
