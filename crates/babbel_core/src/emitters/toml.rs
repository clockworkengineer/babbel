use crate::codec::FormatEmitter;
use crate::error::BabbelError;
use crate::io::traits::IDestination;
use crate::model::Value;

/// Standard built-in TOML format emitter delegating to universal Value serialization.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct TomlEmitter;

impl TomlEmitter {
    /// Serializes value to TOML representation into `dest`.
    pub fn emit_to_dest(&self, value: &Value, dest: &mut dyn IDestination) {
        match value {
            Value::Object(entries) => {
                serialize_toml_table(entries, "", dest);
            }
            Value::Array(items)
                if !items.is_empty() && items.iter().all(|x| matches!(x, Value::Object(_))) =>
            {
                for item in items {
                    dest.add_bytes("[[item]]\n");
                    if let Value::Object(entries) = item {
                        serialize_toml_table(entries, "item", dest);
                    }
                }
            }
            single => {
                dest.add_bytes("value = ");
                serialize_toml_value(single, dest);
                dest.add_bytes("\n");
            }
        }
    }
}

impl FormatEmitter for TomlEmitter {
    fn emit(&self, value: &Value, dest: &mut dyn IDestination) -> Result<(), BabbelError> {
        self.emit_to_dest(value, dest);
        Ok(())
    }
}

fn format_toml_key(key: &str, dest: &mut dyn IDestination) {
    if crate::escape::is_valid_toml_bare_key(key) {
        dest.add_bytes(key);
    } else {
        crate::escape::write_toml_escaped_string(key, dest);
    }
}

fn format_toml_table_path(full_path: &str, dest: &mut dyn IDestination) {
    let mut first = true;
    for part in full_path.split('.') {
        if !first {
            dest.add_byte(b'.');
        }
        first = false;
        format_toml_key(part, dest);
    }
}

fn is_array_of_objects(val: &Value) -> bool {
    match val {
        Value::Array(arr) => !arr.is_empty() && arr.iter().all(|it| matches!(it, Value::Object(_))),
        _ => false,
    }
}

fn serialize_toml_table(
    entries: &[(alloc::string::String, Value)],
    prefix: &str,
    dest: &mut dyn IDestination,
) {
    let mut has_written_direct = false;
    // 1. Emit direct scalar/inline properties
    for (k, v) in entries {
        if matches!(v, Value::Object(_)) || is_array_of_objects(v) {
            continue;
        }
        if !prefix.is_empty() && !has_written_direct {
            dest.add_bytes("[");
            format_toml_table_path(prefix, dest);
            dest.add_bytes("]\n");
        }
        format_toml_key(k, dest);
        dest.add_bytes(" = ");
        serialize_toml_value(v, dest);
        dest.add_byte(b'\n');
        has_written_direct = true;
    }

    // 2. Emit sub-tables
    for (k, v) in entries {
        if let Value::Object(sub_entries) = v {
            let full_key = if prefix.is_empty() {
                k.clone()
            } else {
                alloc::format!("{}.{}", prefix, k)
            };
            if sub_entries.is_empty() {
                dest.add_bytes("[");
                format_toml_table_path(&full_key, dest);
                dest.add_bytes("]\n");
            } else {
                serialize_toml_table(sub_entries, &full_key, dest);
            }
        }
    }

    // 3. Emit array of tables
    for (k, v) in entries {
        if is_array_of_objects(v) {
            if let Value::Array(items) = v {
                let full_key = if prefix.is_empty() {
                    k.clone()
                } else {
                    alloc::format!("{}.{}", prefix, k)
                };
                for item in items {
                    dest.add_bytes("[[");
                    format_toml_table_path(&full_key, dest);
                    dest.add_bytes("]]\n");
                    if let Value::Object(sub_entries) = item {
                        serialize_toml_table(sub_entries, &full_key, dest);
                    }
                }
            }
        }
    }
}

fn serialize_toml_value(val: &Value, dest: &mut dyn IDestination) {
    match val {
        Value::Null => dest.add_bytes("\"\""),
        Value::Bool(b) => dest.add_bytes(if *b { "true" } else { "false" }),
        Value::Integer(i) => {
            let mut buf = itoa::Buffer::new();
            dest.add_bytes(buf.format(*i));
        }
        Value::Float(f) => {
            if f.is_nan() {
                dest.add_bytes("nan");
            } else if f.is_infinite() {
                dest.add_bytes(if *f < 0.0 { "-inf" } else { "inf" });
            } else {
                let mut buf = dtoa::Buffer::new();
                let s = buf.format(*f);
                dest.add_bytes(s);
                if !s.contains('.') && !s.contains('e') && !s.contains('E') {
                    dest.add_bytes(".0");
                }
            }
        }
        Value::String(s) => crate::escape::write_toml_escaped_string(s, dest),
        Value::Bytes(b) => {
            crate::escape::write_toml_escaped_string(
                &alloc::string::String::from_utf8_lossy(b),
                dest,
            );
        }
        Value::Array(items) => {
            dest.add_byte(b'[');
            for (idx, item) in items.iter().enumerate() {
                if idx > 0 {
                    dest.add_bytes(", ");
                }
                serialize_toml_value(item, dest);
            }
            dest.add_byte(b']');
        }
        Value::Object(entries) => {
            dest.add_bytes("{ ");
            for (idx, (k, v)) in entries.iter().enumerate() {
                if idx > 0 {
                    dest.add_bytes(", ");
                }
                format_toml_key(k, dest);
                dest.add_bytes(" = ");
                serialize_toml_value(v, dest);
            }
            dest.add_bytes(" }");
        }
    }
}
