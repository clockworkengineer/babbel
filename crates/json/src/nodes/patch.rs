//! JSON Patch (RFC 6902) implementation
//!
//! Provides operations to modify JSON documents: add, remove, replace, move, copy, test

use crate::nodes::node::Node;

#[cfg(feature = "json-pointer")]
use crate::nodes::json_pointer;

#[cfg(not(feature = "std"))]
use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use arrayvec::ArrayString;

/// JSON Patch operation
#[derive(Debug, Clone, PartialEq)]
pub enum PatchOp {
    Add { path: String, value: Node },
    Remove { path: String },
    Replace { path: String, value: Node },
    Move { from: String, path: String },
    Copy { from: String, path: String },
    Test { path: String, value: Node },
}

/// Error type for patch operations
#[derive(Debug, Clone, PartialEq)]
pub struct PatchError {
    pub message: String,
}

impl PatchError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
        }
    }
}

impl From<String> for PatchError {
    fn from(s: String) -> Self {
        PatchError::new(s)
    }
}

/// Apply a single patch operation to a node
#[cfg(feature = "json-pointer")]
pub fn apply_operation(node: &mut Node, op: &PatchOp) -> Result<(), PatchError> {
    match op {
        PatchOp::Add { path, value } => add_value(node, path, value.clone()),
        PatchOp::Remove { path } => remove_value(node, path),
        PatchOp::Replace { path, value } => replace_value(node, path, value.clone()),
        PatchOp::Move { from, path } => move_value(node, from, path),
        PatchOp::Copy { from, path } => copy_value(node, from, path),
        PatchOp::Test { path, value } => test_value(node, path, value),
    }
}

/// Apply multiple patch operations in sequence
#[cfg(feature = "json-pointer")]
pub fn apply_patch(node: &mut Node, operations: &[PatchOp]) -> Result<(), PatchError> {
    for op in operations {
        apply_operation(node, op)?;
    }
    Ok(())
}

#[cfg(feature = "json-pointer")]
fn add_value(node: &mut Node, path: &str, value: Node) -> Result<(), PatchError> {
    if path == "" {
        *node = value;
        return Ok(());
    }

    // Split path to get parent and key
    let parts: Vec<&str> = path.rsplitn(2, '/').collect();
    if parts.len() != 2 {
        return Err(PatchError::new("Invalid path"));
    }

    let key = parts[0];
    let parent_path = parts[1];

    // Get parent node
    let parent = if parent_path.is_empty() {
        node
    } else {
        json_pointer::get_mut(node, parent_path)
            .ok_or_else(|| PatchError::new("Parent path not found"))?
    };

    // Add to parent
    if let Some(obj) = parent.as_object_mut() {
        obj.insert(key.to_string(), value);
        Ok(())
    } else if let Some(arr) = parent.as_array_mut() {
        if key == "-" {
            arr.push(value);
            Ok(())
        } else if let Ok(idx) = key.parse::<usize>() {
            if idx <= arr.len() {
                arr.insert(idx, value);
                Ok(())
            } else {
                Err(PatchError::new("Array index out of bounds"))
            }
        } else {
            Err(PatchError::new("Invalid array index"))
        }
    } else {
        Err(PatchError::new("Parent is not an object or array"))
    }
}

#[cfg(feature = "json-pointer")]
fn remove_value(node: &mut Node, path: &str) -> Result<(), PatchError> {
    if path == "" {
        return Err(PatchError::new("Cannot remove root"));
    }

    match json_pointer::remove(node, path) {
        Ok(Some(_)) => Ok(()),
        Ok(None) => Err(PatchError::new("Path not found")),
        Err(e) => {
            let mut msg = ArrayString::<64>::new();
            msg.push_str("Invalid path: ");
            msg.push_str(&e);
            Err(PatchError::new(msg.as_str().to_owned()))
        }
    }
}

#[cfg(feature = "json-pointer")]
fn replace_value(node: &mut Node, path: &str, value: Node) -> Result<(), PatchError> {
    if path == "" {
        *node = value;
        return Ok(());
    }

    let target =
        json_pointer::get_mut(node, path).ok_or_else(|| PatchError::new("Path not found"))?;
    *target = value;
    Ok(())
}

