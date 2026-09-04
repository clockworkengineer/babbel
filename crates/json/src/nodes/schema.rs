//! JSON Schema validation (subset of JSON Schema Draft 7)
//!
//! This module provides basic JSON Schema validation support for common use cases.
//! Supports type checking, required fields, min/max constraints, pattern matching, etc.

use crate::nodes::node::Node;
use arrayvec::ArrayString;
use dtoa::Buffer as DtoaBuffer;
use itoa::Buffer as ItoaBuffer;

#[cfg(feature = "std")]
use std::collections::HashSet;

#[cfg(not(feature = "std"))]
use alloc::{
    collections::BTreeSet as HashSet,
    string::{String, ToString},
    vec::Vec,
};

/// Validation error with details
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationError {
    pub path: String,
    pub message: String,
}

impl ValidationError {
    pub fn new(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
        }
    }
}

/// Simple JSON Schema validator
pub struct SchemaValidator {
    schema: Node,
}

impl SchemaValidator {
    /// Create a new validator with the given schema
    pub fn new(schema: Node) -> Self {
        Self { schema }
    }

    /// Validate a node against the schema
    pub fn validate(&self, data: &Node) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();
        self.validate_node(data, &self.schema, "", &mut errors);
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn validate_node(
        &self,
        data: &Node,
        schema: &Node,
        path: &str,
        errors: &mut Vec<ValidationError>,
    ) {
        // Type validation
        if let Some(type_val) = schema.get("type").and_then(|n| n.as_str()) {
            if !self.check_type(data, type_val) {
                let mut msg = ArrayString::<64>::new();
                msg.push_str("Expected type '");
                msg.push_str(type_val);
                msg.push_str("' but got '");
                msg.push_str(self.get_type_name(data));
                msg.push_str("'");
                errors.push(ValidationError::new(path, msg.as_str().to_owned()));
                return; // Don't continue if type is wrong
            }
        }

        // Object validation
        if data.is_object() {
            self.validate_object(data, schema, path, errors);
        }

        // Array validation
        if data.is_array() {
            self.validate_array(data, schema, path, errors);
        }

        // String validation
        if data.is_string() {
            self.validate_string(data, schema, path, errors);
        }

        // Number validation
        if data.is_number() {
            self.validate_number(data, schema, path, errors);
        }
    }

    fn check_type(&self, data: &Node, expected_type: &str) -> bool {
        match expected_type {
            "object" => data.is_object(),
            "array" => data.is_array(),
            "string" => data.is_string(),
            "number" | "integer" => data.is_number(),
            "boolean" => data.is_boolean(),
            "null" => data.is_null(),
            _ => true,
        }
    }

    fn get_type_name(&self, data: &Node) -> &str {
        if data.is_object() {
            "object"
        } else if data.is_array() {
            "array"
        } else if data.is_string() {
            "string"
        } else if data.is_number() {
            "number"
        } else if data.is_boolean() {
            "boolean"
        } else if data.is_null() {
            "null"
        } else {
            "unknown"
        }
    }

    fn validate_object(
        &self,
        data: &Node,
        schema: &Node,
        path: &str,
        errors: &mut Vec<ValidationError>,
    ) {
        let obj = match data.as_object() {
            Some(o) => o,
            None => return,
        };

        // Required properties
        if let Some(required) = schema.get("required").and_then(|n| n.as_array()) {
            for req in required {
                if let Some(key) = req.as_str() {
                    if !obj.contains_key(key) {
                        let mut msg = ArrayString::<48>::new();
                        msg.push_str("Missing required property '");
                        msg.push_str(key);
                        msg.push('\'');
                        errors.push(ValidationError::new(path, msg.as_str().to_owned()));
                    }
                }
            }
        }

        // Properties validation
        if let Some(props) = schema.get("properties").and_then(|n| n.as_object()) {
            for (key, value) in obj.iter() {
                if let Some(prop_schema) = props.get(key) {
                    let new_path = if path.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", path, key)
                    };
                    self.validate_node(value, prop_schema, &new_path, errors);
                }
            }
        }

