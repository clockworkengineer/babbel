//! Declarative macro `value!` for ergonomic `Value` construction.

#[doc(hidden)]
pub use alloc::string::String as __String;
#[doc(hidden)]
pub use alloc::string::ToString as __ToString;
#[doc(hidden)]
pub use alloc::vec::Vec as __Vec;

/// Helper function for macro - creates new Vec for Value arrays
#[doc(hidden)]
#[inline]
pub fn __value_vec_new() -> alloc::vec::Vec<babbel_core::Value> {
    alloc::vec::Vec::new()
}

/// Helper function for macro - creates new Vec for Value objects
#[doc(hidden)]
#[inline]
pub fn __value_map_new() -> alloc::vec::Vec<(alloc::string::String, babbel_core::Value)> {
    alloc::vec::Vec::new()
}

/// Constructs a [`babbel_core::Value`] using literal syntax.
///
/// Supports null, booleans, numbers, strings, arrays, and objects.
///
/// # Examples
///
/// ```
/// use babbel::value;
///
/// let v = value!({
///     "name": "Babbel",
///     "port": 8080,
///     "active": true,
///     "tags": ["serialization", "polyglot"],
///     "meta": null
/// });
///
/// assert_eq!(v["name"].as_str(), Some("Babbel"));
/// assert_eq!(v["port"].as_i64(), Some(8080));
/// assert_eq!(v["active"].as_bool(), Some(true));
/// assert_eq!(v["tags"][0].as_str(), Some("serialization"));
/// assert!(v["meta"].is_null());
/// ```
#[macro_export]
macro_rules! value {
    (null) => {
        $crate::Value::Null
    };

    (true) => {
        $crate::Value::Bool(true)
    };

    (false) => {
        $crate::Value::Bool(false)
    };

    ([]) => {
        $crate::Value::Array($crate::macros::__value_vec_new())
    };

    ([ $($tt:tt)+ ]) => {
        $crate::Value::Array($crate::__value_vec![$($tt)+])
    };

    ({}) => {
        $crate::Value::Object($crate::macros::__value_map_new())
    };

    ({ $($tt:tt)+ }) => {
        $crate::Value::Object($crate::__value_map!($($tt)+))
    };

    ($other:expr) => {
        $crate::Value::from($other)
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __value_vec {
    () => {
        $crate::macros::__value_vec_new()
    };

    ($($content:tt)+) => {{
        let mut vec = $crate::macros::__value_vec_new();
        $crate::__value_vec_push!(&mut vec, $($content)+);
        vec
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! __value_vec_push {
    ($vec:expr, $elem:expr) => {
        $vec.push($crate::value!($elem))
    };

    ($vec:expr, $elem:expr,) => {
        $crate::__value_vec_push!($vec, $elem)
    };

    ($vec:expr, $elem:expr, $($rest:tt)*) => {
        $crate::__value_vec_push!($vec, $elem);
        $crate::__value_vec_push!($vec, $($rest)*);
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __value_map {
    () => {
        $crate::macros::__value_map_new()
    };

    ($($content:tt)+) => {{
        let mut entries = $crate::macros::__value_map_new();
        $crate::__value_map_insert!(&mut entries, $($content)+);
        entries
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! __value_map_insert {
    ($entries:expr, $key:tt : $value:tt) => {
        $entries.push(($crate::macros::__ToString::to_string(&$key), $crate::value!($value)));
    };

    ($entries:expr, $key:tt : $value:tt,) => {
        $crate::__value_map_insert!($entries, $key : $value)
    };

    ($entries:expr, $key:tt : $value:tt, $($rest:tt)*) => {
        $crate::__value_map_insert!($entries, $key : $value);
        $crate::__value_map_insert!($entries, $($rest)*);
    };
}

#[cfg(test)]
mod tests {
    use crate::Value;

    #[test]
    fn test_value_macro_primitives() {
        assert_eq!(value!(null), Value::Null);
        assert_eq!(value!(true), Value::Bool(true));
        assert_eq!(value!(false), Value::Bool(false));
        assert_eq!(value!(42), Value::Integer(42));
        assert_eq!(value!("hello"), Value::String("hello".into()));
    }

    #[test]
    fn test_value_macro_arrays_and_objects() {
        let val = value!({
            "name": "Babbel",
            "active": true,
            "count": 3,
            "items": ["a", "b", "c"],
            "nested": {
                "ok": true
            }
        });

        assert_eq!(val["name"].as_str(), Some("Babbel"));
        assert_eq!(val["active"].as_bool(), Some(true));
        assert_eq!(val["count"].as_i64(), Some(3));
        assert_eq!(val["items"][0].as_str(), Some("a"));
        assert_eq!(val["items"][1].as_str(), Some("b"));
        assert_eq!(val["items"][2].as_str(), Some("c"));
        assert_eq!(val["nested"]["ok"].as_bool(), Some(true));
        assert_eq!(val["nonexistent"], Value::Null);
    }
}
