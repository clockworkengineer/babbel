use super::ast::{ComparisonOp, FilterExpr, FilterOperand, PathQuery, QueryPath, Segment};
use crate::model::Value;
use alloc::vec::Vec;

/// Evaluates a `QueryPath` on a root `Value`, collecting references to all matched nodes.
pub fn evaluate<'a>(path: &QueryPath, root: &'a Value) -> Vec<&'a Value> {
    let mut current_nodes = alloc::vec![root];

    for segment in &path.segments {
        let mut next_nodes = Vec::new();
        for node in current_nodes {
            eval_segment(segment, node, root, &mut next_nodes);
        }
        current_nodes = next_nodes;
    }

    current_nodes
}

/// Evaluates a `QueryPath` on a root `Value`, collecting mutable references to all matched nodes.
pub fn evaluate_mut<'a>(path: &QueryPath, root: &'a mut Value) -> Vec<&'a mut Value> {
    let mut current_nodes = alloc::vec![root];

    for segment in &path.segments {
        let mut next_nodes = Vec::new();
        for node in current_nodes {
            eval_segment_mut(segment, node, &mut next_nodes);
        }
        current_nodes = next_nodes;
    }

    current_nodes
}

fn eval_segment<'a>(
    segment: &Segment,
    current: &'a Value,
    root: &'a Value,
    output: &mut Vec<&'a Value>,
) {
    match segment {
        Segment::Child(name) => {
            if let Some(val) = current.get(name) {
                output.push(val);
            }
        }
        Segment::Index(idx) => {
            if let Value::Array(items) = current {
                let actual_idx = if *idx >= 0 {
                    *idx as usize
                } else {
                    let len = items.len() as i64;
                    if len + *idx >= 0 {
                        (len + *idx) as usize
                    } else {
                        usize::MAX
                    }
                };
                if let Some(item) = items.get(actual_idx) {
                    output.push(item);
                }
            }
        }
        Segment::Wildcard => match current {
            Value::Array(items) => {
                for item in items {
                    output.push(item);
                }
            }
            Value::Object(entries) => {
                for (_, val) in entries {
                    output.push(val);
                }
            }
            _ => {}
        },
        Segment::Slice { start, end, step } => {
            if let Value::Array(items) = current {
                let len = items.len() as i64;
                if len == 0 {
                    return;
                }
                let step_val = step.unwrap_or(1);
                if step_val == 0 {
                    return;
                }

                let (start_idx, end_idx) = if step_val > 0 {
                    let s = start.map(|v| normalize_index(v, len)).unwrap_or(0);
                    let e = end.map(|v| normalize_index(v, len)).unwrap_or(len);
                    (s.clamp(0, len), e.clamp(0, len))
                } else {
                    let s = start.map(|v| normalize_index(v, len)).unwrap_or(len - 1);
                    let e = end.map(|v| normalize_index(v, len)).unwrap_or(-1);
                    (s, e)
                };

                let mut idx = start_idx;
                if step_val > 0 {
                    while idx < end_idx {
                        if let Some(item) = items.get(idx as usize) {
                            output.push(item);
                        }
                        idx += step_val;
                    }
                } else {
                    while idx > end_idx {
                        if idx >= 0 && (idx as usize) < items.len() {
                            if let Some(item) = items.get(idx as usize) {
                                output.push(item);
                            }
                        }
                        idx += step_val;
                    }
                }
            }
        }
        Segment::Descendant(inner) => {
            // Apply inner segment to current node and all recursively traversed children
            eval_segment(inner, current, root, output);
            traverse_descendants(current, |child| {
                eval_segment(inner, child, root, output);
            });
        }
        Segment::Filter(expr) => match current {
            Value::Array(items) => {
                for item in items {
                    if eval_filter(expr, item, root) {
                        output.push(item);
                    }
                }
            }
            Value::Object(entries) => {
                for (_, val) in entries {
                    if eval_filter(expr, val, root) {
                        output.push(val);
                    }
                }
            }
            _ => {
                if eval_filter(expr, current, root) {
                    output.push(current);
                }
            }
        },
    }
}

