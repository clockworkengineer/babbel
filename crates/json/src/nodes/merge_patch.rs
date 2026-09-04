//! JSON Merge Patch (RFC 7386) implementation
//!
//! Simpler alternative to JSON Patch for partial updates.
//! Uses JSON structure itself to describe the changes.

use crate::nodes::node::Node;

#[cfg(feature = "std")]
use std::collections::HashMap;

#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as HashMap;

/// Apply a merge patch to a target document
///
/// # Arguments
/// * `target` - The document to be patched (modified in place)
/// * `patch` - The merge patch to apply
///
/// # Examples
/// ```
/// use json_lib::{json, nodes::merge_patch::merge_patch};
///
/// let mut doc = json!({"a": 1, "b": 2});
/// let patch = json!({"b": 3, "c": 4});
/// merge_patch(&mut doc, &patch);
/// // doc is now {"a": 1, "b": 3, "c": 4}
/// ```
pub fn merge_patch(target: &mut Node, patch: &Node) {
    if !patch.is_object() {
        // If patch is not an object, replace target entirely
        *target = patch.clone();
        return;
    }

    // Ensure target is an object
    if !target.is_object() {
        *target = Node::Object(HashMap::new());
    }

    let target_obj = target.as_object_mut().unwrap();
    let patch_obj = patch.as_object().unwrap();

    for (key, patch_value) in patch_obj.iter() {
        if patch_value.is_null() {
            // Remove key if patch value is null
            target_obj.remove(key);
        } else if patch_value.is_object() {
            // Recursively merge if both are objects
            if let Some(target_value) = target_obj.get_mut(key) {
                if target_value.is_object() {
                    merge_patch(target_value, patch_value);
                } else {
                    *target_value = patch_value.clone();
                }
            } else {
                target_obj.insert(key.clone(), patch_value.clone());
            }
        } else {
            // Replace/add value
            target_obj.insert(key.clone(), patch_value.clone());
        }
    }
}