#[cfg(feature = "json-pointer")]
fn move_value(node: &mut Node, from: &str, to: &str) -> Result<(), PatchError> {
    // Get the value to move
    let value = json_pointer::get(node, from)
        .ok_or_else(|| PatchError::new("Source path not found"))?
        .clone();

    // Remove from source
    match json_pointer::remove(node, from) {
        Ok(Some(_)) => {}
        Ok(None) => return Err(PatchError::new("Failed to remove from source")),
        Err(e) => {
            let mut msg = ArrayString::<64>::new();
            msg.push_str("Invalid path: ");
            msg.push_str(&e);
            return Err(PatchError::new(msg.as_str().to_owned()));
        }
    }

    // Add to destination
    add_value(node, to, value)
}

#[cfg(feature = "json-pointer")]
fn copy_value(node: &mut Node, from: &str, to: &str) -> Result<(), PatchError> {
    let value = json_pointer::get(node, from)
        .ok_or_else(|| PatchError::new("Source path not found"))?
        .clone();

    add_value(node, to, value)
}

#[cfg(feature = "json-pointer")]
fn test_value(node: &Node, path: &str, expected: &Node) -> Result<(), PatchError> {
    let actual = json_pointer::get(node, path).ok_or_else(|| PatchError::new("Path not found"))?;

    if actual == expected {
        Ok(())
    } else {
        // Use stack-allocated buffer for error message
        let mut msg = ArrayString::<96>::new();
        use core::fmt::Write;
        let _ = write!(
            &mut msg,
            "Test failed: expected {:?}, got {:?}",
            expected, actual
        );
        Err(PatchError::new(msg.as_str().to_owned()))
    }
}

/// Parse a JSON Patch document (array of operations)
#[cfg(feature = "json-pointer")]
pub fn parse_patch(patch_doc: &Node) -> Result<Vec<PatchOp>, PatchError> {
    let arr = patch_doc
        .as_array()
        .ok_or_else(|| PatchError::new("Patch document must be an array"))?;

    let mut operations = Vec::new();

    for item in arr {
        let obj = item
            .as_object()
            .ok_or_else(|| PatchError::new("Patch operation must be an object"))?;

        let op_type = obj
            .get("op")
            .and_then(|n| n.as_str())
            .ok_or_else(|| PatchError::new("Missing 'op' field"))?;

        let op = match op_type {
            "add" => {
                let path = obj
                    .get("path")
                    .and_then(|n| n.as_str())
                    .ok_or_else(|| PatchError::new("Missing 'path' field"))?;
                let value = obj
                    .get("value")
                    .ok_or_else(|| PatchError::new("Missing 'value' field"))?;
                PatchOp::Add {
                    path: path.to_string(),
                    value: value.clone(),
                }
            }
            "remove" => {
                let path = obj
                    .get("path")
                    .and_then(|n| n.as_str())
                    .ok_or_else(|| PatchError::new("Missing 'path' field"))?;
                PatchOp::Remove {
                    path: path.to_string(),
                }
            }
            "replace" => {
                let path = obj
                    .get("path")
                    .and_then(|n| n.as_str())
                    .ok_or_else(|| PatchError::new("Missing 'path' field"))?;
                let value = obj
                    .get("value")
                    .ok_or_else(|| PatchError::new("Missing 'value' field"))?;
                PatchOp::Replace {
                    path: path.to_string(),
                    value: value.clone(),
                }
            }
            "move" => {
                let from = obj
                    .get("from")
                    .and_then(|n| n.as_str())
                    .ok_or_else(|| PatchError::new("Missing 'from' field"))?;
                let path = obj
                    .get("path")
                    .and_then(|n| n.as_str())
                    .ok_or_else(|| PatchError::new("Missing 'path' field"))?;
                PatchOp::Move {
                    from: from.to_string(),
                    path: path.to_string(),
                }
            }
            "copy" => {
                let from = obj
                    .get("from")
                    .and_then(|n| n.as_str())
                    .ok_or_else(|| PatchError::new("Missing 'from' field"))?;
                let path = obj
                    .get("path")
                    .and_then(|n| n.as_str())
                    .ok_or_else(|| PatchError::new("Missing 'path' field"))?;
                PatchOp::Copy {
                    from: from.to_string(),
                    path: path.to_string(),
                }
            }
            "test" => {
                let path = obj
                    .get("path")
                    .and_then(|n| n.as_str())
                    .ok_or_else(|| PatchError::new("Missing 'path' field"))?;
                let value = obj
                    .get("value")
                    .ok_or_else(|| PatchError::new("Missing 'value' field"))?;
                PatchOp::Test {
                    path: path.to_string(),
                    value: value.clone(),
                }
            }
            _ => {
                let mut msg = ArrayString::<48>::new();
                msg.push_str("Unknown operation: ");
                msg.push_str(op_type);
                return Err(PatchError::new(msg.as_str().to_owned()));
            }
        };

        operations.push(op);
    }

    Ok(operations)
}

