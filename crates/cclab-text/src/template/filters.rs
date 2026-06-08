//! Built-in template filters.
//!
//! Provides Jinja2-compatible filters for text transformation.

use super::engine::Value;

/// Apply a named filter to a value with optional arguments.
pub fn apply_filter(name: &str, value: &Value, args: &[Value]) -> Result<Value, String> {
    match name {
        "upper" => filter_upper(value),
        "lower" => filter_lower(value),
        "capitalize" => filter_capitalize(value),
        "title" => filter_title(value),
        "trim" => filter_trim(value),
        "strip" => filter_trim(value), // alias
        "length" => filter_length(value),
        "reverse" => filter_reverse(value),
        "first" => filter_first(value),
        "last" => filter_last(value),
        "join" => filter_join(value, args),
        "replace" => filter_replace(value, args),
        "default" => filter_default(value, args),
        "d" => filter_default(value, args), // alias
        "int" => filter_int(value),
        "float" => filter_float(value),
        "string" => filter_string(value),
        "abs" => filter_abs(value),
        "round" => filter_round(value, args),
        "truncate" => filter_truncate(value, args),
        "wordcount" => filter_wordcount(value),
        "sort" => filter_sort(value),
        "unique" => filter_unique(value),
        "batch" => filter_batch(value, args),
        "escape" | "e" => filter_escape(value),
        "list" => filter_list(value),
        "max" => filter_max(value),
        "min" => filter_min(value),
        "sum" => filter_sum(value),
        "map" => filter_map(value, args),
        "select" => Ok(value.clone()), // passthrough for now
        "reject" => Ok(value.clone()), // passthrough for now
        _ => Err(format!("Unknown filter: {}", name)),
    }
}

fn value_to_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::None => "None".to_string(),
        Value::List(items) => {
            let parts: Vec<String> = items.iter().map(value_to_string).collect();
            format!("[{}]", parts.join(", "))
        }
        Value::Map(map) => {
            let parts: Vec<String> = map
                .iter()
                .map(|(k, v)| format!("{}: {}", k, value_to_string(v)))
                .collect();
            format!("{{{}}}", parts.join(", "))
        }
    }
}

fn filter_upper(value: &Value) -> Result<Value, String> {
    Ok(Value::String(value_to_string(value).to_uppercase()))
}

fn filter_lower(value: &Value) -> Result<Value, String> {
    Ok(Value::String(value_to_string(value).to_lowercase()))
}

fn filter_capitalize(value: &Value) -> Result<Value, String> {
    let s = value_to_string(value);
    let mut chars = s.chars();
    let result = match chars.next() {
        None => String::new(),
        Some(first) => {
            let upper: String = first.to_uppercase().collect();
            let rest: String = chars.collect();
            format!("{}{}", upper, rest.to_lowercase())
        }
    };
    Ok(Value::String(result))
}

fn filter_title(value: &Value) -> Result<Value, String> {
    let s = value_to_string(value);
    let result = s
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    let upper: String = first.to_uppercase().collect();
                    let rest: String = chars.collect();
                    format!("{}{}", upper, rest.to_lowercase())
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    Ok(Value::String(result))
}

fn filter_trim(value: &Value) -> Result<Value, String> {
    Ok(Value::String(value_to_string(value).trim().to_string()))
}

fn filter_length(value: &Value) -> Result<Value, String> {
    match value {
        Value::String(s) => Ok(Value::Int(s.len() as i64)),
        Value::List(items) => Ok(Value::Int(items.len() as i64)),
        Value::Map(map) => Ok(Value::Int(map.len() as i64)),
        _ => Ok(Value::Int(value_to_string(value).len() as i64)),
    }
}

fn filter_reverse(value: &Value) -> Result<Value, String> {
    match value {
        Value::String(s) => Ok(Value::String(s.chars().rev().collect())),
        Value::List(items) => {
            let mut reversed = items.clone();
            reversed.reverse();
            Ok(Value::List(reversed))
        }
        _ => Ok(Value::String(
            value_to_string(value).chars().rev().collect(),
        )),
    }
}

fn filter_first(value: &Value) -> Result<Value, String> {
    match value {
        Value::List(items) => Ok(items.first().cloned().unwrap_or(Value::None)),
        Value::String(s) => Ok(s
            .chars()
            .next()
            .map(|c| Value::String(c.to_string()))
            .unwrap_or(Value::None)),
        _ => Err("first filter requires a list or string".to_string()),
    }
}

fn filter_last(value: &Value) -> Result<Value, String> {
    match value {
        Value::List(items) => Ok(items.last().cloned().unwrap_or(Value::None)),
        Value::String(s) => Ok(s
            .chars()
            .last()
            .map(|c| Value::String(c.to_string()))
            .unwrap_or(Value::None)),
        _ => Err("last filter requires a list or string".to_string()),
    }
}

