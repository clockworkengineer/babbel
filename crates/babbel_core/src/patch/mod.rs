//! RFC 6902 JSON Patch and RFC 7396 JSON Merge Patch implementation for `Value`.

pub mod merge;

use crate::error::BabbelError;
use crate::model::Value;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

pub use merge::apply_merge_patch;

/// A single RFC 6902 patch operation.
#[derive(Debug, Clone, PartialEq)]
pub enum PatchOp {
    /// Add a value at path: `{"op": "add", "path": "/a/b", "value": ...}`
    Add { path: String, value: Value },
    /// Remove a value at path: `{"op": "remove", "path": "/a/b"}`
    Remove { path: String },
    /// Replace a value at path: `{"op": "replace", "path": "/a/b", "value": ...}`
    Replace { path: String, value: Value },
    /// Move a value from `from` to `path`: `{"op": "move", "from": "/a", "path": "/b"}`
    Move { from: String, path: String },
    /// Copy a value from `from` to `path`: `{"op": "copy", "from": "/a", "path": "/b"}`
    Copy { from: String, path: String },
    /// Test that value at `path` equals `value`: `{"op": "test", "path": "/a", "value": ...}`
    Test { path: String, value: Value },
}

/// An ordered sequence of RFC 6902 patch operations.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Patch {
    pub ops: Vec<PatchOp>,
}

impl Patch {
    /// Create a new patch from a vector of operations.
    pub fn new(ops: Vec<PatchOp>) -> Self {
        Self { ops }
    }

    /// Parse an RFC 6902 JSON Patch from a `Value::Array`.
    pub fn from_value(val: &Value) -> Result<Self, BabbelError> {
        let items = match val {
            Value::Array(items) => items,
            _ => {
                return Err(BabbelError::syntax(
                    "JSON Patch must be an array of operation objects",
                ));
            }
        };

        let mut ops = Vec::with_capacity(items.len());
        for (idx, item) in items.iter().enumerate() {
            let obj = match item {
                Value::Object(obj) => obj,
                _ => {
                    return Err(BabbelError::syntax(format!(
                        "patch operation at index {} must be an object",
                        idx
                    )));
                }
            };

            let op_str = get_obj_str(obj, "op").ok_or_else(|| {
                BabbelError::syntax(format!("missing 'op' in patch operation at index {}", idx))
            })?;
            let path = get_obj_str(obj, "path")
                .ok_or_else(|| {
                    BabbelError::syntax(format!(
                        "missing 'path' in patch operation at index {}",
                        idx
                    ))
                })?
                .to_string();

            match op_str {
                "add" => {
                    let value = get_obj_val(obj, "value")
                        .ok_or_else(|| {
                            BabbelError::syntax(format!(
                                "missing 'value' in add operation at index {}",
                                idx
                            ))
                        })?
                        .clone();
                    ops.push(PatchOp::Add { path, value });
                }
                "remove" => {
                    ops.push(PatchOp::Remove { path });
                }
                "replace" => {
                    let value = get_obj_val(obj, "value")
                        .ok_or_else(|| {
                            BabbelError::syntax(format!(
                                "missing 'value' in replace operation at index {}",
                                idx
                            ))
                        })?
                        .clone();
                    ops.push(PatchOp::Replace { path, value });
                }
                "move" => {
                    let from = get_obj_str(obj, "from")
                        .ok_or_else(|| {
                            BabbelError::syntax(format!(
                                "missing 'from' in move operation at index {}",
                                idx
                            ))
                        })?
                        .to_string();
                    ops.push(PatchOp::Move { from, path });
                }
                "copy" => {
                    let from = get_obj_str(obj, "from")
                        .ok_or_else(|| {
                            BabbelError::syntax(format!(
                                "missing 'from' in copy operation at index {}",
                                idx
                            ))
                        })?
                        .to_string();
                    ops.push(PatchOp::Copy { from, path });
                }
                "test" => {
                    let value = get_obj_val(obj, "value")
                        .ok_or_else(|| {
                            BabbelError::syntax(format!(
                                "missing 'value' in test operation at index {}",
                                idx
                            ))
                        })?
                        .clone();
                    ops.push(PatchOp::Test { path, value });
                }
                unknown => {
                    return Err(BabbelError::syntax(format!(
                        "unrecognized patch op '{}' at index {}",
                        unknown, idx
                    )));
                }
            }
        }

        Ok(Self { ops })
    }