/// Create a merge patch that transforms source into target
///
/// # Arguments
/// * `source` - The original document
/// * `target` - The desired document
///
/// # Returns
/// A merge patch that, when applied to source, produces target
///
/// # Examples
/// ```
/// use json_lib::{json, nodes::merge_patch::create_merge_patch};
///
/// let source = json!({"a": 1, "b": 2});
/// let target = json!({"a": 1, "c": 3});
/// let patch = create_merge_patch(&source, &target);
/// // patch is {"b": null, "c": 3}
/// ```
pub fn create_merge_patch(source: &Node, target: &Node) -> Node {
    // If target is not an object, the patch is just the target
    if !target.is_object() {
        return target.clone();
    }

    // If source is not an object, the patch is the entire target
    if !source.is_object() {
        return target.clone();
    }

    let source_obj = source.as_object().unwrap();
    let target_obj = target.as_object().unwrap();
    let mut patch = HashMap::new();

    // Add/modify keys
    for (key, target_value) in target_obj.iter() {
        if let Some(source_value) = source_obj.get(key) {
            if source_value != target_value {
                if source_value.is_object() && target_value.is_object() {
                    // Recursively create patch for nested objects
                    let nested_patch = create_merge_patch(source_value, target_value);
                    if !nested_patch.as_object().unwrap().is_empty() {
                        patch.insert(key.clone(), nested_patch);
                    }
                } else {
                    patch.insert(key.clone(), target_value.clone());
                }
            }
        } else {
            // Key exists in target but not source - add it
            patch.insert(key.clone(), target_value.clone());
        }
    }

    // Remove keys that exist in source but not in target
    for key in source_obj.keys() {
        if !target_obj.contains_key(key) {
            patch.insert(key.clone(), Node::None);
        }
    }

    Node::Object(patch)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json;

    #[test]
    fn test_merge_patch_simple() {
        let mut doc = json!({"a": 1, "b": 2});
        let patch = json!({"b": 3, "c": 4});
        merge_patch(&mut doc, &patch);

        assert_eq!(doc["a"], json!(1));
        assert_eq!(doc["b"], json!(3));
        assert_eq!(doc["c"], json!(4));
    }

    #[test]
    fn test_merge_patch_remove() {
        let mut doc = json!({"a": 1, "b": 2});
        let patch = json!({"b": null});
        merge_patch(&mut doc, &patch);

        assert_eq!(doc["a"], json!(1));
        assert!(doc["b"].is_null());
    }

    #[test]
    fn test_merge_patch_nested() {
        let mut doc = json!({
            "user": {
                "name": "Alice",
                "age": 30
            }
        });
        let patch = json!({
            "user": {
                "age": 31,
                "email": "alice@example.com"
            }
        });
        merge_patch(&mut doc, &patch);

        assert_eq!(doc["user"]["name"], json!("Alice"));
        assert_eq!(doc["user"]["age"], json!(31));
        assert_eq!(doc["user"]["email"], json!("alice@example.com"));
    }

    #[test]
    fn test_merge_patch_replace() {
        let mut doc = json!({"a": [1, 2, 3]});
        let patch = json!({"a": [4, 5]});
        merge_patch(&mut doc, &patch);

        assert_eq!(doc["a"], json!([4, 5]));
    }

    #[test]
    fn test_create_merge_patch() {
        let source = json!({"a": 1, "b": 2, "c": 3});
        let target = json!({"a": 1, "b": 4, "d": 5});
        let patch = create_merge_patch(&source, &target);

        // Patch should update b, add d, and remove c
        assert_eq!(patch["b"], json!(4));
        assert_eq!(patch["d"], json!(5));
        assert!(patch["c"].is_null());
        assert!(patch.get("a").is_none() || patch["a"].is_null());
    }

    #[test]
    fn test_create_merge_patch_nested() {
        let source = json!({
            "user": {
                "name": "Alice",
                "age": 30
            }
        });
        let target = json!({
            "user": {
                "name": "Alice",
                "age": 31
            }
        });
        let patch = create_merge_patch(&source, &target);

        assert_eq!(patch["user"]["age"], json!(31));
    }

    #[test]
    fn test_merge_patch_idempotent() {
        let mut doc = json!({"a": 1});
        let patch = json!({"b": 2});

        merge_patch(&mut doc, &patch);
        let first_result = doc.clone();

        merge_patch(&mut doc, &patch);
        assert_eq!(doc, first_result);
    }

    // ─── merge_patch: edge cases ──────────────────────────────────────────────

    #[test]
    fn test_merge_patch_with_empty_patch() {
        let mut doc = json!({"a": 1, "b": 2});
        let original = doc.clone();
        merge_patch(&mut doc, &json!({}));
        assert_eq!(doc, original);
    }

    #[test]
    fn test_merge_patch_onto_empty_target() {
        let mut doc = json!({});
        merge_patch(&mut doc, &json!({"x": 10, "y": 20}));
        assert_eq!(doc["x"], json!(10));
        assert_eq!(doc["y"], json!(20));
    }

    #[test]
    fn test_merge_patch_non_object_patch_replaces_target() {
        let mut doc = json!({"a": 1});
        merge_patch(&mut doc, &json!(42));
        assert_eq!(doc, json!(42));
    }

    #[test]
    fn test_merge_patch_non_object_patch_array_replaces_target() {
        let mut doc = json!({"a": 1});
        merge_patch(&mut doc, &json!([1, 2, 3]));
        assert_eq!(doc, json!([1, 2, 3]));
    }

    #[test]
    fn test_merge_patch_non_object_patch_bool_replaces_target() {
        let mut doc = json!({"a": 1});
        merge_patch(&mut doc, &json!(true));
        assert_eq!(doc, json!(true));
    }

    #[test]
    fn test_merge_patch_non_object_patch_null_replaces_target() {
        let mut doc = json!({"a": 1});
        merge_patch(&mut doc, &json!(null));
        assert!(doc.is_null());
    }

    #[test]
    fn test_merge_patch_target_non_object_becomes_object() {
        // target is a scalar; patch is object → target becomes object
        let mut doc = json!(42);
        merge_patch(&mut doc, &json!({"key": "value"}));
        assert!(doc.is_object());
        assert_eq!(doc["key"], json!("value"));
    }

    #[test]
    fn test_merge_patch_removes_multiple_keys_with_null() {
        let mut doc = json!({"a": 1, "b": 2, "c": 3});
        merge_patch(&mut doc, &json!({"a": null, "c": null}));
        assert!(doc.get("a").is_none() || doc["a"].is_null());
        assert!(doc.get("c").is_none() || doc["c"].is_null());
        assert_eq!(doc["b"], json!(2));
    }

    #[test]
    fn test_merge_patch_adds_new_nested_object() {
        let mut doc = json!({"a": 1});
        merge_patch(&mut doc, &json!({"nested": {"x": 10}}));
        assert_eq!(doc["nested"]["x"], json!(10));
    }

    #[test]
    fn test_merge_patch_replaces_nested_non_object_with_value() {
        let mut doc = json!({"a": {"b": 1}});
        // Patch replaces the nested object's b with a string
        merge_patch(&mut doc, &json!({"a": {"b": "hello"}}));
        assert_eq!(doc["a"]["b"], json!("hello"));
    }

    #[test]
    fn test_merge_patch_overwrites_nested_non_object_with_object() {
        // "a" is a scalar in doc; patch sets it to an object
        let mut doc = json!({"a": 5});
        merge_patch(&mut doc, &json!({"a": {"x": 1}}));
        assert_eq!(doc["a"]["x"], json!(1));
    }

    #[test]
    fn test_merge_patch_string_value() {
        let mut doc = json!({"greeting": "hello"});
        merge_patch(&mut doc, &json!({"greeting": "world"}));
        assert_eq!(doc["greeting"], json!("world"));
    }

    #[test]
    fn test_merge_patch_boolean_value() {
        let mut doc = json!({"active": false});
        merge_patch(&mut doc, &json!({"active": true}));
        assert_eq!(doc["active"], json!(true));
    }

    #[test]
    fn test_merge_patch_array_value_replaced_not_merged() {
        // RFC 7396: arrays are treated as atomic values – replaced, not merged
        let mut doc = json!({"list": [1, 2, 3]});
        merge_patch(&mut doc, &json!({"list": [4, 5]}));
        assert_eq!(doc["list"], json!([4, 5]));
    }

    #[test]
    fn test_merge_patch_deep_nested() {
        let mut doc = json!({"a": {"b": {"c": {"d": 1}}}});
        merge_patch(&mut doc, &json!({"a": {"b": {"c": {"d": 99}}}}));
        assert_eq!(doc["a"]["b"]["c"]["d"], json!(99));
    }

    // ─── create_merge_patch: edge cases ───────────────────────────────────────

    #[test]
    fn test_create_merge_patch_identical_docs_produces_empty_patch() {
        let source = json!({"a": 1, "b": "hello"});
        let target = json!({"a": 1, "b": "hello"});
        let patch = create_merge_patch(&source, &target);
        assert!(patch.is_object());
        assert!(patch.as_object().unwrap().is_empty());
    }

    #[test]
    fn test_create_merge_patch_target_adds_key() {
        let source = json!({"a": 1});
        let target = json!({"a": 1, "b": 2});
        let patch = create_merge_patch(&source, &target);
        assert_eq!(patch["b"], json!(2));
        assert!(patch.get("a").is_none());
    }

    #[test]
    fn test_create_merge_patch_source_has_extra_key() {
        let source = json!({"a": 1, "b": 2});
        let target = json!({"a": 1});
        let patch = create_merge_patch(&source, &target);
        // "b" should be nulled out
        assert!(patch["b"].is_null());
    }

    #[test]
    fn test_create_merge_patch_non_object_target_returns_target() {
        let source = json!({"a": 1});
        let target = json!(42);
        let patch = create_merge_patch(&source, &target);
        assert_eq!(patch, json!(42));
    }

    #[test]
    fn test_create_merge_patch_non_object_source_returns_target() {
        let source = json!(42);
        let target = json!({"a": 1});
        let patch = create_merge_patch(&source, &target);
        assert_eq!(patch, json!({"a": 1}));
    }

    #[test]
    fn test_create_merge_patch_value_type_change() {
        let source = json!({"a": 1});
        let target = json!({"a": "one"});
        let patch = create_merge_patch(&source, &target);
        assert_eq!(patch["a"], json!("one"));
    }

    #[test]
    fn test_create_merge_patch_nested_no_change_excluded() {
        let source = json!({"nested": {"x": 1, "y": 2}});
        let target = json!({"nested": {"x": 1, "y": 2}});
        let patch = create_merge_patch(&source, &target);
        // No changes – patch should be empty
        assert!(patch.as_object().unwrap().is_empty());
    }

    #[test]
    fn test_create_then_apply_roundtrip() {
        let source = json!({"a": 1, "b": 2, "c": {"d": 3}});
        let target = json!({"a": 1, "b": 99, "e": 5});

        let patch = create_merge_patch(&source, &target);
        let mut result = source.clone();
        merge_patch(&mut result, &patch);

        // After applying the patch, result["a"] and result["b"] and result["e"]
        // should match target; result["c"] should be gone (nulled)
        assert_eq!(result["a"], json!(1));
        assert_eq!(result["b"], json!(99));
        assert_eq!(result["e"], json!(5));
        // "c" was in source but not target – patch nulls it
        assert!(result.get("c").is_none() || result["c"].is_null());
    }
}