fn filter_join(value: &Value, args: &[Value]) -> Result<Value, String> {
    let separator = args.first().map(value_to_string).unwrap_or_default();

    match value {
        Value::List(items) => {
            let parts: Vec<String> = items.iter().map(value_to_string).collect();
            Ok(Value::String(parts.join(&separator)))
        }
        _ => Ok(value.clone()),
    }
}

fn filter_replace(value: &Value, args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("replace filter requires 2 arguments (old, new)".to_string());
    }
    let old = value_to_string(&args[0]);
    let new = value_to_string(&args[1]);
    let s = value_to_string(value);
    Ok(Value::String(s.replace(&old, &new)))
}

fn filter_default(value: &Value, args: &[Value]) -> Result<Value, String> {
    match value {
        Value::None => Ok(args
            .first()
            .cloned()
            .unwrap_or(Value::String(String::new()))),
        Value::String(s) if s.is_empty() => Ok(args
            .first()
            .cloned()
            .unwrap_or(Value::String(String::new()))),
        _ => Ok(value.clone()),
    }
}

fn filter_int(value: &Value) -> Result<Value, String> {
    match value {
        Value::Int(_) => Ok(value.clone()),
        Value::Float(f) => Ok(Value::Int(*f as i64)),
        Value::String(s) => {
            let i: i64 = s.parse().unwrap_or(0);
            Ok(Value::Int(i))
        }
        Value::Bool(b) => Ok(Value::Int(if *b { 1 } else { 0 })),
        _ => Ok(Value::Int(0)),
    }
}

fn filter_float(value: &Value) -> Result<Value, String> {
    match value {
        Value::Float(_) => Ok(value.clone()),
        Value::Int(i) => Ok(Value::Float(*i as f64)),
        Value::String(s) => {
            let f: f64 = s.parse().unwrap_or(0.0);
            Ok(Value::Float(f))
        }
        _ => Ok(Value::Float(0.0)),
    }
}

fn filter_string(value: &Value) -> Result<Value, String> {
    Ok(Value::String(value_to_string(value)))
}

fn filter_abs(value: &Value) -> Result<Value, String> {
    match value {
        Value::Int(i) => Ok(Value::Int(i.abs())),
        Value::Float(f) => Ok(Value::Float(f.abs())),
        _ => Err("abs filter requires a number".to_string()),
    }
}

fn filter_round(value: &Value, args: &[Value]) -> Result<Value, String> {
    let precision = match args.first() {
        Some(Value::Int(n)) => *n as i32,
        _ => 0,
    };
    match value {
        Value::Float(f) => {
            let factor = 10f64.powi(precision);
            Ok(Value::Float((f * factor).round() / factor))
        }
        Value::Int(i) => Ok(Value::Int(*i)),
        _ => Err("round filter requires a number".to_string()),
    }
}

fn filter_truncate(value: &Value, args: &[Value]) -> Result<Value, String> {
    let length = match args.first() {
        Some(Value::Int(n)) => *n as usize,
        _ => 255,
    };
    let end = match args.get(1) {
        Some(Value::String(s)) => s.as_str(),
        _ => "...",
    };
    let s = value_to_string(value);
    if s.len() <= length {
        Ok(Value::String(s))
    } else {
        let truncated: String = s.chars().take(length.saturating_sub(end.len())).collect();
        Ok(Value::String(format!("{}{}", truncated, end)))
    }
}

fn filter_wordcount(value: &Value) -> Result<Value, String> {
    let s = value_to_string(value);
    Ok(Value::Int(s.split_whitespace().count() as i64))
}

fn filter_sort(value: &Value) -> Result<Value, String> {
    match value {
        Value::List(items) => {
            let mut sorted = items.clone();
            sorted.sort_by(|a, b| value_to_string(a).cmp(&value_to_string(b)));
            Ok(Value::List(sorted))
        }
        _ => Err("sort filter requires a list".to_string()),
    }
}

fn filter_unique(value: &Value) -> Result<Value, String> {
    match value {
        Value::List(items) => {
            let mut seen = std::collections::HashSet::new();
            let unique: Vec<Value> = items
                .iter()
                .filter(|item| seen.insert(value_to_string(item)))
                .cloned()
                .collect();
            Ok(Value::List(unique))
        }
        _ => Err("unique filter requires a list".to_string()),
    }
}

fn filter_batch(value: &Value, args: &[Value]) -> Result<Value, String> {
    let size = match args.first() {
        Some(Value::Int(n)) => *n as usize,
        _ => return Err("batch filter requires a size argument".to_string()),
    };
    if size == 0 {
        return Err("batch size must be > 0".to_string());
    }

    match value {
        Value::List(items) => {
            let batches: Vec<Value> = items
                .chunks(size)
                .map(|chunk| Value::List(chunk.to_vec()))
                .collect();
            Ok(Value::List(batches))
        }
        _ => Err("batch filter requires a list".to_string()),
    }
}

