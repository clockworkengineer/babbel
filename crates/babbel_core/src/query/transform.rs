//! Polyglot Query & Transformation Pipeline Engine (`jq` / `yq` style operators over [`Value`]).
//!
//! Supports piping (`|`) JSONPath selectors and transformation/aggregation functions:
//! - Navigation: any RFC 9535 JSONPath expression (`$.store.books[*]`, `.price`)
//! - Object & Array inspection: `keys`, `values`, `length`, `count`, `type`
//! - Array transforms: `flatten`, `unique`, `reverse`, `sort`, `sort_by(.field)`, `group_by(.field)`
//! - Higher-order transforms: `map(expr)`, `select(filter)`, `pick(k1, k2)`, `omit(k1, k2)`
//! - Aggregations: `sum`, `min`, `max`, `avg`, `first`, `last`

use crate::error::BabbelError;
use crate::model::Value;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::cmp::Ordering;

/// Evaluates a transformation pipeline expression (`stage1 | stage2 | ...`) against `input`.
pub fn transform(input: &Value, expr: &str) -> Result<Value, BabbelError> {
    let stages = split_pipeline_stages(expr);
    let mut current = input.clone();
    for raw_stage in stages {
        let stage = raw_stage.trim();
        if stage.is_empty() || stage == "." || stage == "$" {
            continue;
        }
        current = eval_stage(&current, stage)?;
    }
    Ok(current)
}

fn split_pipeline_stages(expr: &str) -> Vec<&str> {
    let mut stages = Vec::new();
    let mut start = 0;
    let mut depth_paren = 0usize;
    let mut depth_bracket = 0usize;
    let mut in_quote: Option<char> = None;
    let mut escaped = false;

    for (idx, c) in expr.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if let Some(q) = in_quote {
            if c == '\\' {
                escaped = true;
            } else if c == q {
                in_quote = None;
            }
            continue;
        }
        match c {
            '\'' | '"' => in_quote = Some(c),
            '(' => depth_paren += 1,
            ')' => depth_paren = depth_paren.saturating_sub(1),
            '[' => depth_bracket += 1,
            ']' => depth_bracket = depth_bracket.saturating_sub(1),
            '|' if depth_paren == 0 && depth_bracket == 0 => {
                // Make sure this is not `||` inside a filter
                let rest = &expr[idx..];
                if !rest.starts_with("||") && (idx == 0 || !expr[..idx].ends_with('|')) {
                    stages.push(&expr[start..idx]);
                    start = idx + 1;
                }
            }
            _ => {}
        }
    }
    stages.push(&expr[start..]);
    stages
}