    /// Serialize this patch into an RFC 6902 `Value::Array`.
    pub fn to_value(&self) -> Value {
        let mut arr = Vec::with_capacity(self.ops.len());
        for op in &self.ops {
            let mut entries = Vec::new();
            match op {
                PatchOp::Add { path, value } => {
                    entries.push(("op".to_string(), Value::String("add".to_string())));
                    entries.push(("path".to_string(), Value::String(path.clone())));
                    entries.push(("value".to_string(), value.clone()));
                }
                PatchOp::Remove { path } => {
                    entries.push(("op".to_string(), Value::String("remove".to_string())));
                    entries.push(("path".to_string(), Value::String(path.clone())));
                }
                PatchOp::Replace { path, value } => {
                    entries.push(("op".to_string(), Value::String("replace".to_string())));
                    entries.push(("path".to_string(), Value::String(path.clone())));
                    entries.push(("value".to_string(), value.clone()));
                }
                PatchOp::Move { from, path } => {
                    entries.push(("op".to_string(), Value::String("move".to_string())));
                    entries.push(("from".to_string(), Value::String(from.clone())));
                    entries.push(("path".to_string(), Value::String(path.clone())));
                }
                PatchOp::Copy { from, path } => {
                    entries.push(("op".to_string(), Value::String("copy".to_string())));
                    entries.push(("from".to_string(), Value::String(from.clone())));
                    entries.push(("path".to_string(), Value::String(path.clone())));
                }
                PatchOp::Test { path, value } => {
                    entries.push(("op".to_string(), Value::String("test".to_string())));
                    entries.push(("path".to_string(), Value::String(path.clone())));
                    entries.push(("value".to_string(), value.clone()));
                }
            }
            arr.push(Value::Object(entries));
        }
        Value::Array(arr)
    }

    /// Apply this patch atomically to target, returning the new transformed `Value`.
    ///
    /// If any operation fails (or a `test` condition fails), an error is returned and
    /// the original target remains untouched.
    pub fn apply(&self, target: &Value) -> Result<Value, BabbelError> {
        let mut copy = target.clone();
        self.apply_inplace(&mut copy)?;
        Ok(copy)
    }

    /// Apply this patch in-place to target.
    pub fn apply_inplace(&self, target: &mut Value) -> Result<(), BabbelError> {
        // Create a backup snapshot for transactional rollback in case of error
        let snapshot = target.clone();

        for (idx, op) in self.ops.iter().enumerate() {
            if let Err(e) = apply_op(op, target) {
                *target = snapshot; // Rollback
                return Err(BabbelError::syntax(format!(
                    "patch op failed at index {}: {}",
                    idx, e.message
                )));
            }
        }
        Ok(())
    }
}

