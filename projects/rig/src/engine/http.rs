//! HTTP step execution: one request, expectation checking, jsonpath-lite
//! capture.
//!
//! jsonpath-lite is a hand-rolled dot-path subset (`$.a.b[0].c`) — enough
//! for capture/assert without an expression-language dependency. Response
//! meta is addressable as `status` and `latency_ms`.

use std::time::{Duration, Instant};

use serde_json::Value;

use crate::scenario::interp::VarStore;
use crate::scenario::step::HttpRequest;

/// One completed exchange.
#[derive(Debug)]
pub struct HttpOutcome {
    pub status: u16,
    pub latency_ms: f64,
    pub body: Option<Value>,
    /// None = all expectations met; Some(reason) = first violation.
    pub violation: Option<String>,
}

/// Execute one request template against the var store. Network-level
/// failures (connect refused, timeout) come back as a violation with
/// status 0 — chaos scenarios assert on exactly these.
pub fn execute(request: &HttpRequest, vars: &VarStore) -> Result<HttpOutcome, String> {
    let url = vars.interpolate(&request.url)?;
    let body = match &request.body {
        Some(b) => Some(vars.interpolate(b)?),
        None => None,
    };
    let timeout = Duration::from_millis(request.expect.timeout_ms);

    let agent = ureq::AgentBuilder::new()
        .timeout_connect(timeout)
        .timeout(timeout)
        .build();

    let method = request.method.to_uppercase();
    let started = Instant::now();
    let result = match body {
        Some(ref payload) => agent
            .request(&method, &url)
            .set("content-type", "application/json")
            .send_string(payload),
        None => agent.request(&method, &url).call(),
    };
    let latency_ms = started.elapsed().as_secs_f64() * 1000.0;

    let (status, body_text) = match result {
        Ok(resp) => {
            let status = resp.status();
            let text = resp.into_string().unwrap_or_default();
            (status, text)
        }
        // ureq::Error::Status carries the response for non-2xx codes.
        Err(ureq::Error::Status(status, resp)) => {
            let text = resp.into_string().unwrap_or_default();
            (status, text)
        }
        Err(e) => {
            return Ok(HttpOutcome {
                status: 0,
                latency_ms,
                body: None,
                violation: Some(format!("transport error: {e}")),
            });
        }
    };

    let body_json: Option<Value> = serde_json::from_str(&body_text).ok();

    let mut violation = None;
    if !request.expect.status_ok(status) {
        let expected = if request.expect.statuses.is_empty() {
            request.expect.status.to_string()
        } else {
            format!("{:?}", request.expect.statuses)
        };
        violation = Some(format!("status {status} != expected {expected}"));
    } else {
        for (path, predicate) in &request.expect.jsonpath {
            let actual = body_json.as_ref().and_then(|b| json_path(b, path));
            match check_predicate(actual.as_ref(), predicate) {
                Ok(true) => {}
                Ok(false) => {
                    violation = Some(format!(
                        "jsonpath `{path}` = {} violates `{predicate}`",
                        actual.map(|v| v.to_string()).unwrap_or_else(|| "<missing>".into())
                    ));
                    break;
                }
                Err(e) => {
                    violation = Some(format!("jsonpath `{path}`: {e}"));
                    break;
                }
            }
        }
    }

    Ok(HttpOutcome {
        status,
        latency_ms,
        body: body_json,
        violation,
    })
}

/// Resolve a capture key against an outcome: `status`, `latency_ms`, or a
/// `$.dot.path[i]` into the response body.
pub fn capture_value(outcome: &HttpOutcome, key: &str) -> Option<Value> {
    match key {
        "status" => Some(Value::from(outcome.status)),
        "latency_ms" => serde_json::Number::from_f64(outcome.latency_ms).map(Value::Number),
        path if path.starts_with('$') => outcome.body.as_ref().and_then(|b| json_path(b, path)),
        _ => None,
    }
}

/// Dot-path subset: `$.a.b[0].c`. Returns a clone of the addressed value.
pub fn json_path(root: &Value, path: &str) -> Option<Value> {
    let mut current = root;
    let trimmed = path.strip_prefix('$')?;
    for segment in trimmed.split('.').filter(|s| !s.is_empty()) {
        // Each segment is `name` optionally followed by one or more `[idx]`.
        let (name, indices) = match segment.find('[') {
            Some(b) => (&segment[..b], &segment[b..]),
            None => (segment, ""),
        };
        if !name.is_empty() {
            current = current.get(name)?;
        }
        let mut rest = indices;
        while let Some(open) = rest.find('[') {
            let close = rest.find(']')?;
            let idx: usize = rest[open + 1..close].parse().ok()?;
            current = current.get(idx)?;
            rest = &rest[close + 1..];
        }
    }
    Some(current.clone())
}

/// Predicate over an optional JSON value: `>= 1`, `== "ok"`, `exists`.
fn check_predicate(actual: Option<&Value>, predicate: &str) -> Result<bool, String> {
    let p = predicate.trim();
    if p == "exists" {
        return Ok(actual.is_some());
    }
    let Some(actual) = actual else {
        return Ok(false);
    };
    let (op, rhs) = ["<=", ">=", "==", "!=", "<", ">"]
        .iter()
        .find_map(|op| p.strip_prefix(op).map(|rest| (*op, rest.trim())))
        .ok_or_else(|| format!("unsupported predicate `{p}` (ops: == != < <= > >= exists)"))?;

    // String comparison when the rhs is quoted.
    if let Some(want) = rhs.strip_prefix('"').and_then(|r| r.strip_suffix('"')) {
        let got = actual.as_str().unwrap_or_default();
        return Ok(match op {
            "==" => got == want,
            "!=" => got != want,
            _ => return Err(format!("op `{op}` not valid for strings")),
        });
    }

    let want: f64 = rhs
        .parse()
        .map_err(|_| format!("non-numeric rhs in predicate `{p}`"))?;
    let got = match actual {
        Value::Number(n) => n.as_f64().unwrap_or(f64::NAN),
        Value::String(s) => s.parse().unwrap_or(f64::NAN),
        Value::Bool(b) => {
            if *b {
                1.0
            } else {
                0.0
            }
        }
        _ => return Ok(false),
    };
    Ok(match op {
        "==" => got == want,
        "!=" => got != want,
        "<" => got < want,
        "<=" => got <= want,
        ">" => got > want,
        ">=" => got >= want,
        _ => unreachable!(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn json_path_walks_objects_and_arrays() {
        let v = json!({"hits": [{"id": "a", "score": 1.5}], "total": 7});
        assert_eq!(json_path(&v, "$.total"), Some(json!(7)));
        assert_eq!(json_path(&v, "$.hits[0].id"), Some(json!("a")));
        assert_eq!(json_path(&v, "$.hits[1]"), None);
        assert_eq!(json_path(&v, "$.missing"), None);
    }

    #[test]
    fn predicates_compare_numbers_and_strings() {
        assert!(check_predicate(Some(&json!(7)), ">= 1").unwrap());
        assert!(!check_predicate(Some(&json!(0)), ">= 1").unwrap());
        assert!(check_predicate(Some(&json!("ok")), "== \"ok\"").unwrap());
        assert!(check_predicate(Some(&json!(1)), "exists").unwrap());
        assert!(!check_predicate(None, ">= 1").unwrap());
        assert!(check_predicate(Some(&json!(1)), "~ 2").is_err());
    }
}
