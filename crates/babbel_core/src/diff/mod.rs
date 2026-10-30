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
            diff_arrays_lcs(src_items, tgt_items, path, ops);
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

/// Computes LCS-guided array diff operations that remain index-coherent when applied sequentially.
fn diff_arrays_lcs(src_items: &[Value], tgt_items: &[Value], path: &str, ops: &mut Vec<PatchOp>) {
    let m = src_items.len();
    let n = tgt_items.len();

    // Compute LCS anchors: pairs of (src_idx, tgt_idx) where src_items[src_idx] == tgt_items[tgt_idx]
    let anchors = if m.saturating_mul(n) <= 250_000 {
        compute_lcs_anchors(src_items, tgt_items)
    } else {
        Vec::new()
    };

    let mut cur_idx = 0usize;
    let mut prev_src = 0usize;
    let mut prev_tgt = 0usize;

    for (anchor_src, anchor_tgt) in anchors.into_iter().chain(core::iter::once((m, n))) {
        let src_gap = anchor_src - prev_src;
        let tgt_gap = anchor_tgt - prev_tgt;
        let paired = src_gap.min(tgt_gap);

        // Diff overlapping items in the gap in-place (enables nested object/array edits or Replace)
        for offset in 0..paired {
            let seg = format!("{}/{}", path, cur_idx);
            diff_recursive(
                &src_items[prev_src + offset],
                &tgt_items[prev_tgt + offset],
                &seg,
                ops,
            );
            cur_idx += 1;
        }

        // Remove excess source items in the gap at `cur_idx`
        if src_gap > paired {
            for _ in paired..src_gap {
                let seg = format!("{}/{}", path, cur_idx);
                ops.push(PatchOp::Remove { path: seg });
            }
        }

        // Insert excess target items in the gap at `cur_idx`
        if tgt_gap > paired {
            for offset in paired..tgt_gap {
                let seg = format!("{}/{}", path, cur_idx);
                ops.push(PatchOp::Add {
                    path: seg,
                    value: tgt_items[prev_tgt + offset].clone(),
                });
                cur_idx += 1;
            }
        }

        // Advance past the matched anchor itself
        if anchor_src < m && anchor_tgt < n {
            cur_idx += 1;
        }
        prev_src = anchor_src + 1;
        prev_tgt = anchor_tgt + 1;
    }
}

fn compute_lcs_anchors(src: &[Value], tgt: &[Value]) -> Vec<(usize, usize)> {
    let m = src.len();
    let n = tgt.len();
    if m == 0 || n == 0 {
        return Vec::new();
    }

    let cols = n + 1;
    let mut dp = alloc::vec![0u32; (m + 1) * cols];

    for i in (0..m).rev() {
        for j in (0..n).rev() {
            let v = if src[i] == tgt[j] {
                1 + dp[(i + 1) * cols + (j + 1)]
            } else {
                dp[(i + 1) * cols + j].max(dp[i * cols + (j + 1)])
            };
            dp[i * cols + j] = v;
        }
    }

    let mut anchors = Vec::new();
    let mut i = 0;
    let mut j = 0;
    while i < m && j < n {
        if src[i] == tgt[j] {
            anchors.push((i, j));
            i += 1;
            j += 1;
        } else if dp[(i + 1) * cols + j] >= dp[i * cols + (j + 1)] {
            i += 1;
        } else {
            j += 1;
        }
    }
    anchors
}

/// Formats a human-readable unified diff between two [`Value`] trees.
pub fn format_unified_diff(source: &Value, target: &Value) -> String {
    let patch = diff(source, target);
    if patch.ops.is_empty() {
        return String::new();
    }
    let mut out = String::from("--- source\n+++ target\n");
    for op in &patch.ops {
        match op {
            PatchOp::Add { path, value } => {
                let mut buf = crate::io::Buffer::new();
                value.serialize_json(&mut buf);
                out.push_str(&format!("+ {}: {}\n", path, buf.to_string()));
            }
            PatchOp::Remove { path } => {
                if let Some(old_val) = source.pointer(path) {
                    let mut buf = crate::io::Buffer::new();
                    old_val.serialize_json(&mut buf);
                    out.push_str(&format!("- {}: {}\n", path, buf.to_string()));
                } else {
                    out.push_str(&format!("- {}\n", path));
                }
            }
            PatchOp::Replace { path, value } => {
                if let Some(old_val) = source.pointer(path) {
                    let mut old_buf = crate::io::Buffer::new();
                    old_val.serialize_json(&mut old_buf);
                    out.push_str(&format!("- {}: {}\n", path, old_buf.to_string()));
                }
                let mut new_buf = crate::io::Buffer::new();
                value.serialize_json(&mut new_buf);
                out.push_str(&format!("+ {}: {}\n", path, new_buf.to_string()));
            }
            PatchOp::Move { from, path } => {
                out.push_str(&format!("~ {} -> {}\n", from, path));
            }
            PatchOp::Copy { from, path } => {
                out.push_str(&format!("+ {} (from {})\n", path, from));
            }
            PatchOp::Test { path, value } => {
                let mut buf = crate::io::Buffer::new();
                value.serialize_json(&mut buf);
                out.push_str(&format!("= {}: {}\n", path, buf.to_string()));
            }
        }
    }
    out
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

    #[test]
    fn test_diff_lcs_array_insert_and_remove() {
        let source = Value::Array(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
            Value::Integer(4),
        ]);
        let target = Value::Array(vec![
            Value::Integer(0),
            Value::Integer(1),
            Value::Integer(3),
            Value::Integer(4),
            Value::Integer(5),
        ]);

        let patch = diff(&source, &target);
        // Should only require: Add /0 (0), Remove /2 (which was 2), Add /4 (5) = 3 ops
        assert_eq!(patch.ops.len(), 3);
        let transformed = patch.apply(&source).unwrap();
        assert_eq!(transformed, target);

        let unified = format_unified_diff(&source, &target);
        assert!(unified.contains("--- source"));
        assert!(unified.contains("+++ target"));
        assert!(unified.contains("+ /0: 0"));
    }
}