#[cfg(test)]
#[cfg(feature = "json-pointer")]
mod tests {
    use super::*;
    use crate::json;

    #[test]
    fn test_add_operation() {
        let mut doc = json!({"a": 1});
        let op = PatchOp::Add {
            path: "/b".to_string(),
            value: json!(2),
        };

        assert!(apply_operation(&mut doc, &op).is_ok());
        assert_eq!(doc["b"], json!(2));
    }

    #[test]
    fn test_remove_operation() {
        let mut doc = json!({"a": 1, "b": 2});
        let op = PatchOp::Remove {
            path: "/b".to_string(),
        };

        assert!(apply_operation(&mut doc, &op).is_ok());
        assert!(doc["b"].is_null());
    }

    #[test]
    fn test_replace_operation() {
        let mut doc = json!({"a": 1});
        let op = PatchOp::Replace {
            path: "/a".to_string(),
            value: json!(2),
        };

        assert!(apply_operation(&mut doc, &op).is_ok());
        assert_eq!(doc["a"], json!(2));
    }

    #[test]
    fn test_move_operation() {
        let mut doc = json!({"a": 1, "b": 2});
        let op = PatchOp::Move {
            from: "/a".to_string(),
            path: "/c".to_string(),
        };

        assert!(apply_operation(&mut doc, &op).is_ok());
        assert!(doc["a"].is_null());
        assert_eq!(doc["c"], json!(1));
    }

    #[test]
    fn test_copy_operation() {
        let mut doc = json!({"a": 1});
        let op = PatchOp::Copy {
            from: "/a".to_string(),
            path: "/b".to_string(),
        };

        assert!(apply_operation(&mut doc, &op).is_ok());
        assert_eq!(doc["a"], json!(1));
        assert_eq!(doc["b"], json!(1));
    }

    #[test]
    fn test_test_operation() {
        let doc = json!({"a": 1});
        let op = PatchOp::Test {
            path: "/a".to_string(),
            value: json!(1),
        };

        assert!(apply_operation(&mut doc.clone(), &op).is_ok());

        let op_fail = PatchOp::Test {
            path: "/a".to_string(),
            value: json!(2),
        };
        assert!(apply_operation(&mut doc.clone(), &op_fail).is_err());
    }

    #[test]
    fn test_array_operations() {
        let mut doc = json!([1, 2, 3]);
        let op = PatchOp::Add {
            path: "/-".to_string(),
            value: json!(4),
        };

        assert!(apply_operation(&mut doc, &op).is_ok());
        assert_eq!(doc[3], json!(4));
    }

    // --- PatchError ---

    #[test]
    fn test_patch_error_new() {
        let e = PatchError::new("something went wrong");
        assert_eq!(e.message, "something went wrong");
    }

    #[test]
    fn test_patch_error_from_string() {
        let e = PatchError::from("oops".to_string());
        assert_eq!(e.message, "oops");
    }

    #[test]
    fn test_patch_error_clone_and_eq() {
        let e = PatchError::new("err");
        let e2 = e.clone();
        assert_eq!(e, e2);
        assert_ne!(e, PatchError::new("different"));
    }

    // --- PatchOp clone / eq ---

    #[test]
    fn test_patch_op_clone_and_eq() {
        let op = PatchOp::Add {
            path: "/x".to_string(),
            value: json!(1),
        };
        assert_eq!(op.clone(), op);

        let remove = PatchOp::Remove {
            path: "/x".to_string(),
        };
        assert_ne!(op, remove);
    }

