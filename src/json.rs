//! Zero-dependency JSON parser and serializer (std-only).
//!
//! Provides a recursive descent parser conforming to RFC 8259 for parsing configuration
//! policies, near-field scan data, and formatting SARIF / JSON compliance reports.

#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>),
}

impl JsonValue {
    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        match self {
            Self::Object(pairs) => pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<JsonValue>> {
        match self {
            Self::Array(arr) => Some(arr),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&Vec<(String, JsonValue)>> {
        match self {
            Self::Object(obj) => Some(obj),
            _ => None,
        }
    }

    pub fn to_map(&self) -> Option<HashMap<String, JsonValue>> {
        match self {
            Self::Object(pairs) => {
                let mut map = HashMap::new();
                for (k, v) in pairs {
                    map.insert(k.clone(), v.clone());
                }
                Some(map)
            }
            _ => None,
        }
    }

    pub fn to_pretty_string(&self) -> String {
        let mut out = String::new();
        self.format_pretty(&mut out, 0);
        out
    }

    pub fn to_compact_string(&self) -> String {
        let mut out = String::new();
        self.format_compact(&mut out);
        out
    }

    fn format_compact(&self, out: &mut String) {
        match self {
            Self::Null => out.push_str("null"),
            Self::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
            Self::Number(n) => {
                if n.fract() == 0.0 && n.abs() < 1e15 {
                    out.push_str(&format!("{:.0}", n));
                } else {
                    out.push_str(&format!("{}", n));
                }
            }
            Self::String(s) => {
                out.push('"');
                for c in s.chars() {
                    match c {
                        '"' => out.push_str("\\\""),
                        '\\' => out.push_str("\\\\"),
                        '\n' => out.push_str("\\n"),
                        '\r' => out.push_str("\\r"),
                        '\t' => out.push_str("\\t"),
                        c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
                        c => out.push(c),
                    }
                }
                out.push('"');
            }
            Self::Array(arr) => {
                out.push('[');
                for (i, val) in arr.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    val.format_compact(out);
                }
                out.push(']');
            }
            Self::Object(pairs) => {
                out.push('{');
                for (i, (k, val)) in pairs.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    out.push('"');
                    out.push_str(k);
                    out.push_str("\":");
                    val.format_compact(out);
                }
                out.push('}');
            }
        }
    }

    fn format_pretty(&self, out: &mut String, indent: usize) {
        let pad = "  ".repeat(indent);
        let inner_pad = "  ".repeat(indent + 1);

        match self {
            Self::Null => out.push_str("null"),
            Self::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
            Self::Number(n) => {
                if n.fract() == 0.0 && n.abs() < 1e15 {
                    out.push_str(&format!("{:.0}", n));
                } else {
                    out.push_str(&format!("{}", n));
                }
            }
            Self::String(s) => {
                out.push('"');
                for c in s.chars() {
                    match c {
                        '"' => out.push_str("\\\""),
                        '\\' => out.push_str("\\\\"),
                        '\n' => out.push_str("\\n"),
                        '\r' => out.push_str("\\r"),
                        '\t' => out.push_str("\\t"),
                        c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
                        c => out.push(c),
                    }
                }
                out.push('"');
            }
            Self::Array(arr) => {
                if arr.is_empty() {
                    out.push_str("[]");
                    return;
                }
                out.push_str("[\n");
                for (i, val) in arr.iter().enumerate() {
                    out.push_str(&inner_pad);
                    val.format_pretty(out, indent + 1);
                    if i + 1 < arr.len() {
                        out.push(',');
                    }
                    out.push('\n');
                }
                out.push_str(&pad);
                out.push(']');
            }
            Self::Object(pairs) => {
                if pairs.is_empty() {
                    out.push_str("{}");
                    return;
                }
                out.push_str("{\n");
                for (i, (k, val)) in pairs.iter().enumerate() {
                    out.push_str(&inner_pad);
                    out.push('"');
                    out.push_str(k);
                    out.push_str("\": ");
                    val.format_pretty(out, indent + 1);
                    if i + 1 < pairs.len() {
                        out.push(',');
                    }
                    out.push('\n');
                }
                out.push_str(&pad);
                out.push('}');
            }
        }
    }
}

impl fmt::Display for JsonValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_pretty_string())
    }
}

pub struct JsonParser<'a> {
    chars: Vec<char>,
    pos: usize,
    _marker: std::marker::PhantomData<&'a str>,
}

impl<'a> JsonParser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn parse(input: &'a str) -> Result<JsonValue, String> {
        let mut parser = Self::new(input);
        parser.skip_whitespace();
        let val = parser.parse_value()?;
        parser.skip_whitespace();
        if parser.pos < parser.chars.len() {
            return Err(format!(
                "Unexpected trailing characters at pos {}: '{}'",
                parser.pos, parser.chars[parser.pos]
            ));
        }
        Ok(val)
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.chars.len() && self.chars[self.pos].is_whitespace() {
            self.pos += 1;
        }
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        if self.pos >= self.chars.len() {
            return Err("Unexpected end of input while parsing JSON value".to_string());
        }

