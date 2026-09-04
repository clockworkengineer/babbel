//! Macros for convenient JSON construction

/// Constructs a JSON value from a literal syntax
///
/// This macro provides a convenient way to construct JSON structures using
/// a syntax similar to JSON itself. It supports:
/// - Objects: `json!({"key": value})`
/// - Arrays: `json!([value1, value2])`
/// - Strings: `json!("text")`
/// - Numbers: `json!(42)`, `json!(3.14)`
/// - Booleans: `json!(true)`, `json!(false)`
/// - Null: `json!(null)`
///
/// # Examples
///
/// ```
/// use json_lib::{json, Node};
///
/// let value = json!({
///     "name": "Alice",
///     "age": 30,
///     "active": true,
///     "scores": [85, 92, 78]
/// });
///
/// assert_eq!(value["name"].as_str(), Some("Alice"));
/// assert_eq!(value["age"].as_i64(), Some(30));
/// assert_eq!(value["active"].as_bool(), Some(true));
/// ```
///
/// Variables and expressions can be interpolated:
///
/// ```
/// use json_lib::{json, Node};
///
/// let name = "Bob";
/// let age = 25;
/// let value = json!({
///     "name": name,
///     "age": age,
///     "year": 2025
/// });
/// ```
#[macro_export]
macro_rules! json {
    // null
    (null) => {
        $crate::Node::None
    };

    // boolean
    (true) => {
        $crate::Node::Boolean(true)
    };
    (false) => {
        $crate::Node::Boolean(false)
    };

    // array
    ([]) => {
        $crate::Node::Array($crate::__json_vec![])
    };
    ([ $($tt:tt)+ ]) => {
        $crate::Node::Array($crate::__json_vec![$($tt)+])
    };

    // object
    ({}) => {
        $crate::Node::Object($crate::__json_map!())
    };
    ({ $($tt:tt)+ }) => {
        $crate::Node::Object($crate::__json_map!($($tt)+))
    };

    // Any other literal (numbers, strings)
    ($other:expr) => {
        $crate::Node::from($other)
    };
}

// Helper macro for building arrays
#[macro_export]
#[doc(hidden)]
macro_rules! __json_vec {
    () => {
        $crate::__json_vec_new()
    };

    ($($content:tt)+) => {{
        let mut vec = $crate::__json_vec_new();
        $crate::__json_vec_push!(&mut vec, $($content)+);
        vec
    }};
}

// Helper macro for pushing elements into vec
#[macro_export]
#[doc(hidden)]
macro_rules! __json_vec_push {
    ($vec:expr, $elem:expr) => {
        $vec.push($crate::json!($elem))
    };

    ($vec:expr, $elem:expr,) => {
        $crate::__json_vec_push!($vec, $elem)
    };

    ($vec:expr, $elem:expr, $($rest:tt)*) => {
        $crate::__json_vec_push!($vec, $elem);
        $crate::__json_vec_push!($vec, $($rest)*);
    };
}

// Helper macro for building objects/maps
#[macro_export]
#[doc(hidden)]
macro_rules! __json_map {
    () => {
        $crate::__json_map_new()
    };

    ($($content:tt)+) => {{
        let mut map = $crate::__json_map_new();
        $crate::__json_map_insert!(&mut map, $($content)+);
        map
    }};
}

// Helper macro for inserting key-value pairs
#[macro_export]
#[doc(hidden)]
macro_rules! __json_map_insert {
    // Handle key: value pairs (use tt to capture the colon)
    ($map:expr, $key:tt : $value:tt) => {
        $map.insert($key.into(), $crate::json!($value));
    };

    ($map:expr, $key:tt : $value:tt,) => {
        $crate::__json_map_insert!($map, $key : $value)
    };

    ($map:expr, $key:tt : $value:tt, $($rest:tt)*) => {
        $crate::__json_map_insert!($map, $key : $value);
        $crate::__json_map_insert!($map, $($rest)*);
    };
}

// Re-export helper functions at crate root via module
#[cfg(feature = "std")]
use std::collections::HashMap;

#[cfg(not(feature = "std"))]
use alloc::{collections::BTreeMap as HashMap, string::String, vec::Vec};

use crate::Node;

/// Helper function for macro - creates new Vec for JSON arrays
#[doc(hidden)]
#[inline]
pub fn __json_vec_new() -> Vec<Node> {
    Vec::new()
}

/// Helper function for macro - creates new HashMap for JSON objects
#[doc(hidden)]
#[inline]
pub fn __json_map_new() -> HashMap<String, Node> {
    HashMap::new()
}

#[cfg(test)]
mod tests {
    use crate::Node;

    #[test]
    fn test_json_null() {
        let value = json!(null);
        assert!(value.is_null());
    }

