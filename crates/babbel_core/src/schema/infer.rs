//! Automatic JSON Schema (Draft 2020-12) inference from sample [`Value`] documents.
//!
//! Supports inferring scalar types, formats (`uuid`, `email`, `ipv4`, `date-time`, `uri`),
//! homogeneous and heterogeneous arrays, and merging object schemas across record arrays
//! (automatically distinguishing `required` fields present in all records from optional fields).

use crate::model::Value;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

/// Infers a Draft 2020-12 JSON Schema [`Value`] from a sample [`Value`] document.
pub fn infer_schema(sample: &Value) -> Value {
    let mut schema = infer_value_schema(sample);
    if let Value::Object(ref mut entries) = schema {
        entries.insert(
            0,
            (
                "$schema".to_string(),
                Value::String("https://json-schema.org/draft/2020-12/schema".to_string()),
            ),
        );
    }
    schema
}

fn infer_value_schema(val: &Value) -> Value {
    match val {
        Value::Null => Value::Object(vec![(
            "type".to_string(),
            Value::String("null".to_string()),
        )]),
        Value::Bool(_) => Value::Object(vec![(
            "type".to_string(),
            Value::String("boolean".to_string()),
        )]),
        Value::Integer(_) => Value::Object(vec![(
            "type".to_string(),
            Value::String("integer".to_string()),
        )]),
        Value::Float(_) => Value::Object(vec![(
            "type".to_string(),
            Value::String("number".to_string()),
        )]),
        Value::Bytes(_) => Value::Object(vec![
            ("type".to_string(), Value::String("string".to_string())),
            (
                "contentEncoding".to_string(),
                Value::String("base64".to_string()),
            ),
        ]),
        Value::String(s) => {
            let mut entries = vec![("type".to_string(), Value::String("string".to_string()))];
            if let Some(fmt) = detect_string_format(s) {
                entries.push(("format".to_string(), Value::String(fmt.to_string())));
            }
            Value::Object(entries)
        }
        Value::Array(items) => {
            if items.is_empty() {
                return Value::Object(vec![(
                    "type".to_string(),
                    Value::String("array".to_string()),
                )]);
            }

            // If all elements are objects, merge them into a unified record schema
            if items.iter().all(|i| i.is_object()) {
                let merged_item_schema = merge_object_samples(items);
                return Value::Object(vec![
                    ("type".to_string(), Value::String("array".to_string())),
                    ("items".to_string(), merged_item_schema),
                ]);
            }

            let mut item_schemas: Vec<Value> = Vec::new();
            for item in items {
                let s = infer_value_schema(item);
                if !item_schemas.contains(&s) {
                    item_schemas.push(s);
                }
            }

            // Normalize integer + number into number
            let has_int = item_schemas
                .iter()
                .any(|s| s.get("type").and_then(|t| t.as_str()) == Some("integer"));
            let has_num = item_schemas
                .iter()
                .any(|s| s.get("type").and_then(|t| t.as_str()) == Some("number"));
            if has_int && has_num {
                item_schemas.retain(|s| s.get("type").and_then(|t| t.as_str()) != Some("integer"));
            }

            let items_schema = if item_schemas.len() == 1 {
                item_schemas.remove(0)
            } else {
                Value::Object(vec![("anyOf".to_string(), Value::Array(item_schemas))])
            };

            Value::Object(vec![
                ("type".to_string(), Value::String("array".to_string())),
                ("items".to_string(), items_schema),
            ])
        }
        Value::Object(entries) => {
            let mut props = Vec::with_capacity(entries.len());
            let mut required = Vec::with_capacity(entries.len());
            for (k, v) in entries {
                props.push((k.clone(), infer_value_schema(v)));
                if !v.is_null() {
                    required.push(Value::String(k.clone()));
                }
            }
            let mut schema_entries = vec![
                ("type".to_string(), Value::String("object".to_string())),
                ("properties".to_string(), Value::Object(props)),
            ];
            if !required.is_empty() {
                schema_entries.push(("required".to_string(), Value::Array(required)));
            }
            Value::Object(schema_entries)
        }
    }
}