        match self.chars[self.pos] {
            'n' => self.parse_null(),
            't' | 'f' => self.parse_bool(),
            '"' => self.parse_string().map(JsonValue::String),
            '[' => self.parse_array(),
            '{' => self.parse_object(),
            '-' | '0'..='9' => self.parse_number(),
            ch => Err(format!("Unexpected character '{}' at pos {}", ch, self.pos)),
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, String> {
        if self.pos + 4 <= self.chars.len()
            && self.chars[self.pos..self.pos + 4] == ['n', 'u', 'l', 'l']
        {
            self.pos += 4;
            Ok(JsonValue::Null)
        } else {
            Err(format!("Expected 'null' at pos {}", self.pos))
        }
    }

    fn parse_bool(&mut self) -> Result<JsonValue, String> {
        if self.pos + 4 <= self.chars.len()
            && self.chars[self.pos..self.pos + 4] == ['t', 'r', 'u', 'e']
        {
            self.pos += 4;
            Ok(JsonValue::Bool(true))
        } else if self.pos + 5 <= self.chars.len()
            && self.chars[self.pos..self.pos + 5] == ['f', 'a', 'l', 's', 'e']
        {
            self.pos += 5;
            Ok(JsonValue::Bool(false))
        } else {
            Err(format!("Expected boolean at pos {}", self.pos))
        }
    }

    fn parse_string(&mut self) -> Result<String, String> {
        if self.pos >= self.chars.len() || self.chars[self.pos] != '"' {
            return Err(format!("Expected '\"' at pos {}", self.pos));
        }
        self.pos += 1;
        let mut s = String::new();

        while self.pos < self.chars.len() {
            match self.chars[self.pos] {
                '"' => {
                    self.pos += 1;
                    return Ok(s);
                }
                '\\' => {
                    self.pos += 1;
                    if self.pos >= self.chars.len() {
                        return Err("Unterminated escape sequence in string".to_string());
                    }
                    match self.chars[self.pos] {
                        '"' => s.push('"'),
                        '\\' => s.push('\\'),
                        '/' => s.push('/'),
                        'b' => s.push('\x08'),
                        'f' => s.push('\x0c'),
                        'n' => s.push('\n'),
                        'r' => s.push('\r'),
                        't' => s.push('\t'),
                        'u' => {
                            if self.pos + 4 >= self.chars.len() {
                                return Err("Truncated unicode escape in string".to_string());
                            }
                            let hex_str: String =
                                self.chars[self.pos + 1..=self.pos + 4].iter().collect();
                            match u32::from_str_radix(&hex_str, 16) {
                                Ok(code) => {
                                    if let Some(ch) = char::from_u32(code) {
                                        s.push(ch);
                                    } else {
                                        s.push('?');
                                    }
                                }
                                Err(_) => {
                                    return Err(format!(
                                        "Invalid hex escape '{}' at pos {}",
                                        hex_str, self.pos
                                    ))
                                }
                            }
                            self.pos += 4;
                        }
                        other => s.push(other),
                    }
                    self.pos += 1;
                }
                ch => {
                    s.push(ch);
                    self.pos += 1;
                }
            }
        }
        Err("Unterminated string".to_string())
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.pos;
        if self.chars[self.pos] == '-' {
            self.pos += 1;
        }
        while self.pos < self.chars.len() && self.chars[self.pos].is_ascii_digit() {
            self.pos += 1;
        }
        if self.pos < self.chars.len() && self.chars[self.pos] == '.' {
            self.pos += 1;
            while self.pos < self.chars.len() && self.chars[self.pos].is_ascii_digit() {
                self.pos += 1;
            }
        }
        if self.pos < self.chars.len()
            && (self.chars[self.pos] == 'e' || self.chars[self.pos] == 'E')
        {
            self.pos += 1;
            if self.pos < self.chars.len()
                && (self.chars[self.pos] == '+' || self.chars[self.pos] == '-')
            {
                self.pos += 1;
            }
            while self.pos < self.chars.len() && self.chars[self.pos].is_ascii_digit() {
                self.pos += 1;
            }
        }

        let num_str: String = self.chars[start..self.pos].iter().collect();
        match num_str.parse::<f64>() {
            Ok(n) => Ok(JsonValue::Number(n)),
            Err(e) => Err(format!(
                "Failed to parse number '{}' at pos {}: {}",
                num_str, start, e
            )),
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.pos += 1; // skip '['
        self.skip_whitespace();
        let mut items = Vec::new();

        if self.pos < self.chars.len() && self.chars[self.pos] == ']' {
            self.pos += 1;
            return Ok(JsonValue::Array(items));
        }

        loop {
            self.skip_whitespace();
            let item = self.parse_value()?;
            items.push(item);
            self.skip_whitespace();

            if self.pos >= self.chars.len() {
                return Err("Unterminated array".to_string());
            }

            if self.chars[self.pos] == ']' {
                self.pos += 1;
                break;
            } else if self.chars[self.pos] == ',' {
                self.pos += 1;
            } else {
                return Err(format!(
                    "Expected ',' or ']' in array at pos {}, found '{}'",
                    self.pos, self.chars[self.pos]
                ));
            }
        }

        Ok(JsonValue::Array(items))
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.pos += 1; // skip '{'
        self.skip_whitespace();
        let mut pairs = Vec::new();

        if self.pos < self.chars.len() && self.chars[self.pos] == '}' {
            self.pos += 1;
            return Ok(JsonValue::Object(pairs));
        }

        loop {
            self.skip_whitespace();
            let key = self.parse_string()?;
            self.skip_whitespace();

            if self.pos >= self.chars.len() || self.chars[self.pos] != ':' {
                return Err(format!(
                    "Expected ':' after key '{}' at pos {}",
                    key, self.pos
                ));
            }
            self.pos += 1; // skip ':'

            let value = self.parse_value()?;
            pairs.push((key, value));
            self.skip_whitespace();

            if self.pos >= self.chars.len() {
                return Err("Unterminated object".to_string());
            }

            if self.chars[self.pos] == '}' {
                self.pos += 1;
                break;
            } else if self.chars[self.pos] == ',' {
                self.pos += 1;
            } else {
                return Err(format!(
                    "Expected ',' or '}}' in object at pos {}, found '{}'",
                    self.pos, self.chars[self.pos]
                ));
            }
        }

        Ok(JsonValue::Object(pairs))
    }
}