    #[test]
    fn test_json_boolean() {
        assert_eq!(json!(true), Node::Boolean(true));
        assert_eq!(json!(false), Node::Boolean(false));
    }

    #[test]
    fn test_json_number() {
        let int = json!(42);
        assert_eq!(int.as_i64(), Some(42));

        let float = json!(3.14);
        assert_eq!(float.as_f64(), Some(3.14));
    }

    #[test]
    fn test_json_string() {
        let s = json!("hello");
        assert_eq!(s.as_str(), Some("hello"));
    }

    #[test]
    fn test_json_array() {
        let arr = json!([1, 2, 3]);
        assert!(arr.is_array());
        assert_eq!(arr.len(), Some(3));
        assert_eq!(arr[0].as_i64(), Some(1));
        assert_eq!(arr[2].as_i64(), Some(3));
    }

    #[test]
    fn test_json_empty_array() {
        let arr = json!([]);
        assert!(arr.is_array());
        assert_eq!(arr.len(), Some(0));
    }

    #[test]
    fn test_json_object() {
        let obj = json!({
            "name": "Alice",
            "age": 30
        });
        assert!(obj.is_object());
        assert_eq!(obj["name"].as_str(), Some("Alice"));
        assert_eq!(obj["age"].as_i64(), Some(30));
    }

    #[test]
    fn test_json_empty_object() {
        let obj = json!({});
        assert!(obj.is_object());
        assert_eq!(obj.len(), Some(0));
    }

    #[test]
    fn test_json_nested() {
        let data = json!({
            "user": {
                "name": "Bob",
                "scores": [85, 92, 78]
            },
            "active": true
        });

        assert_eq!(data["user"]["name"].as_str(), Some("Bob"));
        assert_eq!(data["user"]["scores"][1].as_i64(), Some(92));
        assert_eq!(data["active"].as_bool(), Some(true));
    }

    #[test]
    fn test_json_with_variables() {
        let name = "Charlie";
        let age = 25;

        let obj = json!({
            "name": name,
            "age": age
        });

        assert_eq!(obj["name"].as_str(), Some("Charlie"));
        assert_eq!(obj["age"].as_i64(), Some(25));
    }

    #[test]
    fn test_json_mixed_array() {
        let arr = json!([
            "string",
            42,
            true,
            json!(null),
            [1, 2],
            json!({"key": "value"})
        ]);

        assert_eq!(arr.len(), Some(6));
        assert_eq!(arr[0].as_str(), Some("string"));
        assert_eq!(arr[1].as_i64(), Some(42));
        assert_eq!(arr[2].as_bool(), Some(true));
        assert!(arr[3].is_null());
        assert!(arr[4].is_array());
        assert!(arr[5].is_object());
    }

    // null
    #[test]
    fn test_json_null_is_none_variant() {
        assert_eq!(json!(null), Node::None);
    }

    // booleans
    #[test]
    fn test_json_true_is_boolean_true() {
        assert_eq!(json!(true), Node::Boolean(true));
    }

    #[test]
    fn test_json_false_is_boolean_false() {
        assert_eq!(json!(false), Node::Boolean(false));
    }

    // integers
    #[test]
    fn test_json_integer_zero() {
        assert_eq!(json!(0).as_i64(), Some(0));
    }

    #[test]
    fn test_json_integer_negative() {
        assert_eq!(json!(-99).as_i64(), Some(-99));
    }

    #[test]
    fn test_json_integer_large_positive() {
        assert_eq!(json!(1_000_000).as_i64(), Some(1_000_000));
    }

    // floats
    #[test]
    fn test_json_float_zero() {
        assert_eq!(json!(0.0).as_f64(), Some(0.0));
    }

    #[test]
    fn test_json_float_negative() {
        assert!(json!(-1.5).as_f64().unwrap() < 0.0);
    }

    // strings
    #[test]
    fn test_json_empty_string() {
        assert_eq!(json!("").as_str(), Some(""));
    }

    #[test]
    fn test_json_string_with_spaces() {
        assert_eq!(json!("hello world").as_str(), Some("hello world"));
    }

    #[test]
    fn test_json_string_with_special_chars() {
        assert_eq!(json!("a\nb").as_str(), Some("a\nb"));
    }

    // arrays — structure
    #[test]
    fn test_json_single_element_array() {
        let arr = json!([42]);
        assert_eq!(arr.len(), Some(1));
        assert_eq!(arr[0].as_i64(), Some(42));
    }

    #[test]
    fn test_json_array_of_booleans() {
        let arr = json!([true, false, true]);
        assert_eq!(arr.len(), Some(3));
        assert_eq!(arr[0].as_bool(), Some(true));
        assert_eq!(arr[1].as_bool(), Some(false));
        assert_eq!(arr[2].as_bool(), Some(true));
    }

