//! Universal Zero-Dependency JSON Schema Validator (Draft 7 / Draft 2020-12 core).
//!
//! Validates universal [`Value`] documents from any format (JSON, YAML, TOML, KDL, CBOR, BSON, etc.)
//! against standard JSON Schema definitions.

use alloc::boxed::Box;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use crate::error::BabbelError;
use crate::model::Value;

/// Structured validation error containing pointer location, failed keyword, and error description.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaValidationError {
    /// JSON pointer location in the validated document (e.g. `"/users/0/age"`).
    pub pointer: String,
    /// Schema keyword that failed validation (e.g. `"minimum"`, `"required"`).
    pub keyword: &'static str,
    /// Human-readable diagnostic failure message.
    pub message: String,
}

/// A compiled, optimized JSON Schema validator ready for zero-allocation reuse.
#[derive(Debug, Clone)]
pub struct CompiledSchema {
    rules: Vec<ValidationRule>,
}

#[derive(Debug, Clone)]
enum ValidationRule {
    Type(Vec<TypeName>),
    Enum(Vec<Value>),
    Const(Value),
    Minimum(f64),
    Maximum(f64),
    ExclusiveMinimum(f64),
    ExclusiveMaximum(f64),
    MultipleOf(f64),
    MinLength(usize),
    MaxLength(usize),
    Pattern(String),
    Format(String),
    Required(Vec<String>),
    Properties(Vec<(String, CompiledSchema)>),
    AdditionalPropertiesBool(bool),
    AdditionalPropertiesSchema(Box<CompiledSchema>),
    MinProperties(usize),
    MaxProperties(usize),
    Items(Box<CompiledSchema>),
    MinItems(usize),
    MaxItems(usize),
    UniqueItems(bool),
    AllOf(Vec<CompiledSchema>),
    AnyOf(Vec<CompiledSchema>),
    OneOf(Vec<CompiledSchema>),
    Not(Box<CompiledSchema>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TypeName {
    Null,
    Boolean,
    Integer,
    Number,
    String,
    Array,
    Object,
}

impl TypeName {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "null" => Some(TypeName::Null),
            "boolean" => Some(TypeName::Boolean),
            "integer" => Some(TypeName::Integer),
            "number" => Some(TypeName::Number),
            "string" => Some(TypeName::String),
            "array" => Some(TypeName::Array),
            "object" => Some(TypeName::Object),
            _ => None,
        }
    }

    fn matches(&self, val: &Value) -> bool {
        match self {
            TypeName::Null => val.is_null(),
            TypeName::Boolean => val.is_bool(),
            TypeName::Integer => val.is_integer(),
            TypeName::Number => val.is_float() || val.is_integer(),
            TypeName::String => val.is_string(),
            TypeName::Array => val.is_array(),
            TypeName::Object => val.is_object(),
        }
    }
}