fn eval_stage(current: &Value, stage: &str) -> Result<Value, BabbelError> {
    match stage {
        "keys" => match current {
            Value::Object(entries) => {
                let mut keys: Vec<String> = entries.iter().map(|(k, _)| k.clone()).collect();
                keys.sort();
                Ok(Value::Array(keys.into_iter().map(Value::String).collect()))
            }
            Value::Array(items) => Ok(Value::Array(
                (0..items.len())
                    .map(|i| Value::Integer(i as i128))
                    .collect(),
            )),
            _ => Err(BabbelError::syntax("'keys' requires an object or array")),
        },
        "values" => match current {
            Value::Object(entries) => Ok(Value::Array(
                entries.iter().map(|(_, v)| v.clone()).collect(),
            )),
            Value::Array(items) => Ok(Value::Array(items.clone())),
            _ => Err(BabbelError::syntax("'values' requires an object or array")),
        },
        "length" | "count" => match current {
            Value::Null => Ok(Value::Integer(0)),
            Value::String(s) => Ok(Value::Integer(s.chars().count() as i128)),
            Value::Bytes(b) => Ok(Value::Integer(b.len() as i128)),
            Value::Array(a) => Ok(Value::Integer(a.len() as i128)),
            Value::Object(o) => Ok(Value::Integer(o.len() as i128)),
            _ => Err(BabbelError::syntax(
                "'length' requires string, array, object, bytes, or null",
            )),
        },
        "type" => {
            let t = match current {
                Value::Null => "null",
                Value::Bool(_) => "boolean",
                Value::Integer(_) | Value::Float(_) => "number",
                Value::String(_) => "string",
                Value::Bytes(_) => "bytes",
                Value::Array(_) => "array",
                Value::Object(_) => "object",
            };
            Ok(Value::String(t.to_string()))
        }
        "flatten" => match current {
            Value::Array(items) => {
                let mut out = Vec::new();
                flatten_recursive(items, &mut out);
                Ok(Value::Array(out))
            }
            _ => Err(BabbelError::syntax("'flatten' requires an array")),
        },
        "unique" => match current {
            Value::Array(items) => {
                let mut out = Vec::new();
                for item in items {
                    if !out.contains(item) {
                        out.push(item.clone());
                    }
                }
                Ok(Value::Array(out))
            }
            _ => Err(BabbelError::syntax("'unique' requires an array")),
        },
        "reverse" => match current {
            Value::Array(items) => {
                let mut out = items.clone();
                out.reverse();
                Ok(Value::Array(out))
            }
            Value::String(s) => Ok(Value::String(s.chars().rev().collect())),
            _ => Err(BabbelError::syntax("'reverse' requires an array or string")),
        },
        "sort" => match current {
            Value::Array(items) => {
                let mut out = items.clone();
                out.sort_by(compare_value_order);
                Ok(Value::Array(out))
            }
            _ => Err(BabbelError::syntax("'sort' requires an array")),
        },
        "sum" => match current {
            Value::Array(items) => {
                let mut int_sum: i128 = 0;
                let mut float_sum: f64 = 0.0;
                let mut has_float = false;
                for item in items {
                    match item {
                        Value::Integer(i) => int_sum = int_sum.saturating_add(*i),
                        Value::Float(f) => {
                            has_float = true;
                            float_sum += *f;
                        }
                        _ => {}
                    }
                }
                if has_float {
                    Ok(Value::Float(float_sum + int_sum as f64))
                } else {
                    Ok(Value::Integer(int_sum))
                }
            }
            _ => Err(BabbelError::syntax("'sum' requires an array")),
        },
        "avg" => match current {
            Value::Array(items) => {
                if items.is_empty() {
                    return Ok(Value::Null);
                }
                let mut sum = 0.0f64;
                let mut count = 0usize;
                for item in items {
                    if let Some(f) = item.as_f64() {
                        sum += f;
                        count += 1;
                    }
                }
                if count == 0 {
                    Ok(Value::Null)
                } else {
                    Ok(Value::Float(sum / (count as f64)))
                }
            }
            _ => Err(BabbelError::syntax("'avg' requires an array")),
        },
        "min" => match current {
            Value::Array(items) => Ok(items
                .iter()
                .min_by(|a, b| compare_value_order(a, b))
                .cloned()
                .unwrap_or(Value::Null)),
            _ => Err(BabbelError::syntax("'min' requires an array")),
        },
        "max" => match current {
            Value::Array(items) => Ok(items
                .iter()
                .max_by(|a, b| compare_value_order(a, b))
                .cloned()
                .unwrap_or(Value::Null)),
            _ => Err(BabbelError::syntax("'max' requires an array")),
        },
        "first" => match current {
            Value::Array(items) => Ok(items.first().cloned().unwrap_or(Value::Null)),
            _ => Err(BabbelError::syntax("'first' requires an array")),
        },
        "last" => match current {
            Value::Array(items) => Ok(items.last().cloned().unwrap_or(Value::Null)),
            _ => Err(BabbelError::syntax("'last' requires an array")),
        },
        _ => {
            if let Some(arg) = extract_func_arg(stage, "sort_by") {
                match current {
                    Value::Array(items) => {
                        let mut keyed: Vec<(Value, Value)> = Vec::with_capacity(items.len());
                        for item in items {
                            let key = transform(item, arg).unwrap_or(Value::Null);
                            keyed.push((key, item.clone()));
                        }
                        keyed.sort_by(|(ka, _), (kb, _)| compare_value_order(ka, kb));
                        Ok(Value::Array(keyed.into_iter().map(|(_, v)| v).collect()))
                    }
                    _ => Err(BabbelError::syntax("'sort_by' requires an array")),
                }
            } else if let Some(arg) = extract_func_arg(stage, "group_by") {
                match current {
                    Value::Array(items) => {
                        let mut groups: Vec<(String, Vec<Value>)> = Vec::new();
                        for item in items {
                            let key_val = transform(item, arg).unwrap_or(Value::Null);
                            let key_str = match key_val {
                                Value::String(s) => s,
                                Value::Integer(i) => format!("{}", i),
                                Value::Float(f) => format!("{}", f),
                                Value::Bool(b) => format!("{}", b),
                                Value::Null => "null".to_string(),
                                other => {
                                    let mut buf = crate::io::Buffer::new();
                                    other.serialize_json(&mut buf);
                                    buf.to_string()
                                }
                            };
                            if let Some((_, bucket)) =
                                groups.iter_mut().find(|(k, _)| k == &key_str)
                            {
                                bucket.push(item.clone());
                            } else {
                                groups.push((key_str, alloc::vec![item.clone()]));
                            }
                        }
                        Ok(Value::Object(
                            groups
                                .into_iter()
                                .map(|(k, v)| (k, Value::Array(v)))
                                .collect(),
                        ))
                    }
                    _ => Err(BabbelError::syntax("'group_by' requires an array")),
                }
            } else if let Some(arg) = extract_func_arg(stage, "map") {
                match current {
                    Value::Array(items) => {
                        let mut out = Vec::with_capacity(items.len());
                        for item in items {
                            out.push(transform(item, arg)?);
                        }
                        Ok(Value::Array(out))
                    }
                    _ => Err(BabbelError::syntax("'map' requires an array")),
                }
            } else if let Some(arg) = extract_func_arg(stage, "select") {
                let cond = normalize_select_condition(arg);
                let filter_path = format!("$[?({})]", cond);
                match current {
                    Value::Array(_) => {
                        let matched = super::query(current, &filter_path)?;
                        Ok(Value::Array(matched.into_iter().cloned().collect()))
                    }
                    _ => {
                        let wrapped = Value::Array(alloc::vec![current.clone()]);
                        let matched = super::query(&wrapped, &filter_path)?;
                        Ok(matched.first().cloned().cloned().unwrap_or(Value::Null))
                    }
                }
            } else if let Some(arg) = extract_func_arg(stage, "pick") {
                let wanted: Vec<String> = arg
                    .split(',')
                    .map(|s| {
                        s.trim()
                            .trim_matches(|c| c == '\'' || c == '"' || c == '.')
                            .to_string()
                    })
                    .filter(|s| !s.is_empty())
                    .collect();
                match current {
                    Value::Object(entries) => {
                        let mut picked = Vec::new();
                        for w in &wanted {
                            if let Some((k, v)) = entries.iter().find(|(k, _)| k == w) {
                                picked.push((k.clone(), v.clone()));
                            }
                        }
                        Ok(Value::Object(picked))
                    }
                    Value::Array(items) => {
                        let mut out = Vec::with_capacity(items.len());
                        for item in items {
                            out.push(eval_stage(item, stage)?);
                        }
                        Ok(Value::Array(out))
                    }
                    _ => Err(BabbelError::syntax(
                        "'pick' requires an object or array of objects",
                    )),
                }
            } else if let Some(arg) = extract_func_arg(stage, "omit") {
                let excluded: Vec<String> = arg
                    .split(',')
                    .map(|s| {
                        s.trim()
                            .trim_matches(|c| c == '\'' || c == '"' || c == '.')
                            .to_string()
                    })
                    .filter(|s| !s.is_empty())
                    .collect();
                match current {
                    Value::Object(entries) => {
                        let kept = entries
                            .iter()
                            .filter(|(k, _)| !excluded.contains(k))
                            .cloned()
                            .collect();
                        Ok(Value::Object(kept))
                    }
                    Value::Array(items) => {
                        let mut out = Vec::with_capacity(items.len());
                        for item in items {
                            out.push(eval_stage(item, stage)?);
                        }
                        Ok(Value::Array(out))
                    }
                    _ => Err(BabbelError::syntax(
                        "'omit' requires an object or array of objects",
                    )),
                }
            } else {
                // Evaluate as a JSONPath expression
                let jp =
                    if stage.starts_with('$') || stage.starts_with('.') || stage.starts_with('[') {
                        stage.to_string()
                    } else {
                        format!("$.{}", stage)
                    };
                let path = super::JsonPath::parse(&jp)?;
                let matches = path.query(current);
                let produces_multi = path.ast().segments.iter().any(|s| {
                    matches!(
                        s,
                        super::Segment::Wildcard
                            | super::Segment::Slice { .. }
                            | super::Segment::Union(_)
                            | super::Segment::Descendant(_)
                            | super::Segment::Filter(_)
                    )
                });
                if produces_multi {
                    Ok(Value::Array(matches.into_iter().cloned().collect()))
                } else {
                    match matches.len() {
                        0 => Ok(Value::Null),
                        1 => Ok(matches[0].clone()),
                        _ => Ok(Value::Array(matches.into_iter().cloned().collect())),
                    }
                }
            }
        }
    }
}