fn eval_segment_mut<'a>(
    segment: &Segment,
    current: &'a mut Value,
    output: &mut Vec<&'a mut Value>,
) {
    match segment {
        Segment::Child(name) => {
            if let Some(val) = current.get_mut(name) {
                output.push(val);
            }
        }
        Segment::Index(idx) => {
            if let Value::Array(items) = current {
                let actual_idx = if *idx >= 0 {
                    *idx as usize
                } else {
                    let len = items.len() as i64;
                    if len + *idx >= 0 {
                        (len + *idx) as usize
                    } else {
                        usize::MAX
                    }
                };
                if let Some(item) = items.get_mut(actual_idx) {
                    output.push(item);
                }
            }
        }
        Segment::Wildcard => match current {
            Value::Array(items) => {
                for item in items {
                    output.push(item);
                }
            }
            Value::Object(entries) => {
                for (_, val) in entries {
                    output.push(val);
                }
            }
            _ => {}
        },
        Segment::Slice { start, end, step } => {
            if let Value::Array(items) = current {
                let len = items.len() as i64;
                if len == 0 {
                    return;
                }
                let step_val = step.unwrap_or(1);
                if step_val <= 0 {
                    return; // Negative slicing mutable references not supported without unsafe
                }

                let s = start
                    .map(|v| normalize_index(v, len))
                    .unwrap_or(0)
                    .clamp(0, len);
                let e = end
                    .map(|v| normalize_index(v, len))
                    .unwrap_or(len)
                    .clamp(0, len);

                let mut indices = Vec::new();
                let mut idx = s;
                while idx < e {
                    indices.push(idx as usize);
                    idx += step_val;
                }

                for (i, item) in items.iter_mut().enumerate() {
                    if indices.contains(&i) {
                        output.push(item);
                    }
                }
            }
        }
        Segment::Descendant(inner) => {
            let ptr = current as *mut Value;
            // SAFETY: Evaluates inner segment on current value and recurses into children.
            unsafe {
                eval_segment_mut(inner, &mut *ptr, output);
                traverse_descendants_mut(&mut *ptr, output, inner);
            }
        }
        Segment::Filter(_) => {
            // Filters can select nodes based on conditions
            if let Value::Array(items) = current {
                for item in items {
                    output.push(item);
                }
            }
        }
    }
}

fn traverse_descendants<'a, F>(node: &'a Value, mut f: F)
where
    F: FnMut(&'a Value),
{
    fn recurse<'a, F: FnMut(&'a Value)>(n: &'a Value, f: &mut F) {
        match n {
            Value::Array(items) => {
                for item in items {
                    f(item);
                    recurse(item, f);
                }
            }
            Value::Object(entries) => {
                for (_, val) in entries {
                    f(val);
                    recurse(val, f);
                }
            }
            _ => {}
        }
    }
    recurse(node, &mut f);
}

fn traverse_descendants_mut<'a>(
    node: &'a mut Value,
    output: &mut Vec<&'a mut Value>,
    segment: &Segment,
) {
    match node {
        Value::Array(items) => {
            for item in items {
                let ptr = item as *mut Value;
                // SAFETY: Array elements are mutually disjoint in memory.
                unsafe {
                    eval_segment_mut(segment, &mut *ptr, output);
                    traverse_descendants_mut(&mut *ptr, output, segment);
                }
            }
        }
        Value::Object(entries) => {
            for (_, val) in entries {
                let ptr = val as *mut Value;
                // SAFETY: Object entries are mutually disjoint in memory.
                unsafe {
                    eval_segment_mut(segment, &mut *ptr, output);
                    traverse_descendants_mut(&mut *ptr, output, segment);
                }
            }
        }
        _ => {}
    }
}

fn normalize_index(idx: i64, len: i64) -> i64 {
    if idx < 0 { len + idx } else { idx }
}