fn apply_op(op: &PatchOp, doc: &mut Value) -> Result<(), BabbelError> {
    match op {
        PatchOp::Add { path, value } => {
            let tokens = parse_pointer(path)?;
            if tokens.is_empty() {
                *doc = value.clone();
                return Ok(());
            }
            let (parent_tokens, last_token) = tokens.split_at(tokens.len() - 1);
            let parent = navigate_mut(doc, parent_tokens)?;

            match parent {
                Value::Object(entries) => {
                    let key = &last_token[0];
                    if let Some(pos) = entries.iter().position(|(k, _)| k == key) {
                        entries[pos].1 = value.clone();
                    } else {
                        entries.push((key.clone(), value.clone()));
                        entries.sort_by(|a, b| a.0.cmp(&b.0));
                    }
                    Ok(())
                }
                Value::Array(items) => {
                    let token = &last_token[0];
                    if token == "-" {
                        items.push(value.clone());
                        Ok(())
                    } else if let Ok(idx) = token.parse::<usize>() {
                        if idx <= items.len() {
                            items.insert(idx, value.clone());
                            Ok(())
                        } else {
                            Err(BabbelError::syntax("add index out of bounds"))
                        }
                    } else {
                        Err(BabbelError::syntax("invalid array index"))
                    }
                }
                _ => Err(BabbelError::syntax("cannot add to non-container")),
            }
        }
        PatchOp::Remove { path } => {
            let tokens = parse_pointer(path)?;
            if tokens.is_empty() {
                return Err(BabbelError::syntax("cannot remove root element"));
            }
            let (parent_tokens, last_token) = tokens.split_at(tokens.len() - 1);
            let parent = navigate_mut(doc, parent_tokens)?;

            match parent {
                Value::Object(entries) => {
                    let key = &last_token[0];
                    if let Some(pos) = entries.iter().position(|(k, _)| k == key) {
                        entries.remove(pos);
                        Ok(())
                    } else {
                        Err(BabbelError::syntax("remove key not found"))
                    }
                }
                Value::Array(items) => {
                    let token = &last_token[0];
                    if let Ok(idx) = token.parse::<usize>() {
                        if idx < items.len() {
                            items.remove(idx);
                            Ok(())
                        } else {
                            Err(BabbelError::syntax("remove index out of bounds"))
                        }
                    } else {
                        Err(BabbelError::syntax("invalid array index for remove"))
                    }
                }
                _ => Err(BabbelError::syntax("cannot remove from non-container")),
            }
        }
        PatchOp::Replace { path, value } => {
            let tokens = parse_pointer(path)?;
            if tokens.is_empty() {
                *doc = value.clone();
                return Ok(());
            }
            let (parent_tokens, last_token) = tokens.split_at(tokens.len() - 1);
            let parent = navigate_mut(doc, parent_tokens)?;

            match parent {
                Value::Object(entries) => {
                    let key = &last_token[0];
                    if let Some(pos) = entries.iter().position(|(k, _)| k == key) {
                        entries[pos].1 = value.clone();
                        Ok(())
                    } else {
                        Err(BabbelError::syntax("replace target key not found"))
                    }
                }
                Value::Array(items) => {
                    let token = &last_token[0];
                    if let Ok(idx) = token.parse::<usize>() {
                        if idx < items.len() {
                            items[idx] = value.clone();
                            Ok(())
                        } else {
                            Err(BabbelError::syntax("replace index out of bounds"))
                        }
                    } else {
                        Err(BabbelError::syntax("invalid array index for replace"))
                    }
                }
                _ => Err(BabbelError::syntax("cannot replace in non-container")),
            }
        }
        PatchOp::Move { from, path } => {
            // Retrieve value from source
            let val = {
                let from_tokens = parse_pointer(from)?;
                if from_tokens.is_empty() {
                    return Err(BabbelError::syntax("cannot move entire root document"));
                }
                let (from_parent, from_last) = from_tokens.split_at(from_tokens.len() - 1);
                let parent = navigate_mut(doc, from_parent)?;
                match parent {
                    Value::Object(entries) => {
                        let key = &from_last[0];
                        if let Some(pos) = entries.iter().position(|(k, _)| k == key) {
                            entries.remove(pos).1
                        } else {
                            return Err(BabbelError::syntax("move source key not found"));
                        }
                    }
                    Value::Array(items) => {
                        let token = &from_last[0];
                        if let Ok(idx) = token.parse::<usize>() {
                            if idx < items.len() {
                                items.remove(idx)
                            } else {
                                return Err(BabbelError::syntax("move source index out of bounds"));
                            }
                        } else {
                            return Err(BabbelError::syntax("invalid array index for move"));
                        }
                    }
                    _ => return Err(BabbelError::syntax("cannot move from non-container")),
                }
            };

            // Add value to destination
            apply_op(
                &PatchOp::Add {
                    path: path.clone(),
                    value: val,
                },
                doc,
            )
        }
        PatchOp::Copy { from, path } => {
            let val = {
                let node = doc
                    .pointer(from)
                    .ok_or_else(|| BabbelError::syntax("copy source not found"))?;
                node.clone()
            };
            apply_op(
                &PatchOp::Add {
                    path: path.clone(),
                    value: val,
                },
                doc,
            )
        }
        PatchOp::Test { path, value } => {
            let current = doc
                .pointer(path)
                .ok_or_else(|| BabbelError::syntax("test target not found"))?;
            if current == value {
                Ok(())
            } else {
                Err(BabbelError::syntax("test operation equality mismatch"))
            }
        }
    }
}

fn parse_pointer(ptr: &str) -> Result<Vec<String>, BabbelError> {
    if ptr.is_empty() {
        return Ok(Vec::new());
    }
    if !ptr.starts_with('/') {
        return Err(BabbelError::syntax("JSON pointer must start with '/'"));
    }
    Ok(ptr
        .split('/')
        .skip(1)
        .map(|seg| {
            if seg.contains('~') {
                seg.replace("~1", "/").replace("~0", "~")
            } else {
                seg.to_string()
            }
        })
        .collect())
}

fn navigate_mut<'a>(doc: &'a mut Value, tokens: &[String]) -> Result<&'a mut Value, BabbelError> {
    let mut current = doc;
    for token in tokens {
        match current {
            Value::Object(entries) => {
                if let Some(pos) = entries.iter().position(|(k, _)| k == token) {
                    current = &mut entries[pos].1;
                } else {
                    return Err(BabbelError::syntax(format!(
                        "property '{}' not found",
                        token
                    )));
                }
            }
            Value::Array(items) => {
                if let Ok(idx) = token.parse::<usize>() {
                    if idx < items.len() {
                        current = &mut items[idx];
                    } else {
                        return Err(BabbelError::syntax("index out of bounds"));
                    }
                } else {
                    return Err(BabbelError::syntax("invalid array index in pointer"));
                }
            }
            _ => return Err(BabbelError::syntax("cannot navigate into non-container")),
        }
    }
    Ok(current)
}

