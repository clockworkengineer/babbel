//! RFC 7396 JSON Merge Patch implementation.

use crate::model::Value;

/// Applies an RFC 7396 JSON Merge Patch in-place to `target`.
///
/// If `patch` is an object:
/// - If `target` is not an object, `target` is replaced with an empty object.
/// - For each member `(name, value)` of `patch`:
///   - If `value` is `Value::Null`, member `name` is removed from `target`.
///   - Otherwise, `apply_merge_patch` is called recursively on `target[name]` and `value`.
///
/// If `patch` is not an object:
/// - `target` is replaced with a clone of `patch`.
pub fn apply_merge_patch(target: &mut Value, patch: &Value) {
    match patch {
        Value::Object(patch_entries) => {
            if !target.is_object() {
                *target = Value::Object(alloc::vec::Vec::new());
            }

            if let Value::Object(target_entries) = target {
                for (name, val) in patch_entries {
                    if val.is_null() {
                        // Remove key
                        if let Some(pos) = target_entries.iter().position(|(k, _)| k == name) {
                            target_entries.remove(pos);
                        }
                    } else {
                        // Update or insert
                        if let Some(pos) = target_entries.iter().position(|(k, _)| k == name) {
                            apply_merge_patch(&mut target_entries[pos].1, val);
                        } else {
                            let mut new_val = Value::Null;
                            apply_merge_patch(&mut new_val, val);
                            target_entries.push((name.clone(), new_val));
                            target_entries.sort_by(|a, b| a.0.cmp(&b.0));
                        }
                    }
                }
            }
        }
        other => {
            *target = other.clone();
        }
    }
}
