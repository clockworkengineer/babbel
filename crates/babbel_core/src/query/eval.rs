use super::ast::{
    ComparisonOp, FilterExpr, FilterOperand, LocatedNode, PathQuery, QueryPath, Segment,
};
use crate::model::Value;
use alloc::format;
use alloc::string::String;
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

/// Evaluates a `QueryPath` on a root `Value`, collecting [`LocatedNode`]s with normalized paths and JSON pointers.
pub fn evaluate_located<'a>(path: &QueryPath, root: &'a Value) -> Vec<LocatedNode<'a>> {
    let mut current_nodes = alloc::vec![LocatedNode {
        path: String::from("$"),
        pointer: String::new(),
        value: root,
    }];

    for segment in &path.segments {
        let mut next_nodes = Vec::new();
        for node in current_nodes {
            eval_segment_located(segment, &node, root, &mut next_nodes);
        }
        current_nodes = next_nodes;
    }

    current_nodes
}

/// Evaluates a `QueryPath` on a root `Value`, collecting mutable references to all matched nodes.
pub fn evaluate_mut<'a>(path: &QueryPath, root: &'a mut Value) -> Vec<&'a mut Value> {
    let root_snapshot = root.clone();
    let mut current_nodes = alloc::vec![root];

    for segment in &path.segments {
        let mut next_nodes = Vec::new();
        for node in current_nodes {
            eval_segment_mut(segment, node, &root_snapshot, &mut next_nodes);
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
                let actual_idx = resolve_array_index(*idx, items.len());
                if let Some(item) = actual_idx.and_then(|i| items.get(i)) {
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
                for idx in compute_slice_indices(items.len(), *start, *end, *step) {
                    if let Some(item) = items.get(idx) {
                        output.push(item);
                    }
                }
            }
        }
        Segment::Union(selectors) => {
            for sel in selectors {
                eval_segment(sel, current, root, output);
            }
        }
        Segment::Descendant(inner) => {
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

fn eval_segment_located<'a>(
    segment: &Segment,
    current: &LocatedNode<'a>,
    root: &'a Value,
    output: &mut Vec<LocatedNode<'a>>,
) {
    match segment {
        Segment::Child(name) => {
            if let Some(val) = current.value.get(name) {
                output.push(LocatedNode {
                    path: format!("{}['{}']", current.path, escape_normalized_key(name)),
                    pointer: format!("{}/{}", current.pointer, escape_pointer_token(name)),
                    value: val,
                });
            }
        }
        Segment::Index(idx) => {
            if let Value::Array(items) = current.value {
                if let Some(actual_idx) = resolve_array_index(*idx, items.len()) {
                    if let Some(item) = items.get(actual_idx) {
                        output.push(LocatedNode {
                            path: format!("{}[{}]", current.path, actual_idx),
                            pointer: format!("{}/{}", current.pointer, actual_idx),
                            value: item,
                        });
                    }
                }
            }
        }
        Segment::Wildcard => match current.value {
            Value::Array(items) => {
                for (i, item) in items.iter().enumerate() {
                    output.push(LocatedNode {
                        path: format!("{}[{}]", current.path, i),
                        pointer: format!("{}/{}", current.pointer, i),
                        value: item,
                    });
                }
            }
            Value::Object(entries) => {
                for (key, val) in entries {
                    output.push(LocatedNode {
                        path: format!("{}['{}']", current.path, escape_normalized_key(key)),
                        pointer: format!("{}/{}", current.pointer, escape_pointer_token(key)),
                        value: val,
                    });
                }
            }
            _ => {}
        },
        Segment::Slice { start, end, step } => {
            if let Value::Array(items) = current.value {
                for idx in compute_slice_indices(items.len(), *start, *end, *step) {
                    if let Some(item) = items.get(idx) {
                        output.push(LocatedNode {
                            path: format!("{}[{}]", current.path, idx),
                            pointer: format!("{}/{}", current.pointer, idx),
                            value: item,
                        });
                    }
                }
            }
        }
        Segment::Union(selectors) => {
            for sel in selectors {
                eval_segment_located(sel, current, root, output);
            }
        }
        Segment::Descendant(inner) => {
            eval_segment_located(inner, current, root, output);
            traverse_descendants_located(current, &mut |desc| {
                eval_segment_located(inner, &desc, root, output);
            });
        }
        Segment::Filter(expr) => match current.value {
            Value::Array(items) => {
                for (i, item) in items.iter().enumerate() {
                    if eval_filter(expr, item, root) {
                        output.push(LocatedNode {
                            path: format!("{}[{}]", current.path, i),
                            pointer: format!("{}/{}", current.pointer, i),
                            value: item,
                        });
                    }
                }
            }
            Value::Object(entries) => {
                for (key, val) in entries {
                    if eval_filter(expr, val, root) {
                        output.push(LocatedNode {
                            path: format!("{}['{}']", current.path, escape_normalized_key(key)),
                            pointer: format!("{}/{}", current.pointer, escape_pointer_token(key)),
                            value: val,
                        });
                    }
                }
            }
            _ => {
                if eval_filter(expr, current.value, root) {
                    output.push(current.clone());
                }
            }
        },
    }
}

fn eval_segment_mut<'a>(
    segment: &Segment,
    current: &'a mut Value,
    root: &Value,
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
                if let Some(actual_idx) = resolve_array_index(*idx, items.len()) {
                    if let Some(item) = items.get_mut(actual_idx) {
                        output.push(item);
                    }
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
                let indices = compute_slice_indices(items.len(), *start, *end, *step);
                let base_ptr = items.as_mut_ptr();
                let len = items.len();
                for idx in indices {
                    if idx < len {
                        // SAFETY: `compute_slice_indices` with non-zero step produces strictly distinct indices `< len`.
                        unsafe {
                            output.push(&mut *base_ptr.add(idx));
                        }
                    }
                }
            }
        }
        Segment::Union(selectors) => {
            let ptr = current as *mut Value;
            for sel in selectors {
                // SAFETY: Evaluates union selector against current container.
                unsafe {
                    eval_segment_mut(sel, &mut *ptr, root, output);
                }
            }
        }
        Segment::Descendant(inner) => {
            let ptr = current as *mut Value;
            // SAFETY: Evaluates inner segment on current value and recurses into children.
            unsafe {
                eval_segment_mut(inner, &mut *ptr, root, output);
                traverse_descendants_mut(&mut *ptr, root, output, inner);
            }
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

fn resolve_array_index(idx: i64, len: usize) -> Option<usize> {
    if idx >= 0 {
        let u = idx as usize;
        if u < len { Some(u) } else { None }
    } else {
        let len_i = len as i64;
        if len_i + idx >= 0 {
            Some((len_i + idx) as usize)
        } else {
            None
        }
    }
}

fn compute_slice_indices(
    len_usize: usize,
    start: Option<i64>,
    end: Option<i64>,
    step: Option<i64>,
) -> Vec<usize> {
    let len = len_usize as i64;
    if len == 0 {
        return Vec::new();
    }
    let step_val = step.unwrap_or(1);
    if step_val == 0 {
        return Vec::new();
    }

    let (start_idx, end_idx) = if step_val > 0 {
        let s = start.map(|v| normalize_index(v, len)).unwrap_or(0);
        let e = end.map(|v| normalize_index(v, len)).unwrap_or(len);
        (s.clamp(0, len), e.clamp(0, len))
    } else {
        let s = start
            .map(|v| normalize_index(v, len).clamp(-1, len - 1))
            .unwrap_or(len - 1);
        let e = end
            .map(|v| normalize_index(v, len).clamp(-1, len - 1))
            .unwrap_or(-1);
        (s, e)
    };

    let mut indices = Vec::new();
    let mut idx = start_idx;
    if step_val > 0 {
        while idx < end_idx {
            if idx >= 0 && (idx as usize) < len_usize {
                indices.push(idx as usize);
            }
            idx += step_val;
        }
    } else {
        while idx > end_idx {
            if idx >= 0 && (idx as usize) < len_usize {
                indices.push(idx as usize);
            }
            idx += step_val;
        }
    }
    indices
}

fn escape_normalized_key(key: &str) -> String {
    let mut out = String::with_capacity(key.len());
    for c in key.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\'' => out.push_str("\\'"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out
}

fn escape_pointer_token(token: &str) -> String {
    if token.contains('~') || token.contains('/') {
        token.replace('~', "~0").replace('/', "~1")
    } else {
        String::from(token)
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

fn traverse_descendants_located<'a, F>(node: &LocatedNode<'a>, f: &mut F)
where
    F: FnMut(LocatedNode<'a>),
{
    match node.value {
        Value::Array(items) => {
            for (i, item) in items.iter().enumerate() {
                let child = LocatedNode {
                    path: format!("{}[{}]", node.path, i),
                    pointer: format!("{}/{}", node.pointer, i),
                    value: item,
                };
                f(child.clone());
                traverse_descendants_located(&child, f);
            }
        }
        Value::Object(entries) => {
            for (key, val) in entries {
                let child = LocatedNode {
                    path: format!("{}['{}']", node.path, escape_normalized_key(key)),
                    pointer: format!("{}/{}", node.pointer, escape_pointer_token(key)),
                    value: val,
                };
                f(child.clone());
                traverse_descendants_located(&child, f);
            }
        }
        _ => {}
    }
}

fn traverse_descendants_mut<'a>(
    node: &'a mut Value,
    root: &Value,
    output: &mut Vec<&'a mut Value>,
    segment: &Segment,
) {
    match node {
        Value::Array(items) => {
            for item in items {
                let ptr = item as *mut Value;
                // SAFETY: Array elements are mutually disjoint in memory.
                unsafe {
                    eval_segment_mut(segment, &mut *ptr, root, output);
                    traverse_descendants_mut(&mut *ptr, root, output, segment);
                }
            }
        }
        Value::Object(entries) => {
            for (_, val) in entries {
                let ptr = val as *mut Value;
                // SAFETY: Object entries are mutually disjoint in memory.
                unsafe {
                    eval_segment_mut(segment, &mut *ptr, root, output);
                    traverse_descendants_mut(&mut *ptr, root, output, segment);
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
        FilterOperand::FunctionCall { name, args } => match name.as_str() {
            "length" => {
                let inner = resolve_operand(args.first()?, current, root)?;
                match inner {
                    Value::String(s) => Some(Value::Integer(s.chars().count() as i128)),
                    Value::Array(a) => Some(Value::Integer(a.len() as i128)),
                    Value::Object(o) => Some(Value::Integer(o.len() as i128)),
                    Value::Bytes(b) => Some(Value::Integer(b.len() as i128)),
                    _ => None,
                }
            }
            "count" => {
                let arg0 = args.first()?;
                match arg0 {
                    FilterOperand::RelativePath(path) => {
                        let matches = eval_path_query(path, current, root);
                        if matches.len() == 1
                            && matches[0].is_array()
                            && !path.segments.last().is_some_and(|s| {
                                matches!(
                                    s,
                                    Segment::Wildcard
                                        | Segment::Slice { .. }
                                        | Segment::Filter(_)
                                        | Segment::Descendant(_)
                                )
                            })
                        {
                            Some(Value::Integer(matches[0].as_array().unwrap().len() as i128))
                        } else {
                            Some(Value::Integer(matches.len() as i128))
                        }
                    }
                    FilterOperand::RootPath(path) => {
                        let matches = eval_path_query(path, root, root);
                        Some(Value::Integer(matches.len() as i128))
                    }
                    _ => {
                        let inner = resolve_operand(arg0, current, root)?;
                        match inner {
                            Value::Array(a) => Some(Value::Integer(a.len() as i128)),
                            _ => Some(Value::Integer(1)),
                        }
                    }
                }
            }
            "value" => {
                let arg0 = args.first()?;
                match arg0 {
                    FilterOperand::RelativePath(path) => {
                        let matches = eval_path_query(path, current, root);
                        if matches.len() == 1 {
                            Some(matches[0].clone())
                        } else {
                            None
                        }
                    }
                    FilterOperand::RootPath(path) => {
                        let matches = eval_path_query(path, root, root);
                        if matches.len() == 1 {
                            Some(matches[0].clone())
                        } else {
                            None
                        }
                    }
                    _ => resolve_operand(arg0, current, root),
                }
            }
            "match" => {
                if args.len() != 2 {
                    return Some(Value::Bool(false));
                }
                let target = resolve_operand(&args[0], current, root);
                let pattern = resolve_operand(&args[1], current, root);
                match (target, pattern) {
                    (Some(Value::String(s)), Some(Value::String(pat))) => Some(Value::Bool(
                        crate::schema::regex::regex_is_full_match(&pat, &s),
                    )),
                    _ => Some(Value::Bool(false)),
                }
            }
            "search" => {
                if args.len() != 2 {
                    return Some(Value::Bool(false));
                }
                let target = resolve_operand(&args[0], current, root);
                let pattern = resolve_operand(&args[1], current, root);
                match (target, pattern) {
                    (Some(Value::String(s)), Some(Value::String(pat))) => {
                        Some(Value::Bool(crate::schema::regex::regex_is_match(&pat, &s)))
                    }
                    _ => Some(Value::Bool(false)),
                }
            }
            _ => None,
        },
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
        ComparisonOp::Equal => match (left, right) {
            (Value::Integer(l), Value::Float(r)) => (*l as f64) == *r,
            (Value::Float(l), Value::Integer(r)) => *l == (*r as f64),
            _ => left == right,
        },
        ComparisonOp::NotEqual => !compare_values(left, right, ComparisonOp::Equal),
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