fn filter_escape(value: &Value) -> Result<Value, String> {
    let s = value_to_string(value);
    let escaped = s
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;");
    Ok(Value::String(escaped))
}

fn filter_list(value: &Value) -> Result<Value, String> {
    match value {
        Value::String(s) => {
            let chars: Vec<Value> = s.chars().map(|c| Value::String(c.to_string())).collect();
            Ok(Value::List(chars))
        }
        Value::List(_) => Ok(value.clone()),
        _ => Ok(Value::List(vec![value.clone()])),
    }
}

fn filter_max(value: &Value) -> Result<Value, String> {
    match value {
        Value::List(items) => {
            let max = items.iter().max_by(|a, b| compare_values(a, b));
            Ok(max.cloned().unwrap_or(Value::None))
        }
        _ => Err("max filter requires a list".to_string()),
    }
}

fn filter_min(value: &Value) -> Result<Value, String> {
    match value {
        Value::List(items) => {
            let min = items.iter().min_by(|a, b| compare_values(a, b));
            Ok(min.cloned().unwrap_or(Value::None))
        }
        _ => Err("min filter requires a list".to_string()),
    }
}

fn filter_sum(value: &Value) -> Result<Value, String> {
    match value {
        Value::List(items) => {
            let mut sum = 0.0f64;
            let mut is_int = true;
            for item in items {
                match item {
                    Value::Int(i) => sum += *i as f64,
                    Value::Float(f) => {
                        sum += f;
                        is_int = false;
                    }
                    _ => {}
                }
            }
            if is_int {
                Ok(Value::Int(sum as i64))
            } else {
                Ok(Value::Float(sum))
            }
        }
        _ => Err("sum filter requires a list".to_string()),
    }
}

fn filter_map(value: &Value, args: &[Value]) -> Result<Value, String> {
    let attr = match args.first() {
        Some(Value::String(s)) => s.clone(),
        _ => return Err("map filter requires an attribute name".to_string()),
    };

    match value {
        Value::List(items) => {
            let mapped: Vec<Value> = items
                .iter()
                .map(|item| match item {
                    Value::Map(map) => map.get(&attr).cloned().unwrap_or(Value::None),
                    _ => Value::None,
                })
                .collect();
            Ok(Value::List(mapped))
        }
        _ => Err("map filter requires a list".to_string()),
    }
}

fn compare_values(a: &Value, b: &Value) -> std::cmp::Ordering {
    match (a, b) {
        (Value::Int(a), Value::Int(b)) => a.cmp(b),
        (Value::Float(a), Value::Float(b)) => a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal),
        (Value::Int(a), Value::Float(b)) => (*a as f64)
            .partial_cmp(b)
            .unwrap_or(std::cmp::Ordering::Equal),
        (Value::Float(a), Value::Int(b)) => a
            .partial_cmp(&(*b as f64))
            .unwrap_or(std::cmp::Ordering::Equal),
        _ => value_to_string(a).cmp(&value_to_string(b)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upper() {
        let result = apply_filter("upper", &Value::String("hello".into()), &[]).unwrap();
        assert_eq!(result, Value::String("HELLO".into()));
    }

    #[test]
    fn test_lower() {
        let result = apply_filter("lower", &Value::String("HELLO".into()), &[]).unwrap();
        assert_eq!(result, Value::String("hello".into()));
    }

    #[test]
    fn test_capitalize() {
        let result = apply_filter("capitalize", &Value::String("hello world".into()), &[]).unwrap();
        assert_eq!(result, Value::String("Hello world".into()));
    }

    #[test]
    fn test_title() {
        let result = apply_filter("title", &Value::String("hello world".into()), &[]).unwrap();
        assert_eq!(result, Value::String("Hello World".into()));
    }

    #[test]
    fn test_length() {
        let result = apply_filter("length", &Value::String("hello".into()), &[]).unwrap();
        assert_eq!(result, Value::Int(5));
    }

    #[test]
    fn test_join() {
        let list = Value::List(vec![
            Value::String("a".into()),
            Value::String("b".into()),
            Value::String("c".into()),
        ]);
        let result = apply_filter("join", &list, &[Value::String(", ".into())]).unwrap();
        assert_eq!(result, Value::String("a, b, c".into()));
    }

    #[test]
    fn test_default() {
        let result =
            apply_filter("default", &Value::None, &[Value::String("fallback".into())]).unwrap();
        assert_eq!(result, Value::String("fallback".into()));
    }

    #[test]
    fn test_escape() {
        let result = apply_filter("escape", &Value::String("<b>hi</b>".into()), &[]).unwrap();
        assert_eq!(result, Value::String("&lt;b&gt;hi&lt;/b&gt;".into()));
    }
}