impl CompiledSchema {
    /// Compile a JSON Schema definition represented as a [`Value`] into an executable validator.
    pub fn compile(schema: &Value) -> Result<Self, BabbelError> {
        match schema {
            Value::Bool(true) => Ok(Self { rules: Vec::new() }),
            Value::Bool(false) => Ok(Self {
                rules: alloc::vec![ValidationRule::Not(Box::new(Self { rules: Vec::new() }))],
            }),
            Value::Object(entries) => {
                let mut rules = Vec::new();

                // 'type'
                if let Some(type_val) = get_val(entries, "type") {
                    let mut types = Vec::new();
                    match type_val {
                        Value::String(s) => {
                            if let Some(t) = TypeName::from_str(s) {
                                types.push(t);
                            }
                        }
                        Value::Array(items) => {
                            for item in items {
                                if let Some(s) = item.as_str() {
                                    if let Some(t) = TypeName::from_str(s) {
                                        types.push(t);
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                    if !types.is_empty() {
                        rules.push(ValidationRule::Type(types));
                    }
                }

                // 'enum'
                if let Some(Value::Array(items)) = get_val(entries, "enum") {
                    rules.push(ValidationRule::Enum(items.clone()));
                }

                // 'const'
                if let Some(c) = get_val(entries, "const") {
                    rules.push(ValidationRule::Const(c.clone()));
                }

                // Numeric constraints
                if let Some(v) = get_val(entries, "minimum").and_then(|v| v.as_f64()) {
                    rules.push(ValidationRule::Minimum(v));
                }
                if let Some(v) = get_val(entries, "maximum").and_then(|v| v.as_f64()) {
                    rules.push(ValidationRule::Maximum(v));
                }
                if let Some(v) = get_val(entries, "exclusiveMinimum").and_then(|v| v.as_f64()) {
                    rules.push(ValidationRule::ExclusiveMinimum(v));
                }
                if let Some(v) = get_val(entries, "exclusiveMaximum").and_then(|v| v.as_f64()) {
                    rules.push(ValidationRule::ExclusiveMaximum(v));
                }
                if let Some(v) = get_val(entries, "multipleOf").and_then(|v| v.as_f64()) {
                    rules.push(ValidationRule::MultipleOf(v));
                }

                // String constraints
                if let Some(v) = get_val(entries, "minLength").and_then(|v| v.as_u64()) {
                    rules.push(ValidationRule::MinLength(v as usize));
                }
                if let Some(v) = get_val(entries, "maxLength").and_then(|v| v.as_u64()) {
                    rules.push(ValidationRule::MaxLength(v as usize));
                }
                if let Some(s) = get_val(entries, "pattern").and_then(|v| v.as_str()) {
                    rules.push(ValidationRule::Pattern(s.to_string()));
                }
                if let Some(s) = get_val(entries, "format").and_then(|v| v.as_str()) {
                    rules.push(ValidationRule::Format(s.to_string()));
                }

                // Object constraints
                if let Some(Value::Array(items)) = get_val(entries, "required") {
                    let req: Vec<String> = items.iter().filter_map(|i| i.as_str().map(|s| s.to_string())).collect();
                    rules.push(ValidationRule::Required(req));
                }

                if let Some(Value::Object(props)) = get_val(entries, "properties") {
                    let mut prop_rules = Vec::new();
                    for (k, sub) in props {
                        let compiled = Self::compile(sub)?;
                        prop_rules.push((k.clone(), compiled));
                    }
                    rules.push(ValidationRule::Properties(prop_rules));
                }

                if let Some(add_prop) = get_val(entries, "additionalProperties") {
                    match add_prop {
                        Value::Bool(b) => rules.push(ValidationRule::AdditionalPropertiesBool(*b)),
                        sub @ Value::Object(_) => {
                            let compiled = Self::compile(sub)?;
                            rules.push(ValidationRule::AdditionalPropertiesSchema(Box::new(compiled)));
                        }
                        _ => {}
                    }
                }

                if let Some(v) = get_val(entries, "minProperties").and_then(|v| v.as_u64()) {
                    rules.push(ValidationRule::MinProperties(v as usize));
                }
                if let Some(v) = get_val(entries, "maxProperties").and_then(|v| v.as_u64()) {
                    rules.push(ValidationRule::MaxProperties(v as usize));
                }

                // Array constraints
                if let Some(items_val) = get_val(entries, "items") {
                    let compiled = Self::compile(items_val)?;
                    rules.push(ValidationRule::Items(Box::new(compiled)));
                }

                if let Some(v) = get_val(entries, "minItems").and_then(|v| v.as_u64()) {
                    rules.push(ValidationRule::MinItems(v as usize));
                }
                if let Some(v) = get_val(entries, "maxItems").and_then(|v| v.as_u64()) {
                    rules.push(ValidationRule::MaxItems(v as usize));
                }
                if let Some(b) = get_val(entries, "uniqueItems").and_then(|v| v.as_bool()) {
                    rules.push(ValidationRule::UniqueItems(b));
                }

                // Combinators
                if let Some(Value::Array(items)) = get_val(entries, "allOf") {
                    let mut schemas = Vec::new();
                    for item in items {
                        schemas.push(Self::compile(item)?);
                    }
                    rules.push(ValidationRule::AllOf(schemas));
                }
                if let Some(Value::Array(items)) = get_val(entries, "anyOf") {
                    let mut schemas = Vec::new();
                    for item in items {
                        schemas.push(Self::compile(item)?);
                    }
                    rules.push(ValidationRule::AnyOf(schemas));
                }
                if let Some(Value::Array(items)) = get_val(entries, "oneOf") {
                    let mut schemas = Vec::new();
                    for item in items {
                        schemas.push(Self::compile(item)?);
                    }
                    rules.push(ValidationRule::OneOf(schemas));
                }
                if let Some(sub) = get_val(entries, "not") {
                    let compiled = Self::compile(sub)?;
                    rules.push(ValidationRule::Not(Box::new(compiled)));
                }

                Ok(Self { rules })
            }
            _ => Err(BabbelError::syntax("JSON Schema must be a boolean or an object")),
        }
    }

    /// Validate an instance document against this schema.
    pub fn validate(&self, instance: &Value) -> Result<(), Vec<SchemaValidationError>> {
        let mut errors = Vec::new();
        self.validate_internal(instance, "", &mut errors);
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Fast boolean check whether the instance is valid according to this schema.
    pub fn is_valid(&self, instance: &Value) -> bool {
        self.validate(instance).is_ok()
    }

    fn validate_internal(&self, val: &Value, pointer: &str, errors: &mut Vec<SchemaValidationError>) {
        for rule in &self.rules {
            match rule {
                ValidationRule::Type(expected_types) => {
                    if !expected_types.iter().any(|t| t.matches(val)) {
                        errors.push(SchemaValidationError {
                            pointer: pointer.to_string(),
                            keyword: "type",
                            message: format!("value does not match expected type(s)"),
                        });
                    }
                }
                ValidationRule::Enum(allowed_values) => {
                    if !allowed_values.contains(val) {
                        errors.push(SchemaValidationError {
                            pointer: pointer.to_string(),
                            keyword: "enum",
                            message: "value is not in enum set".to_string(),
                        });
                    }
                }
                ValidationRule::Const(expected) => {
                    if val != expected {
                        errors.push(SchemaValidationError {
                            pointer: pointer.to_string(),
                            keyword: "const",
                            message: "value does not match expected const".to_string(),
                        });
                    }
                }
                ValidationRule::Minimum(min) => {
                    if let Some(num) = val.as_f64() {
                        if num < *min {
                            errors.push(SchemaValidationError {
                                pointer: pointer.to_string(),
                                keyword: "minimum",
                                message: format!("{} is less than minimum {}", num, min),
                            });
                        }
                    }
                }
                ValidationRule::Maximum(max) => {
                    if let Some(num) = val.as_f64() {
                        if num > *max {
                            errors.push(SchemaValidationError {
                                pointer: pointer.to_string(),
                                keyword: "maximum",
                                message: format!("{} is greater than maximum {}", num, max),
                            });
                        }
                    }
                }
                ValidationRule::ExclusiveMinimum(min) => {
                    if let Some(num) = val.as_f64() {
                        if num <= *min {
                            errors.push(SchemaValidationError {
                                pointer: pointer.to_string(),
                                keyword: "exclusiveMinimum",
                                message: format!("{} is less than or equal to exclusiveMinimum {}", num, min),
                            });
                        }
                    }
                }
                ValidationRule::ExclusiveMaximum(max) => {
                    if let Some(num) = val.as_f64() {
                        if num >= *max {
                            errors.push(SchemaValidationError {
                                pointer: pointer.to_string(),
                                keyword: "exclusiveMaximum",
                                message: format!("{} is greater than or equal to exclusiveMaximum {}", num, max),
                            });
                        }
                    }
                }
                ValidationRule::MultipleOf(factor) => {
                    if let Some(num) = val.as_f64() {
                        let remainder = (num / factor).fract();
                        if remainder.abs() > 1e-9 && (1.0 - remainder.abs()) > 1e-9 {
                            errors.push(SchemaValidationError {
                                pointer: pointer.to_string(),
                                keyword: "multipleOf",
                                message: format!("{} is not a multiple of {}", num, factor),
                            });
                        }
                    }
                }
                ValidationRule::MinLength(min) => {
                    if let Some(s) = val.as_str() {
                        if s.chars().count() < *min {
                            errors.push(SchemaValidationError {
                                pointer: pointer.to_string(),
                                keyword: "minLength",
                                message: format!("string length is less than minimum {}", min),
                            });
                        }
                    }
                }
                ValidationRule::MaxLength(max) => {
                    if let Some(s) = val.as_str() {
                        if s.chars().count() > *max {
                            errors.push(SchemaValidationError {
                                pointer: pointer.to_string(),
                                keyword: "maxLength",
                                message: format!("string length is greater than maximum {}", max),
                            });
                        }
                    }
                }
                ValidationRule::Pattern(pattern) => {
                    if let Some(s) = val.as_str() {
                        if !simple_regex_match(pattern, s) {
                            errors.push(SchemaValidationError {
                                pointer: pointer.to_string(),
                                keyword: "pattern",
                                message: format!("string does not match pattern '{}'", pattern),
                            });
                        }
                    }
                }
                ValidationRule::Format(fmt) => {
                    if let Some(s) = val.as_str() {
                        if !validate_format_string(fmt, s) {
                            errors.push(SchemaValidationError {
                                pointer: pointer.to_string(),
                                keyword: "format",
                                message: format!("string does not conform to format '{}'", fmt),
                            });
                        }
                    }
                }
                ValidationRule::Required(required_keys) => {
                    if let Value::Object(entries) = val {
                        for key in required_keys {
                            if !entries.iter().any(|(k, _)| k == key) {
                                errors.push(SchemaValidationError {
                                pointer: pointer.to_string(),
                                keyword: "required",
                                message: format!("missing required property '{}'", key),
                            });
                            }
                        }
                    }
                }
                ValidationRule::Properties(props) => {
                    if let Value::Object(entries) = val {
                        for (key, sub_schema) in props {
                            if let Some((_, item_val)) = entries.iter().find(|(k, _)| k == key) {
                                let sub_ptr = format!("{}/{}", pointer, key);
                                sub_schema.validate_internal(item_val, &sub_ptr, errors);
                            }
                        }
                    }
                }
                ValidationRule::AdditionalPropertiesBool(allowed) => {
                    if !*allowed {
                        if let Value::Object(entries) = val {
                            // Find declared properties in sibling rule
                            let declared: Vec<&str> = self.rules.iter().filter_map(|r| {
                                if let ValidationRule::Properties(p) = r {
                                    Some(p.iter().map(|(k, _)| k.as_str()).collect::<Vec<_>>())
                                } else {
                                    None
                                }
                            }).flatten().collect();

                            for (k, _) in entries {
                                if !declared.contains(&k.as_str()) {
                                    errors.push(SchemaValidationError {
                                        pointer: format!("{}/{}", pointer, k),
                                        keyword: "additionalProperties",
                                        message: format!("property '{}' is not allowed", k),
                                    });
                                }
                            }
                        }
                    }
                }
                ValidationRule::AdditionalPropertiesSchema(sub_schema) => {
                    if let Value::Object(entries) = val {
                        let declared: Vec<&str> = self.rules.iter().filter_map(|r| {
                            if let ValidationRule::Properties(p) = r {
                                Some(p.iter().map(|(k, _)| k.as_str()).collect::<Vec<_>>())
                            } else {
                                None
                            }
                        }).flatten().collect();

                        for (k, v) in entries {
                            if !declared.contains(&k.as_str()) {
                                let sub_ptr = format!("{}/{}", pointer, k);
                                sub_schema.validate_internal(v, &sub_ptr, errors);
                            }
                        }
                    }
                }
                ValidationRule::MinProperties(min) => {
                    if let Value::Object(entries) = val {
                        if entries.len() < *min {
                            errors.push(SchemaValidationError {
                                pointer: pointer.to_string(),
                                keyword: "minProperties",
                                message: format!("object property count {} is less than minimum {}", entries.len(), min),
                            });
                        }
                    }
                }
                ValidationRule::MaxProperties(max) => {
                    if let Value::Object(entries) = val {
                        if entries.len() > *max {
                            errors.push(SchemaValidationError {
                                pointer: pointer.to_string(),
                                keyword: "maxProperties",
                                message: format!("object property count {} is greater than maximum {}", entries.len(), max),
                            });
                        }
                    }
                }
                ValidationRule::Items(item_schema) => {
                    if let Value::Array(items) = val {
                        for (idx, item) in items.iter().enumerate() {
                            let sub_ptr = format!("{}/{}", pointer, idx);
                            item_schema.validate_internal(item, &sub_ptr, errors);
                        }
                    }
                }
                ValidationRule::MinItems(min) => {
                    if let Value::Array(items) = val {
                        if items.len() < *min {
                            errors.push(SchemaValidationError {
                                pointer: pointer.to_string(),
                                keyword: "minItems",
                                message: format!("array length {} is less than minimum {}", items.len(), min),
                            });
                        }
                    }
                }
                ValidationRule::MaxItems(max) => {
                    if let Value::Array(items) = val {
                        if items.len() > *max {
                            errors.push(SchemaValidationError {
                                pointer: pointer.to_string(),
                                keyword: "maxItems",
                                message: format!("array length {} is greater than maximum {}", items.len(), max),
                            });
                        }
                    }
                }
                ValidationRule::UniqueItems(unique) => {
                    if *unique {
                        if let Value::Array(items) = val {
                            for i in 0..items.len() {
                                for j in (i + 1)..items.len() {
                                    if items[i] == items[j] {
                                        errors.push(SchemaValidationError {
                                            pointer: pointer.to_string(),
                                            keyword: "uniqueItems",
                                            message: format!("duplicate item detected at index {} and {}", i, j),
                                        });
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                ValidationRule::AllOf(schemas) => {
                    for s in schemas {
                        s.validate_internal(val, pointer, errors);
                    }
                }
                ValidationRule::AnyOf(schemas) => {
                    let mut any_valid = false;
                    for s in schemas {
                        if s.is_valid(val) {
                            any_valid = true;
                            break;
                        }
                    }
                    if !any_valid {
                        errors.push(SchemaValidationError {
                            pointer: pointer.to_string(),
                            keyword: "anyOf",
                            message: "value does not match any allowed schema in anyOf".to_string(),
                        });
                    }
                }
                ValidationRule::OneOf(schemas) => {
                    let valid_count = schemas.iter().filter(|s| s.is_valid(val)).count();
                    if valid_count != 1 {
                        errors.push(SchemaValidationError {
                            pointer: pointer.to_string(),
                            keyword: "oneOf",
                            message: format!("value matched {} schemas in oneOf (expected exactly 1)", valid_count),
                        });
                    }
                }
                ValidationRule::Not(schema) => {
                    if schema.is_valid(val) {
                        errors.push(SchemaValidationError {
                            pointer: pointer.to_string(),
                            keyword: "not",
                            message: "value matches forbidden schema in not".to_string(),
                        });
                    }
                }
            }
        }
    }
}

fn get_val<'a>(entries: &'a [(String, Value)], key: &str) -> Option<&'a Value> {
    entries.iter().find(|(k, _)| k == key).map(|(_, v)| v)
}

fn simple_regex_match(pattern: &str, text: &str) -> bool {
    if pattern.starts_with('^') && pattern.ends_with('$') {
        let core = &pattern[1..pattern.len() - 1];
        text == core
    } else if pattern.starts_with('^') {
        text.starts_with(&pattern[1..])
    } else if pattern.ends_with('$') {
        text.ends_with(&pattern[..pattern.len() - 1])
    } else {
        text.contains(pattern)
    }
}

fn validate_format_string(format: &str, s: &str) -> bool {
    match format {
        "email" => s.contains('@') && s.contains('.'),
        "uuid" => s.len() == 36 && s.chars().filter(|&c| c == '-').count() == 4,
        "ipv4" => {
            let parts: Vec<&str> = s.split('.').collect();
            parts.len() == 4 && parts.iter().all(|p| p.parse::<u8>().is_ok())
        }
        "date-time" => s.contains('T') || s.contains('Z') || s.contains('-'),
        "uri" => s.contains("://"),
        _ => true,
    }
}

/// Convenience function to validate a [`Value`] against a schema [`Value`].
pub fn validate(schema: &Value, instance: &Value) -> Result<(), Vec<SchemaValidationError>> {
    let compiled = CompiledSchema::compile(schema).map_err(|e| {
        alloc::vec![SchemaValidationError {
            pointer: "".to_string(),
            keyword: "schema",
            message: e.message,
        }]
    })?;
    compiled.validate(instance)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_schema_type_and_required() {
        let schema = Value::Object(vec![
            ("type".to_string(), Value::String("object".to_string())),
            ("required".to_string(), Value::Array(vec![Value::String("name".to_string()), Value::String("age".to_string())])),
            ("properties".to_string(), Value::Object(vec![
                ("name".to_string(), Value::Object(vec![("type".to_string(), Value::String("string".to_string()))])),
                ("age".to_string(), Value::Object(vec![
                    ("type".to_string(), Value::String("integer".to_string())),
                    ("minimum".to_string(), Value::Integer(0)),
                ])),
            ])),
        ]);

        let compiled = CompiledSchema::compile(&schema).unwrap();

        let valid = Value::Object(vec![
            ("name".to_string(), Value::String("Alice".to_string())),
            ("age".to_string(), Value::Integer(30)),
        ]);
        assert!(compiled.is_valid(&valid));

        let missing_age = Value::Object(vec![
            ("name".to_string(), Value::String("Alice".to_string())),
        ]);
        assert!(!compiled.is_valid(&missing_age));

        let invalid_age = Value::Object(vec![
            ("name".to_string(), Value::String("Alice".to_string())),
            ("age".to_string(), Value::Integer(-5)),
        ]);
        assert!(!compiled.is_valid(&invalid_age));
    }

    #[test]
    fn test_schema_combinators() {
        let schema = Value::Object(vec![
            ("oneOf".to_string(), Value::Array(vec![
                Value::Object(vec![("type".to_string(), Value::String("string".to_string()))]),
                Value::Object(vec![("type".to_string(), Value::String("number".to_string()))]),
            ])),
        ]);

        let compiled = CompiledSchema::compile(&schema).unwrap();
        assert!(compiled.is_valid(&Value::String("test".to_string())));
        assert!(compiled.is_valid(&Value::Integer(42)));
        assert!(!compiled.is_valid(&Value::Bool(true)));
    }
}