    // --- add_value edge cases ---

    #[test]
    fn test_add_replaces_root() {
        let mut doc = json!({"a": 1});
        let op = PatchOp::Add {
            path: "".to_string(),
            value: json!({"replaced": true}),
        };
        assert!(apply_operation(&mut doc, &op).is_ok());
        assert_eq!(doc["replaced"], json!(true));
    }

    #[test]
    fn test_add_to_nested_object() {
        let mut doc = json!({"user": {"name": "Alice"}});
        let op = PatchOp::Add {
            path: "/user/age".to_string(),
            value: json!(30),
        };
        assert!(apply_operation(&mut doc, &op).is_ok());
        assert_eq!(doc["user"]["age"], json!(30));
    }

    #[test]
    fn test_add_array_at_index() {
        let mut doc = json!({"arr": [1, 2, 3]});
        let op = PatchOp::Add {
            path: "/arr/1".to_string(),
            value: json!(99),
        };
        assert!(apply_operation(&mut doc, &op).is_ok());
        assert_eq!(doc["arr"][0], json!(1));
        assert_eq!(doc["arr"][1], json!(99));
        assert_eq!(doc["arr"][2], json!(2));
    }

    #[test]
    fn test_add_array_at_end_with_dash() {
        let mut doc = json!({"arr": [10, 20]});
        let op = PatchOp::Add {
            path: "/arr/-".to_string(),
            value: json!(30),
        };
        assert!(apply_operation(&mut doc, &op).is_ok());
        assert_eq!(doc["arr"][2], json!(30));
    }

    #[test]
    fn test_add_array_out_of_bounds_fails() {
        let mut doc = json!({"arr": [1, 2]});
        let op = PatchOp::Add {
            path: "/arr/99".to_string(),
            value: json!(0),
        };
        assert!(apply_operation(&mut doc, &op).is_err());
    }

    #[test]
    fn test_add_invalid_array_index_fails() {
        let mut doc = json!({"arr": [1, 2]});
        let op = PatchOp::Add {
            path: "/arr/notanumber".to_string(),
            value: json!(0),
        };
        assert!(apply_operation(&mut doc, &op).is_err());
    }

    #[test]
    fn test_add_parent_path_not_found_fails() {
        let mut doc = json!({"a": 1});
        let op = PatchOp::Add {
            path: "/missing/key".to_string(),
            value: json!(0),
        };
        assert!(apply_operation(&mut doc, &op).is_err());
    }

    #[test]
    fn test_add_to_non_object_non_array_parent_fails() {
        let mut doc = json!({"num": 42});
        // Parent is a number, not object or array
        let op = PatchOp::Add {
            path: "/num/sub".to_string(),
            value: json!(1),
        };
        // This should fail because we can't navigate into a number
        assert!(apply_operation(&mut doc, &op).is_err());
    }

    // --- remove_value edge cases ---