    #[test]
    fn test_json_array_of_strings() {
        let arr = json!(["a", "b", "c"]);
        assert_eq!(arr[0].as_str(), Some("a"));
        assert_eq!(arr[1].as_str(), Some("b"));
        assert_eq!(arr[2].as_str(), Some("c"));
    }

    #[test]
    fn test_json_array_of_nulls() {
        let arr = json!([json!(null), json!(null)]);
        assert_eq!(arr.len(), Some(2));
        assert!(arr[0].is_null());
        assert!(arr[1].is_null());
    }

    #[test]
    fn test_json_array_trailing_comma() {
        // Trailing comma should compile and produce same result
        let arr = json!([1, 2, 3,]);
        assert_eq!(arr.len(), Some(3));
    }

    #[test]
    fn test_json_nested_array() {
        let arr = json!([[1, 2], [3, 4]]);
        assert!(arr[0].is_array());
        assert_eq!(arr[0][0].as_i64(), Some(1));
        assert_eq!(arr[1][1].as_i64(), Some(4));
    }

    #[test]
    fn test_json_array_with_null_element() {
        let arr = json!([1, json!(null), 3]);
        assert_eq!(arr[1].is_null(), true);
    }

    // objects — structure
    #[test]
    fn test_json_object_single_key() {
        let obj = json!({"x": 1});
        assert_eq!(obj["x"].as_i64(), Some(1));
    }

    #[test]
    fn test_json_object_bool_values() {
        let obj = json!({"a": true, "b": false});
        assert_eq!(obj["a"].as_bool(), Some(true));
        assert_eq!(obj["b"].as_bool(), Some(false));
    }

    #[test]
    fn test_json_object_null_value() {
        let obj = json!({"nothing": null});
        assert!(obj["nothing"].is_null());
    }

    #[test]
    fn test_json_object_string_value() {
        let obj = json!({"greeting": "hello"});
        assert_eq!(obj["greeting"].as_str(), Some("hello"));
    }

    #[test]
    fn test_json_object_float_value() {
        let obj = json!({"pi": 3.14});
        let v = obj["pi"].as_f64().unwrap();
        assert!((v - 3.14).abs() < 1e-9);
    }

    #[test]
    fn test_json_object_trailing_comma() {
        let obj = json!({"k": "v",});
        assert_eq!(obj["k"].as_str(), Some("v"));
    }

    #[test]
    fn test_json_object_array_value() {
        let obj = json!({"items": [1, 2, 3]});
        assert!(obj["items"].is_array());
        assert_eq!(obj["items"].len(), Some(3));
    }

    #[test]
    fn test_json_object_nested_object() {
        let obj = json!({"inner": {"val": 99}});
        assert_eq!(obj["inner"]["val"].as_i64(), Some(99));
    }

    // variable interpolation
    #[test]
    fn test_json_variable_string() {
        let s = "interpolated";
        let obj = json!({"key": s});
        assert_eq!(obj["key"].as_str(), Some("interpolated"));
    }

    #[test]
    fn test_json_variable_integer() {
        let n: i32 = 77;
        let obj = json!({"n": n});
        assert_eq!(obj["n"].as_i64(), Some(77));
    }

    #[test]
    fn test_json_variable_bool() {
        let flag = false;
        let obj = json!({"flag": flag});
        assert_eq!(obj["flag"].as_bool(), Some(false));
    }

    #[test]
    fn test_json_variable_in_array() {
        let x = 10;
        let arr = json!([x, x, x]);
        assert_eq!(arr[0].as_i64(), Some(10));
        assert_eq!(arr[2].as_i64(), Some(10));
    }

    // deep nesting
    #[test]
    fn test_json_three_levels_deep() {
        let data = json!({"a": {"b": {"c": 42}}});
        assert_eq!(data["a"]["b"]["c"].as_i64(), Some(42));
    }

    #[test]
    fn test_json_array_in_object_in_array() {
        let arr = json!([json!({"vals": [true, false]})]);
        assert_eq!(arr[0]["vals"][0].as_bool(), Some(true));
        assert_eq!(arr[0]["vals"][1].as_bool(), Some(false));
    }

    // macro produces correct Node variants
    #[test]
    fn test_json_produces_array_variant() {
        assert!(matches!(json!([]), Node::Array(_)));
    }

    #[test]
    fn test_json_produces_object_variant() {
        assert!(matches!(json!({}), Node::Object(_)));
    }

    #[test]
    fn test_json_produces_str_variant() {
        assert!(matches!(json!("x"), Node::Str(_)));
    }

    #[test]
    fn test_json_produces_boolean_variants() {
        assert!(matches!(json!(true), Node::Boolean(true)));
        assert!(matches!(json!(false), Node::Boolean(false)));
    }

    #[test]
    fn test_json_produces_none_variant() {
        assert!(matches!(json!(null), Node::None));
    }
}
