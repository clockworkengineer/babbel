//! Structural AST diffing engine between `Value` trees.
//!
//! Generates minimal RFC 6902 JSON Patch operations or RFC 7396 Merge Patch documents.

use crate::model::Value;
use crate::patch::{Patch, PatchOp};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// Computes an RFC 6902 [`Patch`] representing the delta from `source` to `target`.
pub fn diff(source: &Value, target: &Value) -> Patch {
    let mut ops = Vec::new();
    diff_recursive(source, target, "", &mut ops);
    Patch::new(ops)
}

fn diff_recursive(source: &Value, target: &Value, path: &str, ops: &mut Vec<PatchOp>) {
    if source == target {
        return;
    }

    match (source, target) {
        (Value::Object(src_entries), Value::Object(tgt_entries)) => {
            // Find removed keys
            for (key, _) in src_entries {
                if !tgt_entries.iter().any(|(k, _)| k == key) {
                    ops.push(PatchOp::Remove {
                        path: escape_pointer_segment(path, key),
                    });
                }
            }

            // Find added and modified keys
            for (key, tgt_val) in tgt_entries {
                if let Some((_, src_val)) = src_entries.iter().find(|(k, _)| k == key) {
                    diff_recursive(src_val, tgt_val, &escape_pointer_segment(path, key), ops);
                } else {
                    ops.push(PatchOp::Add {
                        path: escape_pointer_segment(path, key),
                        value: tgt_val.clone(),
                    });
                }
            }
        }
        (Value::Array(src_items), Value::Array(tgt_items)) => {
            let common_len = src_items.len().min(tgt_items.len());

            for i in 0..common_len {
                let seg = format!("{}/{}", path, i);
                diff_recursive(&src_items[i], &tgt_items[i], &seg, ops);
            }

            if tgt_items.len() > src_items.len() {
                // Elements were added to the end
                for i in common_len..tgt_items.len() {
                    let seg = format!("{}/{}", path, i);
                    ops.push(PatchOp::Add {
                        path: seg,
                        value: tgt_items[i].clone(),
                    });
                }
            } else if src_items.len() > tgt_items.len() {
                // Elements were removed from the end (remove in reverse order to preserve indices)
                for i in (tgt_items.len()..src_items.len()).rev() {
                    let seg = format!("{}/{}", path, i);
                    ops.push(PatchOp::Remove { path: seg });
                }
            }
        }
        _ => {
            let p = if path.is_empty() {
                "".to_string()
            } else {
                path.to_string()
            };
            ops.push(PatchOp::Replace {
                path: p,
                value: target.clone(),
            });
        }
    }
}

/// Computes an RFC 7396 JSON Merge Patch document representing changes from `source` to `target`.
pub fn diff_merge_patch(source: &Value, target: &Value) -> Value {
    if source == target {
        return Value::Object(Vec::new());
    }

    match (source, target) {
        (Value::Object(src_entries), Value::Object(tgt_entries)) => {
            let mut patch_entries = Vec::new();

            // Removed keys: set to null
            for (key, _) in src_entries {
                if !tgt_entries.iter().any(|(k, _)| k == key) {
                    patch_entries.push((key.clone(), Value::Null));
                }
            }

            // Added or modified keys
            for (key, tgt_val) in tgt_entries {
                if let Some((_, src_val)) = src_entries.iter().find(|(k, _)| k == key) {
                    if src_val != tgt_val {
                        if src_val.is_object() && tgt_val.is_object() {
                            let sub_patch = diff_merge_patch(src_val, tgt_val);
                            patch_entries.push((key.clone(), sub_patch));
                        } else {
                            patch_entries.push((key.clone(), tgt_val.clone()));
                        }
                    }
                } else {
                    patch_entries.push((key.clone(), tgt_val.clone()));
                }
            }

            Value::Object(patch_entries)
        }
        _ => target.clone(),
    }
}

fn escape_pointer_segment(base: &str, segment: &str) -> String {
    let escaped = if segment.contains('~') || segment.contains('/') {
        segment.replace('~', "~0").replace('/', "~1")
    } else {
        segment.to_string()
    };
    format!("{}/{}", base, escaped)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_diff_roundtrip_objects() {
        let source = Value::Object(vec![
            ("a".to_string(), Value::Integer(1)),
            ("b".to_string(), Value::String("old".to_string())),
            ("to_remove".to_string(), Value::Bool(true)),
        ]);

        let target = Value::Object(vec![
            ("a".to_string(), Value::Integer(1)),
            ("added".to_string(), Value::Float(3.14)),
            ("b".to_string(), Value::String("new".to_string())),
        ]);

        let patch = diff(&source, &target);
        let transformed = patch.apply(&source).unwrap();
        assert_eq!(transformed, target);
    }

    #[test]
    fn test_diff_roundtrip_arrays() {
        let source = Value::Array(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]);

        let target = Value::Array(vec![
            Value::Integer(1),
            Value::Integer(99),
            Value::Integer(3),
            Value::Integer(4),
        ]);

        let patch = diff(&source, &target);
        let transformed = patch.apply(&source).unwrap();
        assert_eq!(transformed, target);
    }

    #[test]
    fn test_diff_merge_patch_roundtrip() {
        let source = Value::Object(vec![
            (
                "author".to_string(),
                Value::Object(vec![
                    ("first".to_string(), Value::String("Alice".to_string())),
                    ("last".to_string(), Value::String("Smith".to_string())),
                ]),
            ),
            ("old_key".to_string(), Value::Integer(123)),
            ("title".to_string(), Value::String("Hello".to_string())),
        ]);

        let target = Value::Object(vec![
            (
                "author".to_string(),
                Value::Object(vec![
                    ("first".to_string(), Value::String("Alice".to_string())),
                    ("last".to_string(), Value::String("Jones".to_string())),
                ]),
            ),
            ("new_key".to_string(), Value::Bool(true)),
            (
                "title".to_string(),
                Value::String("Hello World".to_string()),
            ),
        ]);

        let merge_patch = diff_merge_patch(&source, &target);
        let mut source_copy = source.clone();
        source_copy.merge_patch(&merge_patch);
        assert_eq!(source_copy, target);
    }
}