    #[test]
    fn test_remove_root_fails() {
        let mut doc = json!({"a": 1});
        let op = PatchOp::Remove {
            path: "".to_string(),
        };
        let result = apply_operation(&mut doc, &op);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("root"));
    }

    #[test]
    fn test_remove_missing_path_fails() {
        let mut doc = json!({"a": 1});
        let op = PatchOp::Remove {
            path: "/missing".to_string(),
        };
        assert!(apply_operation(&mut doc, &op).is_err());
    }

    #[test]
    fn test_remove_nested_key() {
        let mut doc = json!({"user": {"name": "Alice", "age": 30}});
        let op = PatchOp::Remove {
            path: "/user/age".to_string(),
        };
        assert!(apply_operation(&mut doc, &op).is_ok());
        assert!(doc["user"]["age"].is_null());
        assert_eq!(doc["user"]["name"].as_str(), Some("Alice"));
    }

    #[test]
    fn test_remove_array_element() {
        let mut doc = json!({"arr": [1, 2, 3]});
        let op = PatchOp::Remove {
            path: "/arr/1".to_string(),
        };
        assert!(apply_operation(&mut doc, &op).is_ok());
        assert_eq!(doc["arr"][0], json!(1));
        assert_eq!(doc["arr"][1], json!(3));
    }

    // --- replace_value edge cases ---

    #[test]
    fn test_replace_root() {
        let mut doc = json!({"a": 1});
        let op = PatchOp::Replace {
            path: "".to_string(),
            value: json!({"b": 2}),
        };
        assert!(apply_operation(&mut doc, &op).is_ok());
        assert_eq!(doc["b"], json!(2));
    }

    #[test]
    fn test_replace_missing_path_fails() {
        let mut doc = json!({"a": 1});
        let op = PatchOp::Replace {
            path: "/missing".to_string(),
            value: json!(0),
        };
        assert!(apply_operation(&mut doc, &op).is_err());
    }

    #[test]
    fn test_replace_nested() {
        let mut doc = json!({"user": {"name": "Alice"}});
        let op = PatchOp::Replace {
            path: "/user/name".to_string(),
            value: json!("Bob"),
        };
        assert!(apply_operation(&mut doc, &op).is_ok());
        assert_eq!(doc["user"]["name"].as_str(), Some("Bob"));
    }

    #[test]
    fn test_replace_with_different_type() {
        let mut doc = json!({"x": 42});
        let op = PatchOp::Replace {
            path: "/x".to_string(),
            value: json!("now a string"),
        };
        assert!(apply_operation(&mut doc, &op).is_ok());
        assert_eq!(doc["x"].as_str(), Some("now a string"));
    }

    // --- move_value ---

    #[test]
    fn test_move_missing_source_fails() {
        let mut doc = json!({"a": 1});
        let op = PatchOp::Move {
            from: "/missing".to_string(),
            path: "/b".to_string(),
        };
        assert!(apply_operation(&mut doc, &op).is_err());
    }

    #[test]
    fn test_move_to_nested_destination() {
        let mut doc = json!({"src": "val", "obj": {}});
        let op = PatchOp::Move {
            from: "/src".to_string(),
            path: "/obj/dst".to_string(),
        };
        assert!(apply_operation(&mut doc, &op).is_ok());
        assert!(doc["src"].is_null());
        assert_eq!(doc["obj"]["dst"].as_str(), Some("val"));
    }

    #[test]
    fn test_move_within_array() {
        let mut doc = json!({"arr": [10, 20, 30]});
        let op = PatchOp::Move {
            from: "/arr/0".to_string(),
            path: "/arr/-".to_string(),
        };
        assert!(apply_operation(&mut doc, &op).is_ok());
        // 10 was removed from index 0, appended to end
        assert_eq!(doc["arr"][0], json!(20));
        assert_eq!(doc["arr"][2], json!(10));
    }

    // --- copy_value ---

    #[test]
    fn test_copy_missing_source_fails() {
        let mut doc = json!({"a": 1});
        let op = PatchOp::Copy {
            from: "/missing".to_string(),
            path: "/b".to_string(),
        };
        assert!(apply_operation(&mut doc, &op).is_err());
    }

    #[test]
    fn test_copy_preserves_source() {
        let mut doc = json!({"arr": [1, 2, 3]});
        let op = PatchOp::Copy {
            from: "/arr".to_string(),
            path: "/arr2".to_string(),
        };
        assert!(apply_operation(&mut doc, &op).is_ok());
        assert_eq!(doc["arr"][0], json!(1));
        assert_eq!(doc["arr2"][0], json!(1));
    }

    #[test]
    fn test_copy_nested_value() {
        let mut doc = json!({"a": {"x": 99}, "b": {}});
        let op = PatchOp::Copy {
            from: "/a/x".to_string(),
            path: "/b/y".to_string(),
        };
        assert!(apply_operation(&mut doc, &op).is_ok());
        assert_eq!(doc["b"]["y"], json!(99));
        assert_eq!(doc["a"]["x"], json!(99)); // source unchanged
    }

    // --- test_value ---

    #[test]
    fn test_test_missing_path_fails() {
        let mut doc = json!({"a": 1});
        let op = PatchOp::Test {
            path: "/missing".to_string(),
            value: json!(1),
        };
        assert!(apply_operation(&mut doc, &op).is_err());
    }

    #[test]
    fn test_test_null_value() {
        let mut doc = json!({"a": null});
        let op = PatchOp::Test {
            path: "/a".to_string(),
            value: json!(null),
        };
        assert!(apply_operation(&mut doc, &op).is_ok());
    }

    #[test]
    fn test_test_array_value() {
        let mut doc = json!({"arr": [1, 2, 3]});
        let op = PatchOp::Test {
            path: "/arr".to_string(),
            value: json!([1, 2, 3]),
        };
        assert!(apply_operation(&mut doc, &op).is_ok());

        let op_fail = PatchOp::Test {
            path: "/arr".to_string(),
            value: json!([1, 2]),
        };
        assert!(apply_operation(&mut doc, &op_fail).is_err());
    }

    #[test]
    fn test_test_nested_object() {
        let mut doc = json!({"user": {"name": "Alice"}});
        let op = PatchOp::Test {
            path: "/user/name".to_string(),
            value: json!("Alice"),
        };
        assert!(apply_operation(&mut doc, &op).is_ok());
    }

    // --- apply_patch (multiple ops) ---

    #[test]
    fn test_apply_patch_empty_ops() {
        let mut doc = json!({"a": 1});
        assert!(apply_patch(&mut doc, &[]).is_ok());
        assert_eq!(doc["a"], json!(1));
    }

    #[test]
    fn test_apply_patch_multiple_ops() {
        let mut doc = json!({"a": 1, "b": 2});
        let ops = vec![
            PatchOp::Replace {
                path: "/a".to_string(),
                value: json!(10),
            },
            PatchOp::Remove {
                path: "/b".to_string(),
            },
            PatchOp::Add {
                path: "/c".to_string(),
                value: json!(30),
            },
        ];
        assert!(apply_patch(&mut doc, &ops).is_ok());
        assert_eq!(doc["a"], json!(10));
        assert!(doc["b"].is_null());
        assert_eq!(doc["c"], json!(30));
    }

    #[test]
    fn test_apply_patch_stops_on_first_error() {
        let mut doc = json!({"a": 1});
        let ops = vec![
            PatchOp::Remove {
                path: "/missing".to_string(),
            }, // fails
            PatchOp::Add {
                path: "/b".to_string(),
                value: json!(2),
            }, // never reached
        ];
        assert!(apply_patch(&mut doc, &ops).is_err());
        assert!(doc["b"].is_null()); // not added
    }

    #[test]
    fn test_apply_patch_atomicity_not_guaranteed() {
        // apply_patch does NOT roll back on failure — verify side-effects of prior ops remain
        let mut doc = json!({"a": 1, "b": 2});
        let ops = vec![
            PatchOp::Replace {
                path: "/a".to_string(),
                value: json!(99),
            }, // succeeds
            PatchOp::Remove {
                path: "/nonexistent".to_string(),
            }, // fails
        ];
        let result = apply_patch(&mut doc, &ops);
        assert!(result.is_err());
        assert_eq!(doc["a"], json!(99)); // side-effect remains
    }

    #[test]
    fn test_apply_patch_test_then_modify() {
        let mut doc = json!({"version": 1, "data": "old"});
        let ops = vec![
            PatchOp::Test {
                path: "/version".to_string(),
                value: json!(1),
            },
            PatchOp::Replace {
                path: "/data".to_string(),
                value: json!("new"),
            },
        ];
        assert!(apply_patch(&mut doc, &ops).is_ok());
        assert_eq!(doc["data"].as_str(), Some("new"));
    }

    #[test]
    fn test_apply_patch_test_fails_blocks_modification() {
        let mut doc = json!({"version": 1, "data": "old"});
        let ops = vec![
            PatchOp::Test {
                path: "/version".to_string(),
                value: json!(2),
            }, // wrong version
            PatchOp::Replace {
                path: "/data".to_string(),
                value: json!("new"),
            },
        ];
        assert!(apply_patch(&mut doc, &ops).is_err());
        assert_eq!(doc["data"].as_str(), Some("old")); // unchanged
    }

    // --- parse_patch ---

    #[test]
    fn test_parse_patch_not_array_fails() {
        let not_array = json!({"op": "add"});
        assert!(parse_patch(&not_array).is_err());
    }

    #[test]
    fn test_parse_patch_operation_not_object_fails() {
        let doc = json!([42]);
        assert!(parse_patch(&doc).is_err());
    }

    #[test]
    fn test_parse_patch_missing_op_field_fails() {
        let doc: Node = r#"[{"path": "/a", "value": 1}]"#.parse().unwrap();
        assert!(parse_patch(&doc).is_err());
    }

    #[test]
    fn test_parse_patch_unknown_op_fails() {
        let doc: Node = r#"[{"op": "unknown", "path": "/a"}]"#.parse().unwrap();
        assert!(parse_patch(&doc).is_err());
    }

    #[test]
    fn test_parse_patch_add_missing_value_fails() {
        let doc: Node = r#"[{"op": "add", "path": "/a"}]"#.parse().unwrap();
        assert!(parse_patch(&doc).is_err());
    }

    #[test]
    fn test_parse_patch_remove_missing_path_fails() {
        let doc: Node = r#"[{"op": "remove"}]"#.parse().unwrap();
        assert!(parse_patch(&doc).is_err());
    }

    #[test]
    fn test_parse_patch_replace_missing_value_fails() {
        let doc: Node = r#"[{"op": "replace", "path": "/a"}]"#.parse().unwrap();
        assert!(parse_patch(&doc).is_err());
    }

    #[test]
    fn test_parse_patch_move_missing_from_fails() {
        let doc: Node = r#"[{"op": "move", "path": "/b"}]"#.parse().unwrap();
        assert!(parse_patch(&doc).is_err());
    }

    #[test]
    fn test_parse_patch_copy_missing_from_fails() {
        let doc: Node = r#"[{"op": "copy", "path": "/b"}]"#.parse().unwrap();
        assert!(parse_patch(&doc).is_err());
    }

    #[test]
    fn test_parse_patch_test_missing_value_fails() {
        let doc: Node = r#"[{"op": "test", "path": "/a"}]"#.parse().unwrap();
        assert!(parse_patch(&doc).is_err());
    }

    #[test]
    fn test_parse_patch_all_op_types() {
        let doc: Node = r#"[
            {"op": "add",     "path": "/a", "value": 1},
            {"op": "remove",  "path": "/b"},
            {"op": "replace", "path": "/c", "value": 2},
            {"op": "move",    "from": "/d", "path": "/e"},
            {"op": "copy",    "from": "/f", "path": "/g"},
            {"op": "test",    "path": "/h", "value": 3}
        ]"#
        .parse()
        .unwrap();
        let ops = parse_patch(&doc).unwrap();
        assert_eq!(ops.len(), 6);
        assert_eq!(
            ops[0],
            PatchOp::Add {
                path: "/a".to_string(),
                value: Node::from(1i64)
            }
        );
        assert_eq!(
            ops[1],
            PatchOp::Remove {
                path: "/b".to_string()
            }
        );
        assert_eq!(
            ops[2],
            PatchOp::Replace {
                path: "/c".to_string(),
                value: Node::from(2i64)
            }
        );
        assert_eq!(
            ops[3],
            PatchOp::Move {
                from: "/d".to_string(),
                path: "/e".to_string()
            }
        );
        assert_eq!(
            ops[4],
            PatchOp::Copy {
                from: "/f".to_string(),
                path: "/g".to_string()
            }
        );
        assert_eq!(
            ops[5],
            PatchOp::Test {
                path: "/h".to_string(),
                value: Node::from(3i64)
            }
        );
    }

    #[test]
    fn test_parse_then_apply_roundtrip() {
        let mut doc = json!({"a": 1, "b": 2});
        let patch_doc: Node = r#"[
            {"op": "add",     "path": "/c", "value": 3},
            {"op": "remove",  "path": "/b"},
            {"op": "replace", "path": "/a", "value": 99}
        ]"#
        .parse()
        .unwrap();
        let ops = parse_patch(&patch_doc).unwrap();
        apply_patch(&mut doc, &ops).unwrap();
        assert_eq!(doc["a"].as_i64(), Some(99));
        assert!(doc["b"].is_null());
        assert_eq!(doc["c"].as_i64(), Some(3));
    }

    #[test]
    fn test_parse_patch_empty_array() {
        let doc = json!([]);
        let ops = parse_patch(&doc).unwrap();
        assert_eq!(ops.len(), 0);
    }
}