fn merge_object_samples(objects: &[Value]) -> Value {
    let total = objects.len();
    let mut keys_in_order: Vec<String> = Vec::new();
    for obj in objects {
        if let Value::Object(entries) = obj {
            for (k, _) in entries {
                if !keys_in_order.contains(k) {
                    keys_in_order.push(k.clone());
                }
            }
        }
    }

    let mut properties = Vec::with_capacity(keys_in_order.len());
    let mut required = Vec::new();

    for key in keys_in_order {
        let mut present_non_null_count = 0usize;
        let mut field_schemas: Vec<Value> = Vec::new();

        for obj in objects {
            if let Some(val) = obj.get(&key) {
                if !val.is_null() {
                    present_non_null_count += 1;
                    let s = infer_value_schema(val);
                    if !field_schemas.contains(&s) {
                        field_schemas.push(s);
                    }
                } else {
                    let null_s = Value::Object(vec![(
                        "type".to_string(),
                        Value::String("null".to_string()),
                    )]);
                    if !field_schemas.contains(&null_s) {
                        field_schemas.push(null_s);
                    }
                }
            }
        }

        // Normalize integer + number -> number
        let has_int = field_schemas
            .iter()
            .any(|s| s.get("type").and_then(|t| t.as_str()) == Some("integer"));
        let has_num = field_schemas
            .iter()
            .any(|s| s.get("type").and_then(|t| t.as_str()) == Some("number"));
        if has_int && has_num {
            field_schemas.retain(|s| s.get("type").and_then(|t| t.as_str()) != Some("integer"));
        }

        let prop_schema = if field_schemas.is_empty() {
            Value::Object(vec![(
                "type".to_string(),
                Value::String("null".to_string()),
            )])
        } else if field_schemas.len() == 1 {
            field_schemas.remove(0)
        } else {
            Value::Object(vec![("anyOf".to_string(), Value::Array(field_schemas))])
        };

        properties.push((key.clone(), prop_schema));
        if present_non_null_count == total {
            required.push(Value::String(key));
        }
    }

    let mut out = vec![
        ("type".to_string(), Value::String("object".to_string())),
        ("properties".to_string(), Value::Object(properties)),
    ];
    if !required.is_empty() {
        out.push(("required".to_string(), Value::Array(required)));
    }
    Value::Object(out)
}

fn detect_string_format(s: &str) -> Option<&'static str> {
    if is_uuid(s) {
        Some("uuid")
    } else if is_ipv4(s) {
        Some("ipv4")
    } else if is_date_time(s) {
        Some("date-time")
    } else if is_date(s) {
        Some("date")
    } else if s.starts_with("http://") || s.starts_with("https://") {
        Some("uri")
    } else if is_email(s) {
        Some("email")
    } else {
        None
    }
}

fn is_uuid(s: &str) -> bool {
    if s.len() != 36 {
        return false;
    }
    for (i, b) in s.bytes().enumerate() {
        if i == 8 || i == 13 || i == 18 || i == 23 {
            if b != b'-' {
                return false;
            }
        } else if !b.is_ascii_hexdigit() {
            return false;
        }
    }
    true
}

fn is_ipv4(s: &str) -> bool {
    let parts: Vec<&str> = s.split('.').collect();
    parts.len() == 4
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.parse::<u8>().is_ok())
}

fn is_date(s: &str) -> bool {
    if s.len() != 10 {
        return false;
    }
    let b = s.as_bytes();
    b[4] == b'-'
        && b[7] == b'-'
        && s[0..4].parse::<u16>().is_ok()
        && s[5..7]
            .parse::<u8>()
            .map_or(false, |m| (1..=12).contains(&m))
        && s[8..10]
            .parse::<u8>()
            .map_or(false, |d| (1..=31).contains(&d))
}

fn is_date_time(s: &str) -> bool {
    if s.len() < 19 {
        return false;
    }
    let b = s.as_bytes();
    is_date(&s[0..10])
        && (b[10] == b'T' || b[10] == b't' || b[10] == b' ')
        && b[13] == b':'
        && b[16] == b':'
}

fn is_email(s: &str) -> bool {
    if let Some((local, domain)) = s.split_once('@') {
        !local.is_empty()
            && !local.contains(' ')
            && domain.contains('.')
            && !domain.starts_with('.')
            && !domain.ends_with('.')
    } else {
        false
    }
}