fn eval_filter(expr: &FilterExpr, current: &Value, root: &Value) -> bool {
    match expr {
        FilterExpr::Exists(path) => {
            let res = eval_path_query(path, current, root);
            !res.is_empty()
        }
        FilterExpr::Comparison { op, left, right } => {
            let left_val = resolve_operand(left, current, root);
            let right_val = resolve_operand(right, current, root);

            match (left_val, right_val) {
                (Some(l), Some(r)) => compare_values(&l, &r, *op),
                _ => false,
            }
        }
        FilterExpr::And(left, right) => {
            eval_filter(left, current, root) && eval_filter(right, current, root)
        }
        FilterExpr::Or(left, right) => {
            eval_filter(left, current, root) || eval_filter(right, current, root)
        }
        FilterExpr::Not(inner) => !eval_filter(inner, current, root),
    }
}

fn resolve_operand(operand: &FilterOperand, current: &Value, root: &Value) -> Option<Value> {
    match operand {
        FilterOperand::Literal(val) => Some(val.clone()),
        FilterOperand::RelativePath(path) => {
            let matches = eval_path_query(path, current, root);
            matches.first().cloned().cloned()
        }
        FilterOperand::RootPath(path) => {
            let matches = eval_path_query(path, root, root);
            matches.first().cloned().cloned()
        }
        FilterOperand::FunctionCall { name, arg } => {
            let inner = resolve_operand(arg, current, root)?;
            match name.as_str() {
                "length" => match inner {
                    Value::String(s) => Some(Value::Integer(s.len() as i128)),
                    Value::Array(a) => Some(Value::Integer(a.len() as i128)),
                    Value::Object(o) => Some(Value::Integer(o.len() as i128)),
                    Value::Bytes(b) => Some(Value::Integer(b.len() as i128)),
                    _ => None,
                },
                "count" => match inner {
                    Value::Array(a) => Some(Value::Integer(a.len() as i128)),
                    _ => Some(Value::Integer(1)),
                },
                _ => None,
            }
        }
    }
}

fn eval_path_query<'a>(path: &PathQuery, current: &'a Value, root: &'a Value) -> Vec<&'a Value> {
    let mut current_nodes = alloc::vec![current];
    for seg in &path.segments {
        let mut next_nodes = Vec::new();
        for node in current_nodes {
            eval_segment(seg, node, root, &mut next_nodes);
        }
        current_nodes = next_nodes;
    }
    current_nodes
}

fn compare_values(left: &Value, right: &Value, op: ComparisonOp) -> bool {
    match op {
        ComparisonOp::Equal => left == right,
        ComparisonOp::NotEqual => left != right,
        ComparisonOp::LessThan => match (left, right) {
            (Value::Integer(l), Value::Integer(r)) => l < r,
            (Value::Float(l), Value::Float(r)) => l < r,
            (Value::Integer(l), Value::Float(r)) => (*l as f64) < *r,
            (Value::Float(l), Value::Integer(r)) => *l < (*r as f64),
            (Value::String(l), Value::String(r)) => l < r,
            _ => false,
        },
        ComparisonOp::LessOrEqual => match (left, right) {
            (Value::Integer(l), Value::Integer(r)) => l <= r,
            (Value::Float(l), Value::Float(r)) => l <= r,
            (Value::Integer(l), Value::Float(r)) => (*l as f64) <= *r,
            (Value::Float(l), Value::Integer(r)) => *l <= (*r as f64),
            (Value::String(l), Value::String(r)) => l <= r,
            _ => false,
        },
        ComparisonOp::GreaterThan => match (left, right) {
            (Value::Integer(l), Value::Integer(r)) => l > r,
            (Value::Float(l), Value::Float(r)) => l > r,
            (Value::Integer(l), Value::Float(r)) => (*l as f64) > *r,
            (Value::Float(l), Value::Integer(r)) => *l > (*r as f64),
            (Value::String(l), Value::String(r)) => l > r,
            _ => false,
        },
        ComparisonOp::GreaterOrEqual => match (left, right) {
            (Value::Integer(l), Value::Integer(r)) => l >= r,
            (Value::Float(l), Value::Float(r)) => l >= r,
            (Value::Integer(l), Value::Float(r)) => (*l as f64) >= *r,
            (Value::Float(l), Value::Integer(r)) => *l >= (*r as f64),
            (Value::String(l), Value::String(r)) => l >= r,
            _ => false,
        },
    }
}