fn extract_func_arg<'a>(stage: &'a str, func_name: &str) -> Option<&'a str> {
    let rest = stage.strip_prefix(func_name)?.trim_start();
    let inner = rest.strip_prefix('(')?.strip_suffix(')')?;
    Some(inner.trim())
}

fn normalize_select_condition(cond: &str) -> String {
    let trimmed = cond.trim();
    if trimmed.starts_with('@') || trimmed.starts_with('$') {
        trimmed.to_string()
    } else if trimmed.starts_with('.') {
        format!("@{}", trimmed)
    } else {
        trimmed.to_string()
    }
}

fn flatten_recursive(items: &[Value], out: &mut Vec<Value>) {
    for item in items {
        if let Value::Array(nested) = item {
            flatten_recursive(nested, out);
        } else {
            out.push(item.clone());
        }
    }
}

fn type_rank(v: &Value) -> u8 {
    match v {
        Value::Null => 0,
        Value::Bool(false) => 1,
        Value::Bool(true) => 2,
        Value::Integer(_) | Value::Float(_) => 3,
        Value::String(_) => 4,
        Value::Bytes(_) => 5,
        Value::Array(_) => 6,
        Value::Object(_) => 7,
    }
}

pub(crate) fn compare_value_order(a: &Value, b: &Value) -> Ordering {
    let ra = type_rank(a);
    let rb = type_rank(b);
    if ra != rb {
        return ra.cmp(&rb);
    }
    match (a, b) {
        (Value::Integer(ia), Value::Integer(ib)) => ia.cmp(ib),
        (Value::Float(fa), Value::Float(fb)) => fa.partial_cmp(fb).unwrap_or(Ordering::Equal),
        (Value::Integer(ia), Value::Float(fb)) => {
            (*ia as f64).partial_cmp(fb).unwrap_or(Ordering::Equal)
        }
        (Value::Float(fa), Value::Integer(ib)) => {
            fa.partial_cmp(&(*ib as f64)).unwrap_or(Ordering::Equal)
        }
        (Value::String(sa), Value::String(sb)) => sa.cmp(sb),
        (Value::Bytes(ba), Value::Bytes(bb)) => ba.cmp(bb),
        (Value::Array(aa), Value::Array(ab)) => {
            for (va, vb) in aa.iter().zip(ab.iter()) {
                let cmp = compare_value_order(va, vb);
                if cmp != Ordering::Equal {
                    return cmp;
                }
            }
            aa.len().cmp(&ab.len())
        }
        _ => Ordering::Equal,
    }
}
