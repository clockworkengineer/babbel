//! Universal Zero-Dependency JSON Schema Validator & Inference Engine (Draft 7 / Draft 2020-12).
//!
//! Validates universal [`Value`] documents from any format (JSON, YAML, TOML, KDL, CBOR, BSON, etc.)
//! against standard JSON Schema definitions, and infers schemas from sample documents.

pub mod infer;
pub mod regex;

use crate::error::BabbelError;
use crate::model::Value;
use alloc::boxed::Box;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

pub use infer::infer_schema;
pub use regex::{Regex, regex_is_full_match, regex_is_match};

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
    Pattern {
        raw: String,
        compiled: Option<Regex>,
    },
    Format(String),
    Required(Vec<String>),
    Properties(Vec<(String, CompiledSchema)>),
    PatternProperties(Vec<(String, Option<Regex>, CompiledSchema)>),
    PropertyNames(Box<CompiledSchema>),
    AdditionalPropertiesBool(bool),
    AdditionalPropertiesSchema(Box<CompiledSchema>),
    DependentRequired(Vec<(String, Vec<String>)>),
    DependentSchemas(Vec<(String, CompiledSchema)>),
    MinProperties(usize),
    MaxProperties(usize),
    PrefixItems(Vec<CompiledSchema>),
    Items {
        start_index: usize,
        schema: Box<CompiledSchema>,
    },
    Contains {
        schema: Box<CompiledSchema>,
        min: usize,
        max: Option<usize>,
    },
    MinItems(usize),
    MaxItems(usize),
    UniqueItems(bool),
    IfThenElse {
        cond: Box<CompiledSchema>,
        then_schema: Option<Box<CompiledSchema>>,
        else_schema: Option<Box<CompiledSchema>>,
    },
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
        Self::compile_with_root(schema, schema, 0)
    }

    /// Infer a Draft 2020-12 JSON Schema [`Value`] from a sample [`Value`].
    pub fn infer(sample: &Value) -> Value {
        infer::infer_schema(sample)
    }

    fn compile_with_root(
        schema: &Value,
        root: &Value,
        ref_depth: usize,
    ) -> Result<Self, BabbelError> {
        if ref_depth > 32 {
            return Err(BabbelError::syntax(
                "JSON Schema $ref recursion depth limit exceeded",
            ));
        }

        match schema {
            Value::Bool(true) => Ok(Self { rules: Vec::new() }),
            Value::Bool(false) => Ok(Self {
                rules: alloc::vec![ValidationRule::Not(Box::new(Self { rules: Vec::new() }))],
            }),
            Value::Object(entries) => {
                let mut rules = Vec::new();

                // '$ref' resolution against root schema
                if let Some(ref_str) = get_val(entries, "$ref").and_then(|v| v.as_str()) {
                    if ref_str == "#" {
                        let resolved = Self::compile_with_root(root, root, ref_depth + 1)?;
                        rules.extend(resolved.rules);
                    } else if let Some(ptr) = ref_str.strip_prefix('#') {
                        if let Some(target) = root.pointer(ptr) {
                            let resolved = Self::compile_with_root(target, root, ref_depth + 1)?;
                            rules.extend(resolved.rules);
                        } else {
                            return Err(BabbelError::syntax(format!(
                                "unresolved JSON Schema $ref '{}'",
                                ref_str
                            )));
                        }
                    }
                }

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
                    if v > 0.0 {
                        rules.push(ValidationRule::MultipleOf(v));
                    }
                }

                // String constraints
                if let Some(v) = get_val(entries, "minLength").and_then(|v| v.as_u64()) {
                    rules.push(ValidationRule::MinLength(v as usize));
                }
                if let Some(v) = get_val(entries, "maxLength").and_then(|v| v.as_u64()) {
                    rules.push(ValidationRule::MaxLength(v as usize));
                }
                if let Some(s) = get_val(entries, "pattern").and_then(|v| v.as_str()) {
                    let compiled = Regex::compile(s).ok();
                    rules.push(ValidationRule::Pattern {
                        raw: s.to_string(),
                        compiled,
                    });
                }
                if let Some(s) = get_val(entries, "format").and_then(|v| v.as_str()) {
                    rules.push(ValidationRule::Format(s.to_string()));
                }

                // Object constraints
                if let Some(Value::Array(items)) = get_val(entries, "required") {
                    let req: Vec<String> = items
                        .iter()
                        .filter_map(|i| i.as_str().map(|s| s.to_string()))
                        .collect();
                    rules.push(ValidationRule::Required(req));
                }

                if let Some(Value::Object(props)) = get_val(entries, "properties") {
                    let mut prop_rules = Vec::new();
                    for (k, sub) in props {
                        let compiled = Self::compile_with_root(sub, root, ref_depth)?;
                        prop_rules.push((k.clone(), compiled));
                    }
                    rules.push(ValidationRule::Properties(prop_rules));
                }

                if let Some(Value::Object(pat_props)) = get_val(entries, "patternProperties") {
                    let mut pat_rules = Vec::new();
                    for (pat, sub) in pat_props {
                        let re = Regex::compile(pat).ok();
                        let compiled = Self::compile_with_root(sub, root, ref_depth)?;
                        pat_rules.push((pat.clone(), re, compiled));
                    }
                    rules.push(ValidationRule::PatternProperties(pat_rules));
                }

                if let Some(prop_names) = get_val(entries, "propertyNames") {
                    let compiled = Self::compile_with_root(prop_names, root, ref_depth)?;
                    rules.push(ValidationRule::PropertyNames(Box::new(compiled)));
                }

                if let Some(add_prop) = get_val(entries, "additionalProperties") {
                    match add_prop {
                        Value::Bool(b) => rules.push(ValidationRule::AdditionalPropertiesBool(*b)),
                        sub @ Value::Object(_) => {
                            let compiled = Self::compile_with_root(sub, root, ref_depth)?;
                            rules.push(ValidationRule::AdditionalPropertiesSchema(Box::new(
                                compiled,
                            )));
                        }
                        _ => {}
                    }
                }

                if let Some(Value::Object(dep_req)) = get_val(entries, "dependentRequired") {
                    let mut list = Vec::new();
                    for (k, arr_val) in dep_req {
                        if let Value::Array(arr) = arr_val {
                            let req_keys: Vec<String> = arr
                                .iter()
                                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                                .collect();
                            list.push((k.clone(), req_keys));
                        }
                    }
                    rules.push(ValidationRule::DependentRequired(list));
                }

                if let Some(Value::Object(dep_schemas)) = get_val(entries, "dependentSchemas") {
                    let mut list = Vec::new();
                    for (k, sub) in dep_schemas {
                        let compiled = Self::compile_with_root(sub, root, ref_depth)?;
                        list.push((k.clone(), compiled));
                    }
                    rules.push(ValidationRule::DependentSchemas(list));
                }

                if let Some(v) = get_val(entries, "minProperties").and_then(|v| v.as_u64()) {
                    rules.push(ValidationRule::MinProperties(v as usize));
                }
                if let Some(v) = get_val(entries, "maxProperties").and_then(|v| v.as_u64()) {
                    rules.push(ValidationRule::MaxProperties(v as usize));
                }

                // Array constraints: prefixItems & items
                let mut prefix_len = 0usize;
                if let Some(Value::Array(prefix_arr)) = get_val(entries, "prefixItems") {
                    let mut prefixes = Vec::with_capacity(prefix_arr.len());
                    for sub in prefix_arr {
                        prefixes.push(Self::compile_with_root(sub, root, ref_depth)?);
                    }
                    prefix_len = prefixes.len();
                    rules.push(ValidationRule::PrefixItems(prefixes));
                }

                if let Some(items_val) = get_val(entries, "items") {
                    let compiled = Self::compile_with_root(items_val, root, ref_depth)?;
                    rules.push(ValidationRule::Items {
                        start_index: prefix_len,
                        schema: Box::new(compiled),
                    });
                }

                if let Some(contains_val) = get_val(entries, "contains") {
                    let compiled = Self::compile_with_root(contains_val, root, ref_depth)?;
                    let min = get_val(entries, "minContains")
                        .and_then(|v| v.as_u64())
                        .map(|u| u as usize)
                        .unwrap_or(1);
                    let max = get_val(entries, "maxContains")
                        .and_then(|v| v.as_u64())
                        .map(|u| u as usize);
                    rules.push(ValidationRule::Contains {
                        schema: Box::new(compiled),
                        min,
                        max,
                    });
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

                // Conditional applicators: if / then / else
                if let Some(if_val) = get_val(entries, "if") {
                    let cond = Box::new(Self::compile_with_root(if_val, root, ref_depth)?);
                    let then_schema = match get_val(entries, "then") {
                        Some(t) => Some(Box::new(Self::compile_with_root(t, root, ref_depth)?)),
                        None => None,
                    };
                    let else_schema = match get_val(entries, "else") {
                        Some(e) => Some(Box::new(Self::compile_with_root(e, root, ref_depth)?)),
                        None => None,
                    };
                    rules.push(ValidationRule::IfThenElse {
                        cond,
                        then_schema,
                        else_schema,
                    });
                }

                // Combinators
                if let Some(Value::Array(items)) = get_val(entries, "allOf") {
                    let mut schemas = Vec::new();
                    for item in items {
                        schemas.push(Self::compile_with_root(item, root, ref_depth)?);
                    }
                    rules.push(ValidationRule::AllOf(schemas));
                }
                if let Some(Value::Array(items)) = get_val(entries, "anyOf") {
                    let mut schemas = Vec::new();
                    for item in items {
                        schemas.push(Self::compile_with_root(item, root, ref_depth)?);
                    }
                    rules.push(ValidationRule::AnyOf(schemas));
                }
                if let Some(Value::Array(items)) = get_val(entries, "oneOf") {
                    let mut schemas = Vec::new();
                    for item in items {
                        schemas.push(Self::compile_with_root(item, root, ref_depth)?);
                    }
                    rules.push(ValidationRule::OneOf(schemas));
                }
                if let Some(sub) = get_val(entries, "not") {
                    let compiled = Self::compile_with_root(sub, root, ref_depth)?;
                    rules.push(ValidationRule::Not(Box::new(compiled)));
                }

                Ok(Self { rules })
            }
            _ => Err(BabbelError::syntax(
                "JSON Schema must be a boolean or an object",
            )),
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

    fn is_property_matched_by_siblings(&self, key: &str) -> bool {
        for r in &self.rules {
            match r {
                ValidationRule::Properties(props) => {
                    if props.iter().any(|(k, _)| k == key) {
                        return true;
                    }
                }
                ValidationRule::PatternProperties(pat_props) => {
                    for (raw, re, _) in pat_props {
                        let matched = match re {
                            Some(compiled) => compiled.is_match(key),
                            None => key.contains(raw.as_str()),
                        };
                        if matched {
                            return true;
                        }
                    }
                }
                _ => {}
            }
        }
        false
    }

    fn validate_internal(
        &self,
        val: &Value,
        pointer: &str,
        errors: &mut Vec<SchemaValidationError>,
    ) {
        for rule in &self.rules {
            match rule {
                ValidationRule::Type(expected_types) => {
                    if !expected_types.iter().any(|t| t.matches(val)) {
                        errors.push(SchemaValidationError {
                            pointer: pointer.to_string(),
                            keyword: "type",
                            message: "value does not match expected type(s)".to_string(),
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
                                message: format!(
                                    "{} is less than or equal to exclusiveMinimum {}",
                                    num, min
                                ),
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
                                message: format!(
                                    "{} is greater than or equal to exclusiveMaximum {}",
                                    num, max
                                ),
                            });
                        }
                    }
                }
                ValidationRule::MultipleOf(factor) => {
                    if let Some(num) = val.as_f64() {
                        let div = num / factor;
                        let remainder = div - (div as i64 as f64);
                        let abs_rem = if remainder < 0.0 {
                            -remainder
                        } else {
                            remainder
                        };
                        if abs_rem > 1e-9 && (1.0 - abs_rem) > 1e-9 {
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
                ValidationRule::Pattern { raw, compiled } => {
                    if let Some(s) = val.as_str() {
                        let matched = match compiled {
                            Some(re) => re.is_match(s),
                            None => s.contains(raw.as_str()),
                        };
                        if !matched {
                            errors.push(SchemaValidationError {
                                pointer: pointer.to_string(),
                                keyword: "pattern",
                                message: format!("string does not match pattern '{}'", raw),
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
                ValidationRule::PatternProperties(pat_props) => {
                    if let Value::Object(entries) = val {
                        for (k, v) in entries {
                            for (raw, re, sub_schema) in pat_props {
                                let matched = match re {
                                    Some(compiled) => compiled.is_match(k),
                                    None => k.contains(raw.as_str()),
                                };
                                if matched {
                                    let sub_ptr = format!("{}/{}", pointer, k);
                                    sub_schema.validate_internal(v, &sub_ptr, errors);
                                }
                            }
                        }
                    }
                }
                ValidationRule::PropertyNames(name_schema) => {
                    if let Value::Object(entries) = val {
                        for (k, _) in entries {
                            let key_val = Value::String(k.clone());
                            let sub_ptr = format!("{}/{}", pointer, k);
                            let mut key_errors = Vec::new();
                            name_schema.validate_internal(&key_val, &sub_ptr, &mut key_errors);
                            if !key_errors.is_empty() {
                                errors.push(SchemaValidationError {
                                    pointer: sub_ptr,
                                    keyword: "propertyNames",
                                    message: format!(
                                        "property name '{}' does not match propertyNames schema",
                                        k
                                    ),
                                });
                            }
                        }
                    }
                }
                ValidationRule::AdditionalPropertiesBool(allowed) => {
                    if !*allowed {
                        if let Value::Object(entries) = val {
                            for (k, _) in entries {
                                if !self.is_property_matched_by_siblings(k) {
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
                        for (k, v) in entries {
                            if !self.is_property_matched_by_siblings(k) {
                                let sub_ptr = format!("{}/{}", pointer, k);
                                sub_schema.validate_internal(v, &sub_ptr, errors);
                            }
                        }
                    }
                }
                ValidationRule::DependentRequired(deps) => {
                    if let Value::Object(entries) = val {
                        for (prop, required_list) in deps {
                            if entries.iter().any(|(k, _)| k == prop) {
                                for req in required_list {
                                    if !entries.iter().any(|(k, _)| k == req) {
                                        errors.push(SchemaValidationError {
                                            pointer: pointer.to_string(),
                                            keyword: "dependentRequired",
                                            message: format!(
                                                "property '{}' requires '{}' to be present",
                                                prop, req
                                            ),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
                ValidationRule::DependentSchemas(deps) => {
                    if let Value::Object(entries) = val {
                        for (prop, sub_schema) in deps {
                            if entries.iter().any(|(k, _)| k == prop) {
                                sub_schema.validate_internal(val, pointer, errors);
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
                                message: format!(
                                    "object property count {} is less than minimum {}",
                                    entries.len(),
                                    min
                                ),
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
                                message: format!(
                                    "object property count {} is greater than maximum {}",
                                    entries.len(),
                                    max
                                ),
                            });
                        }
                    }
                }
                ValidationRule::PrefixItems(prefixes) => {
                    if let Value::Array(items) = val {
                        for (idx, (item, schema)) in items.iter().zip(prefixes.iter()).enumerate() {
                            let sub_ptr = format!("{}/{}", pointer, idx);
                            schema.validate_internal(item, &sub_ptr, errors);
                        }
                    }
                }
                ValidationRule::Items {
                    start_index,
                    schema,
                } => {
                    if let Value::Array(items) = val {
                        for (idx, item) in items.iter().enumerate().skip(*start_index) {
                            let sub_ptr = format!("{}/{}", pointer, idx);
                            schema.validate_internal(item, &sub_ptr, errors);
                        }
                    }
                }
                ValidationRule::Contains { schema, min, max } => {
                    if let Value::Array(items) = val {
                        let matched_count =
                            items.iter().filter(|item| schema.is_valid(item)).count();
                        if matched_count < *min {
                            errors.push(SchemaValidationError {
                                pointer: pointer.to_string(),
                                keyword: "contains",
                                message: format!(
                                    "array contains {} matching items (minimum required: {})",
                                    matched_count, min
                                ),
                            });
                        }
                        if let Some(max_c) = max {
                            if matched_count > *max_c {
                                errors.push(SchemaValidationError {
                                    pointer: pointer.to_string(),
                                    keyword: "maxContains",
                                    message: format!(
                                        "array contains {} matching items (maximum allowed: {})",
                                        matched_count, max_c
                                    ),
                                });
                            }
                        }
                    }
                }
                ValidationRule::MinItems(min) => {
                    if let Value::Array(items) = val {
                        if items.len() < *min {
                            errors.push(SchemaValidationError {
                                pointer: pointer.to_string(),
                                keyword: "minItems",
                                message: format!(
                                    "array length {} is less than minimum {}",
                                    items.len(),
                                    min
                                ),
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
                                message: format!(
                                    "array length {} is greater than maximum {}",
                                    items.len(),
                                    max
                                ),
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
                                            message: format!(
                                                "duplicate item detected at index {} and {}",
                                                i, j
                                            ),
                                        });
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                ValidationRule::IfThenElse {
                    cond,
                    then_schema,
                    else_schema,
                } => {
                    if cond.is_valid(val) {
                        if let Some(then_s) = then_schema {
                            then_s.validate_internal(val, pointer, errors);
                        }
                    } else if let Some(else_s) = else_schema {
                        else_s.validate_internal(val, pointer, errors);
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
                            message: format!(
                                "value matched {} schemas in oneOf (expected exactly 1)",
                                valid_count
                            ),
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

fn validate_format_string(format: &str, s: &str) -> bool {
    match format {
        "email" => {
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
        "uuid" => {
            s.len() == 36
                && s.bytes().enumerate().all(|(i, b)| {
                    if i == 8 || i == 13 || i == 18 || i == 23 {
                        b == b'-'
                    } else {
                        b.is_ascii_hexdigit()
                    }
                })
        }
        "ipv4" => {
            let parts: Vec<&str> = s.split('.').collect();
            parts.len() == 4
                && parts
                    .iter()
                    .all(|p| !p.is_empty() && p.parse::<u8>().is_ok())
        }
        "ipv6" => {
            if !s.contains(':') {
                return false;
            }
            let double_colons = s.matches("::").count();
            if double_colons > 1 {
                return false;
            }
            let groups: Vec<&str> = s.split(':').filter(|g| !g.is_empty()).collect();
            if double_colons == 0 && groups.len() != 8 {
                return false;
            }
            if groups.len() > 8 {
                return false;
            }
            groups
                .iter()
                .all(|g| g.len() <= 4 && g.bytes().all(|b| b.is_ascii_hexdigit()))
        }
        "hostname" => {
            !s.is_empty()
                && s.len() <= 253
                && s.split('.').all(|label| {
                    !label.is_empty()
                        && label.len() <= 63
                        && !label.starts_with('-')
                        && !label.ends_with('-')
                        && label
                            .bytes()
                            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
                })
        }
        "date" => {
            s.len() == 10
                && s.as_bytes()[4] == b'-'
                && s.as_bytes()[7] == b'-'
                && s[0..4].parse::<u16>().is_ok()
                && s[5..7]
                    .parse::<u8>()
                    .map_or(false, |m| (1..=12).contains(&m))
                && s[8..10]
                    .parse::<u8>()
                    .map_or(false, |d| (1..=31).contains(&d))
        }
        "time" => {
            s.len() >= 8
                && s.as_bytes()[2] == b':'
                && s.as_bytes()[5] == b':'
                && s[0..2].parse::<u8>().map_or(false, |h| h <= 23)
                && s[3..5].parse::<u8>().map_or(false, |m| m <= 59)
                && s[6..8].parse::<u8>().map_or(false, |sec| sec <= 60)
        }
        "date-time" => {
            s.len() >= 19
                && validate_format_string("date", &s[0..10])
                && matches!(s.as_bytes()[10], b'T' | b't' | b' ')
                && validate_format_string("time", &s[11..])
        }
        "duration" => s.starts_with('P') && s.len() >= 2,
        "uri" => s.contains("://") && !s.contains(' '),
        "uri-reference" => !s.contains(' '),
        "json-pointer" => s.is_empty() || s.starts_with('/'),
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
            (
                "required".to_string(),
                Value::Array(vec![
                    Value::String("name".to_string()),
                    Value::String("age".to_string()),
                ]),
            ),
            (
                "properties".to_string(),
                Value::Object(vec![
                    (
                        "name".to_string(),
                        Value::Object(vec![(
                            "type".to_string(),
                            Value::String("string".to_string()),
                        )]),
                    ),
                    (
                        "age".to_string(),
                        Value::Object(vec![
                            ("type".to_string(), Value::String("integer".to_string())),
                            ("minimum".to_string(), Value::Integer(0)),
                        ]),
                    ),
                ]),
            ),
        ]);

        let compiled = CompiledSchema::compile(&schema).unwrap();

        let valid = Value::Object(vec![
            ("name".to_string(), Value::String("Alice".to_string())),
            ("age".to_string(), Value::Integer(30)),
        ]);
        assert!(compiled.is_valid(&valid));

        let missing_age = Value::Object(vec![(
            "name".to_string(),
            Value::String("Alice".to_string()),
        )]);
        assert!(!compiled.is_valid(&missing_age));

        let invalid_age = Value::Object(vec![
            ("name".to_string(), Value::String("Alice".to_string())),
            ("age".to_string(), Value::Integer(-5)),
        ]);
        assert!(!compiled.is_valid(&invalid_age));
    }

    #[test]
    fn test_schema_combinators() {
        let schema = Value::Object(vec![(
            "oneOf".to_string(),
            Value::Array(vec![
                Value::Object(vec![(
                    "type".to_string(),
                    Value::String("string".to_string()),
                )]),
                Value::Object(vec![(
                    "type".to_string(),
                    Value::String("number".to_string()),
                )]),
            ]),
        )]);

        let compiled = CompiledSchema::compile(&schema).unwrap();
        assert!(compiled.is_valid(&Value::String("test".to_string())));
        assert!(compiled.is_valid(&Value::Integer(42)));
        assert!(!compiled.is_valid(&Value::Bool(true)));
    }

    #[test]
    fn test_schema_defs_and_refs() {
        let schema = Value::Object(vec![
            (
                "$defs".to_string(),
                Value::Object(vec![(
                    "positiveInt".to_string(),
                    Value::Object(vec![
                        ("type".to_string(), Value::String("integer".to_string())),
                        ("minimum".to_string(), Value::Integer(1)),
                    ]),
                )]),
            ),
            ("type".to_string(), Value::String("object".to_string())),
            (
                "properties".to_string(),
                Value::Object(vec![(
                    "count".to_string(),
                    Value::Object(vec![(
                        "$ref".to_string(),
                        Value::String("#/$defs/positiveInt".to_string()),
                    )]),
                )]),
            ),
        ]);

        let compiled = CompiledSchema::compile(&schema).unwrap();
        assert!(compiled.is_valid(&Value::Object(vec![(
            "count".to_string(),
            Value::Integer(5)
        )])));
        assert!(!compiled.is_valid(&Value::Object(vec![(
            "count".to_string(),
            Value::Integer(0)
        )])));
    }

    #[test]
    fn test_schema_if_then_else_and_prefix_items() {
        let cond_schema = Value::Object(vec![
            (
                "if".to_string(),
                Value::Object(vec![(
                    "properties".to_string(),
                    Value::Object(vec![(
                        "country".to_string(),
                        Value::Object(vec![("const".to_string(), Value::String("US".to_string()))]),
                    )]),
                )]),
            ),
            (
                "then".to_string(),
                Value::Object(vec![(
                    "properties".to_string(),
                    Value::Object(vec![(
                        "zip".to_string(),
                        Value::Object(vec![(
                            "pattern".to_string(),
                            Value::String(r"^\d{5}$".to_string()),
                        )]),
                    )]),
                )]),
            ),
        ]);

        let compiled = CompiledSchema::compile(&cond_schema).unwrap();
        let valid_us = Value::Object(vec![
            ("country".to_string(), Value::String("US".to_string())),
            ("zip".to_string(), Value::String("90210".to_string())),
        ]);
        let invalid_us = Value::Object(vec![
            ("country".to_string(), Value::String("US".to_string())),
            ("zip".to_string(), Value::String("ABCDE".to_string())),
        ]);
        assert!(compiled.is_valid(&valid_us));
        assert!(!compiled.is_valid(&invalid_us));

        // prefixItems + items
        let tuple_schema = Value::Object(vec![
            ("type".to_string(), Value::String("array".to_string())),
            (
                "prefixItems".to_string(),
                Value::Array(vec![
                    Value::Object(vec![(
                        "type".to_string(),
                        Value::String("integer".to_string()),
                    )]),
                    Value::Object(vec![(
                        "type".to_string(),
                        Value::String("string".to_string()),
                    )]),
                ]),
            ),
            (
                "items".to_string(),
                Value::Object(vec![(
                    "type".to_string(),
                    Value::String("boolean".to_string()),
                )]),
            ),
        ]);
        let compiled_tuple = CompiledSchema::compile(&tuple_schema).unwrap();
        assert!(compiled_tuple.is_valid(&Value::Array(vec![
            Value::Integer(1),
            Value::String("ok".to_string()),
            Value::Bool(true),
        ])));
        assert!(!compiled_tuple.is_valid(&Value::Array(vec![
            Value::Integer(1),
            Value::String("ok".to_string()),
            Value::Integer(2),
        ])));
    }

    #[test]
    fn test_schema_inference_roundtrip() {
        let sample = Value::Array(vec![
            Value::Object(vec![
                ("id".to_string(), Value::Integer(1)),
                (
                    "email".to_string(),
                    Value::String("alice@example.com".to_string()),
                ),
                ("nickname".to_string(), Value::String("ally".to_string())),
            ]),
            Value::Object(vec![
                ("id".to_string(), Value::Integer(2)),
                (
                    "email".to_string(),
                    Value::String("bob@example.org".to_string()),
                ),
            ]),
        ]);

        let inferred = CompiledSchema::infer(&sample);
        let compiled = CompiledSchema::compile(&inferred).unwrap();
        assert!(compiled.is_valid(&sample));
    }
}