fn get_obj_str<'a>(obj: &'a [(String, Value)], key: &str) -> Option<&'a str> {
    obj.iter()
        .find(|(k, _)| k == key)
        .and_then(|(_, v)| v.as_str())
}

fn get_obj_val<'a>(obj: &'a [(String, Value)], key: &str) -> Option<&'a Value> {
    obj.iter().find(|(k, _)| k == key).map(|(_, v)| v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_patch_add_replace_remove() {
        let mut doc = Value::Object(vec![
            ("foo".to_string(), Value::String("bar".to_string())),
            (
                "numbers".to_string(),
                Value::Array(vec![Value::Integer(1), Value::Integer(2)]),
            ),
        ]);

        let patch = Patch::new(vec![
            PatchOp::Add {
                path: "/baz".to_string(),
                value: Value::String("qux".to_string()),
            },
            PatchOp::Replace {
                path: "/foo".to_string(),
                value: Value::String("updated".to_string()),
            },
            PatchOp::Add {
                path: "/numbers/1".to_string(),
                value: Value::Integer(99),
            },
            PatchOp::Remove {
                path: "/numbers/0".to_string(),
            },
        ]);

        patch.apply_inplace(&mut doc).unwrap();

        assert_eq!(doc.pointer("/foo").unwrap().as_str(), Some("updated"));
        assert_eq!(doc.pointer("/baz").unwrap().as_str(), Some("qux"));
        assert_eq!(doc.pointer("/numbers/0").unwrap().as_i64(), Some(99));
        assert_eq!(doc.pointer("/numbers/1").unwrap().as_i64(), Some(2));
    }

    #[test]
    fn test_patch_move_copy_test() {
        let doc = Value::Object(vec![("foo".to_string(), Value::String("bar".to_string()))]);

        let patch = Patch::new(vec![
            PatchOp::Test {
                path: "/foo".to_string(),
                value: Value::String("bar".to_string()),
            },
            PatchOp::Copy {
                from: "/foo".to_string(),
                path: "/foo_copy".to_string(),
            },
            PatchOp::Move {
                from: "/foo".to_string(),
                path: "/foo_moved".to_string(),
            },
        ]);

        let res = patch.apply(&doc).unwrap();
        assert!(res.pointer("/foo").is_none());
        assert_eq!(res.pointer("/foo_copy").unwrap().as_str(), Some("bar"));
        assert_eq!(res.pointer("/foo_moved").unwrap().as_str(), Some("bar"));
    }

    #[test]
    fn test_patch_rollback_on_test_failure() {
        let mut doc = Value::Object(vec![("count".to_string(), Value::Integer(42))]);

        let patch = Patch::new(vec![
            PatchOp::Replace {
                path: "/count".to_string(),
                value: Value::Integer(100),
            },
            PatchOp::Test {
                path: "/count".to_string(),
                value: Value::Integer(999),
            }, // Fails!
        ]);

        assert!(patch.apply_inplace(&mut doc).is_err());
        // Value should be rolled back to 42
        assert_eq!(doc.pointer("/count").unwrap().as_i64(), Some(42));
    }

    #[test]
    fn test_merge_patch() {
        let mut target = Value::Object(vec![
            ("title".to_string(), Value::String("Goodbye!".to_string())),
            (
                "author".to_string(),
                Value::Object(vec![
                    ("givenName".to_string(), Value::String("John".to_string())),
                    ("familyName".to_string(), Value::String("Doe".to_string())),
                ]),
            ),
            (
                "tags".to_string(),
                Value::Array(vec![Value::String("example".to_string())]),
            ),
        ]);

        let patch = Value::Object(vec![
            ("title".to_string(), Value::String("Hello!".to_string())),
            (
                "author".to_string(),
                Value::Object(vec![
                    ("familyName".to_string(), Value::Null), // Delete familyName
                ]),
            ),
            (
                "tags".to_string(),
                Value::Array(vec![Value::String("rust".to_string())]),
            ),
        ]);

        target.merge_patch(&patch);

        assert_eq!(target.pointer("/title").unwrap().as_str(), Some("Hello!"));
        assert_eq!(
            target.pointer("/author/givenName").unwrap().as_str(),
            Some("John")
        );
        assert!(target.pointer("/author/familyName").is_none());
        assert_eq!(target.pointer("/tags/0").unwrap().as_str(), Some("rust"));
    }
}