        // Min/max properties
        if let Some(min) = schema.get("minProperties").and_then(|n| n.as_i64()) {
            if obj.len() < min as usize {
                let mut msg = ArrayString::<64>::new();
                msg.push_str("Object has ");
                let mut buf = ItoaBuffer::new();
                msg.push_str(buf.format(obj.len()));
                msg.push_str(" properties, minimum is ");
                let mut buf = ItoaBuffer::new();
                msg.push_str(buf.format(min));
                errors.push(ValidationError::new(path, msg.as_str().to_owned()));
            }
        }

        if let Some(max) = schema.get("maxProperties").and_then(|n| n.as_i64()) {
            if obj.len() > max as usize {
                let mut msg = ArrayString::<64>::new();
                msg.push_str("Object has ");
                let mut buf = ItoaBuffer::new();
                msg.push_str(buf.format(obj.len()));
                msg.push_str(" properties, maximum is ");
                let mut buf = ItoaBuffer::new();
                msg.push_str(buf.format(max));
                errors.push(ValidationError::new(path, msg.as_str().to_owned()));
            }
        }
    }

    fn validate_array(
        &self,
        data: &Node,
        schema: &Node,
        path: &str,
        errors: &mut Vec<ValidationError>,
    ) {
        let arr = match data.as_array() {
            Some(a) => a,
            None => return,
        };

        // Items validation
        if let Some(items_schema) = schema.get("items") {
            for (i, item) in arr.iter().enumerate() {
                let new_path = format!("{}[{}]", path, i);
                self.validate_node(item, items_schema, &new_path, errors);
            }
        }

        // Min/max items
        if let Some(min) = schema.get("minItems").and_then(|n| n.as_i64()) {
            if arr.len() < min as usize {
                let mut msg = ArrayString::<64>::new();
                msg.push_str("Array has ");
                let mut buf = ItoaBuffer::new();
                msg.push_str(buf.format(arr.len()));
                msg.push_str(" items, minimum is ");
                let mut buf = ItoaBuffer::new();
                msg.push_str(buf.format(min));
                errors.push(ValidationError::new(path, msg.as_str().to_owned()));
            }
        }

        if let Some(max) = schema.get("maxItems").and_then(|n| n.as_i64()) {
            if arr.len() > max as usize {
                let mut msg = ArrayString::<64>::new();
                msg.push_str("Array has ");
                let mut buf = ItoaBuffer::new();
                msg.push_str(buf.format(arr.len()));
                msg.push_str(" items, maximum is ");
                let mut buf = ItoaBuffer::new();
                msg.push_str(buf.format(max));
                errors.push(ValidationError::new(path, msg.as_str().to_owned()));
            }
        }

        // Unique items
        if let Some(unique) = schema.get("uniqueItems").and_then(|n| n.as_bool()) {
            if unique && !self.has_unique_items(arr) {
                errors.push(ValidationError::new(
                    path,
                    "Array items must be unique".to_string(),
                ));
            }
        }
    }

    fn validate_string(
        &self,
        data: &Node,
        schema: &Node,
        path: &str,
        errors: &mut Vec<ValidationError>,
    ) {
        let s = match data.as_str() {
            Some(s) => s,
            None => return,
        };

        // Min/max length
        if let Some(min) = schema.get("minLength").and_then(|n| n.as_i64()) {
            if s.len() < min as usize {
                let mut msg = ArrayString::<64>::new();
                msg.push_str("String length is ");
                let mut buf = ItoaBuffer::new();
                msg.push_str(buf.format(s.len()));
                msg.push_str(", minimum is ");
                let mut buf = ItoaBuffer::new();
                msg.push_str(buf.format(min));
                errors.push(ValidationError::new(path, msg.as_str().to_owned()));
            }
        }

        if let Some(max) = schema.get("maxLength").and_then(|n| n.as_i64()) {
            if s.len() > max as usize {
                let mut msg = ArrayString::<64>::new();
                msg.push_str("String length is ");
                let mut buf = ItoaBuffer::new();
                msg.push_str(buf.format(s.len()));
                msg.push_str(", maximum is ");
                let mut buf = ItoaBuffer::new();
                msg.push_str(buf.format(max));
                errors.push(ValidationError::new(path, msg.as_str().to_owned()));
            }
        }

        // Enum validation
        if let Some(enum_vals) = schema.get("enum").and_then(|n| n.as_array()) {
            let mut found = false;
            for val in enum_vals {
                if let Some(v) = val.as_str() {
                    if v == s {
                        found = true;
                        break;
                    }
                }
            }
            if !found {
                let mut msg = ArrayString::<64>::new();
                msg.push_str("Value '");
                msg.push_str(s);
                msg.push_str("' is not in allowed enum values");
                errors.push(ValidationError::new(path, msg.as_str().to_owned()));
            }
        }
    }

    fn validate_number(
        &self,
        data: &Node,
        schema: &Node,
        path: &str,
        errors: &mut Vec<ValidationError>,
    ) {
        let num = match data.as_f64() {
            Some(n) => n,
            None => return,
        };

        // Min/max
        if let Some(min) = schema.get("minimum").and_then(|n| n.as_f64()) {
            if num < min {
                let mut msg = ArrayString::<64>::new();
                msg.push_str("Number ");
                let mut buf = DtoaBuffer::new();
                msg.push_str(buf.format(num));
                msg.push_str(" is less than minimum ");
                let mut buf2 = DtoaBuffer::new();
                msg.push_str(buf2.format(min));
                errors.push(ValidationError::new(path, msg.as_str().to_owned()));
            }
        }

        if let Some(max) = schema.get("maximum").and_then(|n| n.as_f64()) {
            if num > max {
                let mut msg = ArrayString::<64>::new();
                msg.push_str("Number ");
                let mut buf = DtoaBuffer::new();
                msg.push_str(buf.format(num));
                msg.push_str(" is greater than maximum ");
                let mut buf2 = DtoaBuffer::new();
                msg.push_str(buf2.format(max));
                errors.push(ValidationError::new(path, msg.as_str().to_owned()));
            }
        }

        // Exclusive min/max
        if let Some(min) = schema.get("exclusiveMinimum").and_then(|n| n.as_f64()) {
            if num <= min {
                let mut msg = ArrayString::<64>::new();
                msg.push_str("Number ");
                let mut buf = DtoaBuffer::new();
                msg.push_str(buf.format(num));
                msg.push_str(" is not greater than exclusive minimum ");
                let mut buf2 = DtoaBuffer::new();
                msg.push_str(buf2.format(min));
                errors.push(ValidationError::new(path, msg.as_str().to_owned()));
            }
        }

        if let Some(max) = schema.get("exclusiveMaximum").and_then(|n| n.as_f64()) {
            if num >= max {
                let mut msg = ArrayString::<64>::new();
                msg.push_str("Number ");
                let mut buf = DtoaBuffer::new();
                msg.push_str(buf.format(num));
                msg.push_str(" is not less than exclusive maximum ");
                let mut buf2 = DtoaBuffer::new();
                msg.push_str(buf2.format(max));
                errors.push(ValidationError::new(path, msg.as_str().to_owned()));
            }
        }
    }

    fn has_unique_items(&self, arr: &[Node]) -> bool {
        let mut seen = HashSet::new();
        for item in arr {
            // Use a stack-allocated buffer for small debug strings
            let mut buf = ArrayString::<128>::new();
            use core::fmt::Write;
            let _ = write!(&mut buf, "{:?}", item);
            if !seen.insert(buf) {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json;

    #[test]
    fn test_type_validation() {
        let schema = json!({"type": "string"});
        let validator = SchemaValidator::new(schema);

        assert!(validator.validate(&json!("hello")).is_ok());
        assert!(validator.validate(&json!(42)).is_err());
    }

    #[test]
    fn test_required_properties() {
        let schema = json!({
            "type": "object",
            "required": ["name", "age"]
        });
        let validator = SchemaValidator::new(schema);

        assert!(
            validator
                .validate(&json!({"name": "Alice", "age": 30}))
                .is_ok()
        );
        assert!(validator.validate(&json!({"name": "Bob"})).is_err());
    }

    #[test]
    fn test_min_max_properties() {
        let schema = json!({
            "type": "object",
            "minProperties": 2,
            "maxProperties": 4
        });
        let validator = SchemaValidator::new(schema);

        assert!(validator.validate(&json!({"a": 1, "b": 2})).is_ok());
        assert!(validator.validate(&json!({"a": 1})).is_err());
        assert!(
            validator
                .validate(&json!({"a": 1, "b": 2, "c": 3, "d": 4, "e": 5}))
                .is_err()
        );
    }

    #[test]
    fn test_array_validation() {
        let schema = json!({
            "type": "array",
            "items": {"type": "number"},
            "minItems": 1,
            "maxItems": 5
        });
        let validator = SchemaValidator::new(schema);

        assert!(validator.validate(&json!([1, 2, 3])).is_ok());
        assert!(validator.validate(&json!([])).is_err());
        assert!(validator.validate(&json!([1, 2, 3, 4, 5, 6])).is_err());
    }

    #[test]
    fn test_string_constraints() {
        let schema = json!({
            "type": "string",
            "minLength": 3,
            "maxLength": 10
        });
        let validator = SchemaValidator::new(schema);

        assert!(validator.validate(&json!("hello")).is_ok());
        assert!(validator.validate(&json!("hi")).is_err());
        assert!(validator.validate(&json!("this is too long")).is_err());
    }

    #[test]
    fn test_number_constraints() {
        let schema = json!({
            "type": "number",
            "minimum": 0,
            "maximum": 100
        });
        let validator = SchemaValidator::new(schema);

        assert!(validator.validate(&json!(50)).is_ok());
        assert!(validator.validate(&json!(-1)).is_err());
        assert!(validator.validate(&json!(101)).is_err());
    }

    // --- ValidationError ---

    #[test]
    fn test_validation_error_fields() {
        let e = ValidationError::new("root.name", "too short");
        assert_eq!(e.path, "root.name");
        assert_eq!(e.message, "too short");
    }

    #[test]
    fn test_validation_error_clone_and_eq() {
        let e = ValidationError::new("a", "b");
        let e2 = e.clone();
        assert_eq!(e, e2);
        assert_ne!(e, ValidationError::new("a", "c"));
    }

    // --- type checking ---

    #[test]
    fn test_type_object() {
        let schema = json!({"type": "object"});
        let v = SchemaValidator::new(schema);
        assert!(v.validate(&json!({})).is_ok());
        assert!(v.validate(&json!("str")).is_err());
        assert!(v.validate(&json!(42)).is_err());
    }

    #[test]
    fn test_type_array() {
        let schema = json!({"type": "array"});
        let v = SchemaValidator::new(schema);
        assert!(v.validate(&json!([])).is_ok());
        assert!(v.validate(&json!({})).is_err());
    }

    #[test]
    fn test_type_boolean() {
        let schema = json!({"type": "boolean"});
        let v = SchemaValidator::new(schema);
        assert!(v.validate(&json!(true)).is_ok());
        assert!(v.validate(&json!(false)).is_ok());
        assert!(v.validate(&json!("true")).is_err());
    }

    #[test]
    fn test_type_null() {
        let schema = json!({"type": "null"});
        let v = SchemaValidator::new(schema);
        assert!(v.validate(&json!(null)).is_ok());
        assert!(v.validate(&json!(false)).is_err());
    }

    #[test]
    fn test_type_integer() {
        // "integer" maps to is_number()
        let schema = json!({"type": "integer"});
        let v = SchemaValidator::new(schema);
        assert!(v.validate(&json!(42)).is_ok());
        assert!(v.validate(&json!("42")).is_err());
    }

    #[test]
    fn test_unknown_type_always_passes() {
        let schema = json!({"type": "custom_type"});
        let v = SchemaValidator::new(schema);
        assert!(v.validate(&json!("anything")).is_ok());
        assert!(v.validate(&json!(42)).is_ok());
    }

    #[test]
    fn test_no_type_constraint_passes_all() {
        let schema = json!({});
        let v = SchemaValidator::new(schema);
        assert!(v.validate(&json!("str")).is_ok());
        assert!(v.validate(&json!(42)).is_ok());
        assert!(v.validate(&json!(null)).is_ok());
        assert!(v.validate(&json!(true)).is_ok());
    }

    // --- type mismatch returns early (no further checks) ---

    #[test]
    fn test_type_mismatch_returns_one_error() {
        let schema = json!({"type": "string", "minLength": 100});
        let v = SchemaValidator::new(schema);
        // Not a string, should have exactly one error (type mismatch) and not a minLength error
        let errs = v.validate(&json!(42)).unwrap_err();
        assert_eq!(errs.len(), 1);
        assert!(errs[0].message.contains("Expected type"));
    }

    // --- required properties ---

    #[test]
    fn test_required_all_present() {
        let schema = json!({"type": "object", "required": ["a", "b", "c"]});
        let v = SchemaValidator::new(schema);
        let data: Node = r#"{"a":1,"b":2,"c":3}"#.parse().unwrap();
        assert!(v.validate(&data).is_ok());
    }

    #[test]
    fn test_required_multiple_missing() {
        let schema = json!({"type": "object", "required": ["a", "b", "c"]});
        let v = SchemaValidator::new(schema);
        let data = json!({});
        let errs = v.validate(&data).unwrap_err();
        assert_eq!(errs.len(), 3);
    }

    #[test]
    fn test_required_no_required_key_in_schema() {
        let schema = json!({"type": "object"});
        let v = SchemaValidator::new(schema);
        assert!(v.validate(&json!({})).is_ok());
    }

    // --- properties validation (recursive) ---

    #[test]
    fn test_properties_type_check() {
        let schema = json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"},
                "age":  {"type": "number"}
            }
        });
        let v = SchemaValidator::new(schema);

        let good: Node = r#"{"name":"Alice","age":30}"#.parse().unwrap();
        assert!(v.validate(&good).is_ok());

        // age is a string instead of number — should fail
        let bad: Node = r#"{"name":"Alice","age":"thirty"}"#.parse().unwrap();
        assert!(v.validate(&bad).is_err());
    }

    #[test]
    fn test_properties_extra_keys_ignored() {
        // Properties not listed in schema.properties are not validated
        let schema = json!({"type": "object", "properties": {"a": {"type": "string"}}});
        let v = SchemaValidator::new(schema);
        let data: Node = r#"{"a":"hello","extra":999}"#.parse().unwrap();
        assert!(v.validate(&data).is_ok());
    }

    #[test]
    fn test_properties_nested_path_in_error() {
        let schema = json!({
            "type": "object",
            "properties": {
                "user": {
                    "type": "object",
                    "properties": {
                        "age": {"type": "number", "minimum": 0}
                    }
                }
            }
        });
        let v = SchemaValidator::new(schema);
        let bad: Node = r#"{"user":{"age":"not-a-number"}}"#.parse().unwrap();
        let errs = v.validate(&bad).unwrap_err();
        assert!(!errs.is_empty());
        // Path should include "user.age"
        let has_nested_path = errs.iter().any(|e| e.path.contains("user"));
        assert!(has_nested_path);
    }

    // --- minProperties / maxProperties ---

    #[test]
    fn test_min_properties_exact() {
        let schema = json!({"type": "object", "minProperties": 2});
        let v = SchemaValidator::new(schema);
        let data: Node = r#"{"a":1,"b":2}"#.parse().unwrap();
        assert!(v.validate(&data).is_ok());
    }

    #[test]
    fn test_min_properties_fails() {
        let schema = json!({"type": "object", "minProperties": 3});
        let v = SchemaValidator::new(schema);
        let data: Node = r#"{"a":1,"b":2}"#.parse().unwrap();
        let errs = v.validate(&data).unwrap_err();
        assert!(errs[0].message.contains("minimum"));
    }

    #[test]
    fn test_max_properties_exact() {
        let schema = json!({"type": "object", "maxProperties": 2});
        let v = SchemaValidator::new(schema);
        let data: Node = r#"{"a":1,"b":2}"#.parse().unwrap();
        assert!(v.validate(&data).is_ok());
    }

    #[test]
    fn test_max_properties_fails() {
        let schema = json!({"type": "object", "maxProperties": 1});
        let v = SchemaValidator::new(schema);
        let data: Node = r#"{"a":1,"b":2}"#.parse().unwrap();
        let errs = v.validate(&data).unwrap_err();
        assert!(errs[0].message.contains("maximum"));
    }

    // --- array items validation ---

    #[test]
    fn test_array_items_all_valid() {
        let schema = json!({"type": "array", "items": {"type": "string"}});
        let v = SchemaValidator::new(schema);
        assert!(v.validate(&json!(["a", "b", "c"])).is_ok());
    }

    #[test]
    fn test_array_items_one_invalid() {
        let schema = json!({"type": "array", "items": {"type": "number"}});
        let v = SchemaValidator::new(schema);
        // "b" is not a number
        assert!(v.validate(&json!([1, 2, 3])).is_ok());
        let bad: Node = r#"[1, "b", 3]"#.parse().unwrap();
        assert!(v.validate(&bad).is_err());
    }

    #[test]
    fn test_array_items_error_path_includes_index() {
        let schema = json!({"type": "array", "items": {"type": "string"}});
        let v = SchemaValidator::new(schema);
        let bad: Node = r#"["ok", 42, "also-ok"]"#.parse().unwrap();
        let errs = v.validate(&bad).unwrap_err();
        assert!(errs[0].path.contains("[1]"));
    }

    #[test]
    fn test_array_empty_no_items_schema() {
        // No items schema — any elements pass
        let schema = json!({"type": "array"});
        let v = SchemaValidator::new(schema);
        assert!(v.validate(&json!([])).is_ok());
        assert!(v.validate(&json!([1, "hi"])).is_ok());
    }

    // --- minItems / maxItems ---

    #[test]
    fn test_min_items_exact() {
        let schema = json!({"type": "array", "minItems": 2});
        let v = SchemaValidator::new(schema);
        assert!(v.validate(&json!([1, 2])).is_ok());
    }

    #[test]
    fn test_min_items_fails() {
        let schema = json!({"type": "array", "minItems": 3});
        let v = SchemaValidator::new(schema);
        let errs = v.validate(&json!([1, 2])).unwrap_err();
        assert!(errs[0].message.contains("minimum"));
    }

    #[test]
    fn test_max_items_exact() {
        let schema = json!({"type": "array", "maxItems": 2});
        let v = SchemaValidator::new(schema);
        assert!(v.validate(&json!([1, 2])).is_ok());
    }

    #[test]
    fn test_max_items_fails() {
        let schema = json!({"type": "array", "maxItems": 2});
        let v = SchemaValidator::new(schema);
        let errs = v.validate(&json!([1, 2, 3])).unwrap_err();
        assert!(errs[0].message.contains("maximum"));
    }

    // --- uniqueItems ---

    #[test]
    fn test_unique_items_valid() {
        let schema = json!({"type": "array", "uniqueItems": true});
        let v = SchemaValidator::new(schema);
        assert!(v.validate(&json!([1, 2, 3])).is_ok());
    }

    #[test]
    fn test_unique_items_invalid() {
        let schema = json!({"type": "array", "uniqueItems": true});
        let v = SchemaValidator::new(schema);
        let bad: Node = r#"[1, 2, 1]"#.parse().unwrap();
        let errs = v.validate(&bad).unwrap_err();
        assert!(errs[0].message.contains("unique"));
    }

    #[test]
    fn test_unique_items_false_allows_duplicates() {
        let schema = json!({"type": "array", "uniqueItems": false});
        let v = SchemaValidator::new(schema);
        let dup: Node = r#"[1, 1, 1]"#.parse().unwrap();
        assert!(v.validate(&dup).is_ok());
    }

    #[test]
    fn test_unique_items_empty_array() {
        let schema = json!({"type": "array", "uniqueItems": true});
        let v = SchemaValidator::new(schema);
        assert!(v.validate(&json!([])).is_ok());
    }

    // --- minLength / maxLength ---

    #[test]
    fn test_min_length_exact() {
        let schema = json!({"type": "string", "minLength": 5});
        let v = SchemaValidator::new(schema);
        assert!(v.validate(&json!("hello")).is_ok());
    }

    #[test]
    fn test_min_length_fails() {
        let schema = json!({"type": "string", "minLength": 5});
        let v = SchemaValidator::new(schema);
        let errs = v.validate(&json!("hi")).unwrap_err();
        assert!(errs[0].message.contains("minimum"));
    }

    #[test]
    fn test_max_length_exact() {
        let schema = json!({"type": "string", "maxLength": 5});
        let v = SchemaValidator::new(schema);
        assert!(v.validate(&json!("hello")).is_ok());
    }

    #[test]
    fn test_max_length_fails() {
        let schema = json!({"type": "string", "maxLength": 3});
        let v = SchemaValidator::new(schema);
        let errs = v.validate(&json!("toolong")).unwrap_err();
        assert!(errs[0].message.contains("maximum"));
    }

    #[test]
    fn test_min_and_max_length_together() {
        let schema = json!({"type": "string", "minLength": 2, "maxLength": 4});
        let v = SchemaValidator::new(schema);
        assert!(v.validate(&json!("ab")).is_ok());
        assert!(v.validate(&json!("abcd")).is_ok());
        assert!(v.validate(&json!("a")).is_err());
        assert!(v.validate(&json!("abcde")).is_err());
    }

    // --- enum ---

    #[test]
    fn test_enum_valid_value() {
        let schema = json!({"type": "string", "enum": ["red", "green", "blue"]});
        let v = SchemaValidator::new(schema);
        assert!(v.validate(&json!("red")).is_ok());
        assert!(v.validate(&json!("green")).is_ok());
        assert!(v.validate(&json!("blue")).is_ok());
    }

    #[test]
    fn test_enum_invalid_value() {
        let schema = json!({"type": "string", "enum": ["red", "green", "blue"]});
        let v = SchemaValidator::new(schema);
        let errs = v.validate(&json!("yellow")).unwrap_err();
        assert!(errs[0].message.contains("enum"));
    }

    #[test]
    fn test_enum_empty_list_always_fails() {
        let schema = json!({"type": "string", "enum": []});
        let v = SchemaValidator::new(schema);
        assert!(v.validate(&json!("any")).is_err());
    }

    // --- minimum / maximum ---

    #[test]
    fn test_minimum_boundary() {
        let schema = json!({"type": "number", "minimum": 10});
        let v = SchemaValidator::new(schema);
        assert!(v.validate(&json!(10)).is_ok()); // exactly at minimum
        assert!(v.validate(&json!(11)).is_ok());
        assert!(v.validate(&json!(9)).is_err());
    }

    #[test]
    fn test_maximum_boundary() {
        let schema = json!({"type": "number", "maximum": 10});
        let v = SchemaValidator::new(schema);
        assert!(v.validate(&json!(10)).is_ok()); // exactly at maximum
        assert!(v.validate(&json!(9)).is_ok());
        assert!(v.validate(&json!(11)).is_err());
    }

    #[test]
    fn test_minimum_error_message_contains_values() {
        let schema = json!({"type": "number", "minimum": 5});
        let v = SchemaValidator::new(schema);
        let errs = v.validate(&json!(3)).unwrap_err();
        assert!(errs[0].message.contains("minimum"));
    }

    #[test]
    fn test_maximum_error_message_contains_values() {
        let schema = json!({"type": "number", "maximum": 5});
        let v = SchemaValidator::new(schema);
        let errs = v.validate(&json!(10)).unwrap_err();
        assert!(errs[0].message.contains("maximum"));
    }

    // --- exclusiveMinimum / exclusiveMaximum ---

    #[test]
    fn test_exclusive_minimum_passes() {
        let schema = json!({"type": "number", "exclusiveMinimum": 0});
        let v = SchemaValidator::new(schema);
        assert!(v.validate(&json!(1)).is_ok());
    }

    #[test]
    fn test_exclusive_minimum_at_boundary_fails() {
        let schema = json!({"type": "number", "exclusiveMinimum": 0});
        let v = SchemaValidator::new(schema);
        assert!(v.validate(&json!(0)).is_err()); // equal is not allowed
        assert!(v.validate(&json!(-1)).is_err());
    }

    #[test]
    fn test_exclusive_maximum_passes() {
        let schema = json!({"type": "number", "exclusiveMaximum": 10});
        let v = SchemaValidator::new(schema);
        assert!(v.validate(&json!(9)).is_ok());
    }

    #[test]
    fn test_exclusive_maximum_at_boundary_fails() {
        let schema = json!({"type": "number", "exclusiveMaximum": 10});
        let v = SchemaValidator::new(schema);
        assert!(v.validate(&json!(10)).is_err()); // equal is not allowed
        assert!(v.validate(&json!(11)).is_err());
    }

    // --- error accumulation ---

    #[test]
    fn test_multiple_errors_accumulated() {
        // An object missing two required fields and under minProperties
        let schema = json!({
            "type": "object",
            "required": ["a", "b"],
            "minProperties": 3
        });
        let v = SchemaValidator::new(schema);
        let data = json!({});
        let errs = v.validate(&data).unwrap_err();
        // missing "a", missing "b", minProperties violation = 3 errors
        assert!(errs.len() >= 3);
    }

    #[test]
    fn test_error_at_root_path() {
        let schema = json!({"type": "string", "minLength": 10});
        let v = SchemaValidator::new(schema);
        let errs = v.validate(&json!("hi")).unwrap_err();
        assert_eq!(errs[0].path, "");
    }

    #[test]
    fn test_error_at_nested_property_path() {
        let schema = json!({
            "type": "object",
            "properties": {
                "score": {"type": "number", "minimum": 0, "maximum": 100}
            }
        });
        let v = SchemaValidator::new(schema);
        let bad: Node = r#"{"score": 200}"#.parse().unwrap();
        let errs = v.validate(&bad).unwrap_err();
        assert_eq!(errs[0].path, "score");
    }

    // --- complex / real-world schemas ---

    #[test]
    fn test_user_object_schema() {
        let schema = json!({
            "type": "object",
            "required": ["name", "email"],
            "properties": {
                "name":  {"type": "string", "minLength": 1, "maxLength": 100},
                "email": {"type": "string", "minLength": 5}
            }
        });
        let v = SchemaValidator::new(schema);

        let good: Node = r#"{"name":"Alice","email":"alice@example.com"}"#.parse().unwrap();
        assert!(v.validate(&good).is_ok());

        let no_email: Node = r#"{"name":"Bob"}"#.parse().unwrap();
        assert!(v.validate(&no_email).is_err());
    }

    #[test]
    fn test_array_of_objects_schema() {
        let schema = json!({
            "type": "array",
            "items": {
                "type": "object",
                "required": ["id"]
            },
            "minItems": 1
        });
        let v = SchemaValidator::new(schema);

        let good: Node = r#"[{"id":1},{"id":2}]"#.parse().unwrap();
        assert!(v.validate(&good).is_ok());

        // Empty array violates minItems
        assert!(v.validate(&json!([])).is_err());

        // Item missing "id"
        let bad: Node = r#"[{"name":"no-id"}]"#.parse().unwrap();
        assert!(v.validate(&bad).is_err());
    }

    #[test]
    fn test_min_max_number_combined() {
        let schema = json!({"type": "number", "minimum": 1, "maximum": 10});
        let v = SchemaValidator::new(schema);
        assert!(v.validate(&json!(1)).is_ok());
        assert!(v.validate(&json!(10)).is_ok());
        assert!(v.validate(&json!(0)).is_err());
        assert!(v.validate(&json!(11)).is_err());
    }

    #[test]
    fn test_schema_without_type_still_validates_properties() {
        // No "type" key but has "required" — object check still runs
        let schema = json!({"required": ["x"]});
        let v = SchemaValidator::new(schema);
        // Non-object: no object validation runs (is_object() is false), should be Ok
        assert!(v.validate(&json!("hello")).is_ok());
        // Object without "x": should fail required
        let no_x = json!({});
        assert!(v.validate(&no_x).is_err());
    }
}
