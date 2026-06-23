// SPEC-MANAGED: projects/vat/tech-design/semantic/vat-src.md#schema
// CODEGEN-BEGIN
//! The step interpreter for the built-in Cloud Workflows emulator.
//!
//! Parses a workflow source (YAML or JSON) and executes the supported step set —
//! assign, call (http.* / sys.log / named subworkflow), switch, for,
//! try/retry/except, next, return, raise — over a `serde_json` scope, using
//! [`expr`] for `${...}` and the shared [`dispatch`] for `call: http.*`. A step
//! budget guards against runaway loops; any error (including unsupported syntax)
//! becomes a workflow failure, never a panic.
//!
//! @spec projects/vat/tech-design/logic/built-in-cloud-workflows-emulator.md#logic

use std::collections::BTreeMap;
use std::time::Duration;

use serde_json::{json, Map, Value};

use super::expr::{self, Scope};
use crate::emulator::dispatch::{dispatch_collect, Oidc, Target};

const STEP_BUDGET: u32 = 10_000;

/// Outcome of running a list of steps.
enum Flow {
    /// Fell off the end of the steps.
    Fell,
    /// `return` produced this value.
    Return(Value),
}

/// Outcome of one step.
enum StepOut {
    Normal,
    Goto(String),
    Return(Value),
}

struct Interp<'a> {
    defs: Map<String, Value>,
    client: &'a reqwest::Client,
    budget: u32,
}

/// Run a workflow `source` with `argument`, returning the execution result.
pub async fn run(source: &str, argument: Value, client: &reqwest::Client) -> Result<Value, String> {
    let defs: Value =
        serde_yaml::from_str(source).map_err(|e| format!("parse workflow source: {e}"))?;
    let defs = defs
        .as_object()
        .ok_or("workflow source must be a map of definitions")?
        .clone();
    if !defs.contains_key("main") {
        return Err("workflow source has no `main` definition".to_string());
    }
    let mut interp = Interp {
        defs,
        client,
        budget: STEP_BUDGET,
    };
    interp
        .run_subworkflow("main", main_args(&interp.defs, argument))
        .await
}

/// Bind the execution argument to `main`'s params (a single param gets the whole
/// argument; otherwise bind by name from an object argument).
fn main_args(defs: &Map<String, Value>, argument: Value) -> Map<String, Value> {
    let params = defs
        .get("main")
        .and_then(|m| m.get("params"))
        .and_then(Value::as_array);
    let mut scope = Map::new();
    match params {
        Some(names) if names.len() == 1 => {
            if let Some(name) = names[0].as_str() {
                scope.insert(name.to_string(), argument);
            }
        }
        Some(names) => {
            for n in names {
                if let Some(name) = n.as_str() {
                    scope.insert(
                        name.to_string(),
                        argument.get(name).cloned().unwrap_or(Value::Null),
                    );
                }
            }
        }
        None => {
            scope.insert("args".to_string(), argument);
        }
    }
    scope
}

impl<'a> Interp<'a> {
    fn tick(&mut self) -> Result<(), String> {
        self.budget = self
            .budget
            .checked_sub(1)
            .ok_or("workflow step budget exceeded")?;
        Ok(())
    }

    /// Run a named definition with a prepared scope; return its result value.
    async fn run_subworkflow(
        &mut self,
        name: &str,
        scope: Map<String, Value>,
    ) -> Result<Value, String> {
        let def = self
            .defs
            .get(name)
            .ok_or_else(|| format!("no subworkflow `{name}`"))?
            .clone();
        let steps = def
            .get("steps")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("definition `{name}` has no steps"))?
            .clone();
        let mut scope = scope;
        match Box::pin(self.run_steps(&steps, &mut scope)).await? {
            Flow::Return(v) => Ok(v),
            Flow::Fell => Ok(Value::Null),
        }
    }

    /// Run a list of `{name: body}` steps with in-list `next` jumps.
    async fn run_steps(&mut self, steps: &[Value], scope: &mut Scope) -> Result<Flow, String> {
        let named = parse_steps(steps)?;
        let mut i = 0;
        while i < named.len() {
            self.tick()?;
            let (_, body) = &named[i];
            match Box::pin(self.exec_step(body, scope)).await? {
                StepOut::Normal => i += 1,
                StepOut::Return(v) => return Ok(Flow::Return(v)),
                StepOut::Goto(target) => {
                    if target == "end" {
                        return Ok(Flow::Fell);
                    }
                    i = named
                        .iter()
                        .position(|(n, _)| n == &target)
                        .ok_or_else(|| format!("next: unknown step `{target}`"))?;
                }
            }
        }
        Ok(Flow::Fell)
    }

    async fn exec_step(&mut self, body: &Value, scope: &mut Scope) -> Result<StepOut, String> {
        let obj = body.as_object().ok_or("step body must be a map")?;
        if let Some(assigns) = obj.get("assign") {
            self.do_assign(assigns, scope)?;
            return Ok(StepOut::Normal);
        }
        if let Some(call) = obj.get("call").and_then(Value::as_str) {
            self.do_call(call, obj, scope).await?;
            return Ok(StepOut::Normal);
        }
        if let Some(cases) = obj.get("switch") {
            return Box::pin(self.do_switch(cases, scope)).await;
        }
        if let Some(forb) = obj.get("for") {
            return Box::pin(self.do_for(forb, scope)).await;
        }
        if obj.contains_key("try") {
            return Box::pin(self.do_try(obj, scope)).await;
        }
        if let Some(next) = obj.get("next").and_then(Value::as_str) {
            return Ok(StepOut::Goto(next.to_string()));
        }
        if let Some(ret) = obj.get("return") {
            return Ok(StepOut::Return(expr::eval_value(ret, scope)?));
        }
        if let Some(r) = obj.get("raise") {
            let v = expr::eval_value(r, scope)?;
            return Err(expr::to_text(&v));
        }
        Err(format!("unsupported step: {body}"))
    }

    fn do_assign(&self, assigns: &Value, scope: &mut Scope) -> Result<(), String> {
        let list = assigns.as_array().ok_or("assign must be a list")?;
        for entry in list {
            let map = entry.as_object().ok_or("assign entry must be a map")?;
            for (target, rhs) in map {
                let val = expr::eval_value(rhs, scope)?;
                assign_target(scope, target, val)?;
            }
        }
        Ok(())
    }

    async fn do_call(
        &mut self,
        call: &str,
        obj: &Map<String, Value>,
        scope: &mut Scope,
    ) -> Result<(), String> {
        let args = match obj.get("args") {
            Some(a) => expr::eval_value(a, scope)?,
            None => Value::Object(Map::new()),
        };
        let result = if let Some(method) = call.strip_prefix("http.") {
            self.call_http(method, &args).await?
        } else if call == "sys.log" {
            if let Some(text) = args.get("text").or_else(|| args.get("data")) {
                eprintln!("vat workflow sys.log: {}", expr::to_text(text));
            }
            Value::Null
        } else {
            // Named subworkflow.
            let sub_scope = subworkflow_scope(&self.defs, call, &args)?;
            Box::pin(self.run_subworkflow(call, sub_scope)).await?
        };
        if let Some(result_var) = obj.get("result").and_then(Value::as_str) {
            scope.insert(result_var.to_string(), result);
        }
        Ok(())
    }

    async fn call_http(&mut self, method: &str, args: &Value) -> Result<Value, String> {
        let http_method = match method {
            "get" | "post" | "put" | "delete" | "patch" => method.to_uppercase(),
            "request" => args
                .get("method")
                .and_then(Value::as_str)
                .unwrap_or("GET")
                .to_uppercase(),
            other => return Err(format!("unsupported call: http.{other}")),
        };
        let url = args
            .get("url")
            .and_then(Value::as_str)
            .ok_or("http call needs `url`")?
            .to_string();
        let mut headers = BTreeMap::new();
        if let Some(h) = args.get("headers").and_then(Value::as_object) {
            for (k, v) in h {
                headers.insert(k.clone(), expr::to_text(v));
            }
        }
        let body = match args.get("body") {
            Some(Value::String(s)) => s.clone().into_bytes(),
            Some(other) => serde_json::to_vec(other).unwrap_or_default(),
            None => Vec::new(),
        };
        if args.get("body").is_some() && !headers.contains_key("Content-Type") {
            headers.insert("Content-Type".to_string(), "application/json".to_string());
        }
        let oidc = args.get("auth").and_then(|a| {
            let audience = a
                .get("audience")
                .and_then(Value::as_str)
                .unwrap_or(&url)
                .to_string();
            matches!(
                a.get("type").and_then(Value::as_str),
                Some("OIDC") | Some("OAuth2")
            )
            .then(|| Oidc {
                service_account_email: "workflow@vat-emulator".to_string(),
                audience,
            })
        });
        let target = Target {
            uri: url,
            method: http_method,
            headers,
            body,
            oidc,
        };
        let resp = dispatch_collect(self.client, &target)
            .await
            .map_err(|e| format!("http call failed: {e}"))?;
        if resp.code >= 400 {
            return Err(format!("HTTP {} from target", resp.code));
        }
        // Parse JSON body when possible, else keep as a string.
        let body: Value = serde_json::from_str(&resp.body).unwrap_or(Value::String(resp.body));
        Ok(json!({ "code": resp.code, "body": body }))
    }

    async fn do_switch(&mut self, cases: &Value, scope: &mut Scope) -> Result<StepOut, String> {
        let list = cases.as_array().ok_or("switch must be a list")?;
        for case in list {
            let cond = case.get("condition").ok_or("switch case needs condition")?;
            if expr::eval_value(cond, scope)? == Value::Bool(true) {
                if let Some(next) = case.get("next").and_then(Value::as_str) {
                    return Ok(StepOut::Goto(next.to_string()));
                }
                if let Some(ret) = case.get("return") {
                    return Ok(StepOut::Return(expr::eval_value(ret, scope)?));
                }
                if let Some(steps) = case.get("steps").and_then(Value::as_array) {
                    if let Flow::Return(v) = Box::pin(self.run_steps(steps, scope)).await? {
                        return Ok(StepOut::Return(v));
                    }
                }
                return Ok(StepOut::Normal);
            }
        }
        Ok(StepOut::Normal)
    }

    async fn do_for(&mut self, forb: &Value, scope: &mut Scope) -> Result<StepOut, String> {
        let f = forb.as_object().ok_or("for must be a map")?;
        let value_var = f
            .get("value")
            .and_then(Value::as_str)
            .unwrap_or("v")
            .to_string();
        let index_var = f.get("index").and_then(Value::as_str).map(str::to_string);
        let steps = f
            .get("steps")
            .and_then(Value::as_array)
            .ok_or("for needs steps")?
            .clone();

        let items: Vec<Value> = if let Some(range) = f.get("range").and_then(Value::as_array) {
            let a = range.first().and_then(Value::as_i64).unwrap_or(0);
            let b = range.get(1).and_then(Value::as_i64).unwrap_or(0);
            (a..=b).map(Value::from).collect()
        } else if let Some(in_expr) = f.get("in") {
            match expr::eval_value(in_expr, scope)? {
                Value::Array(a) => a,
                other => return Err(format!("for..in needs a list, got {other}")),
            }
        } else {
            return Err("for needs `in` or `range`".to_string());
        };

        for (i, item) in items.into_iter().enumerate() {
            self.tick()?;
            scope.insert(value_var.clone(), item);
            if let Some(iv) = &index_var {
                scope.insert(iv.clone(), Value::from(i));
            }
            if let Flow::Return(v) = Box::pin(self.run_steps(&steps, scope)).await? {
                return Ok(StepOut::Return(v));
            }
        }
        Ok(StepOut::Normal)
    }

    async fn do_try(
        &mut self,
        obj: &Map<String, Value>,
        scope: &mut Scope,
    ) -> Result<StepOut, String> {
        let try_block = obj.get("try").ok_or("try needs a body")?;
        let max_retries = obj
            .get("retry")
            .and_then(|r| r.get("max_retries").or_else(|| r.get("maxRetries")))
            .and_then(Value::as_u64)
            .unwrap_or(0);
        let attempts = max_retries + 1;

        let mut last_err = String::new();
        for attempt in 0..attempts {
            match Box::pin(self.run_try_body(try_block, scope)).await {
                Ok(StepOut::Normal) => return Ok(StepOut::Normal),
                Ok(other) => return Ok(other),
                Err(e) => {
                    last_err = e;
                    if attempt + 1 < attempts {
                        tokio::time::sleep(Duration::from_millis(10)).await;
                    }
                }
            }
        }

        // All attempts failed: run except, binding the error.
        if let Some(except) = obj.get("except").and_then(Value::as_object) {
            if let Some(as_var) = except.get("as").and_then(Value::as_str) {
                scope.insert(as_var.to_string(), json!({ "message": last_err }));
            }
            if let Some(steps) = except.get("steps").and_then(Value::as_array) {
                if let Flow::Return(v) = Box::pin(self.run_steps(steps, scope)).await? {
                    return Ok(StepOut::Return(v));
                }
                return Ok(StepOut::Normal);
            }
        }
        Err(last_err)
    }

    /// A try body is either `{steps: [...]}` or a single inline step body.
    async fn run_try_body(
        &mut self,
        try_block: &Value,
        scope: &mut Scope,
    ) -> Result<StepOut, String> {
        if let Some(steps) = try_block.get("steps").and_then(Value::as_array) {
            match Box::pin(self.run_steps(steps, scope)).await? {
                Flow::Return(v) => Ok(StepOut::Return(v)),
                Flow::Fell => Ok(StepOut::Normal),
            }
        } else {
            Box::pin(self.exec_step(try_block, scope)).await
        }
    }
}

/// Parse `[{name: body}, ...]` into ordered (name, body) pairs.
fn parse_steps(steps: &[Value]) -> Result<Vec<(String, Value)>, String> {
    let mut out = Vec::with_capacity(steps.len());
    for step in steps {
        let map = step.as_object().ok_or("each step must be a map")?;
        let (name, body) = map.iter().next().ok_or("empty step")?;
        out.push((name.clone(), body.clone()));
    }
    Ok(out)
}

/// Bind a subworkflow's params from a call's `args` map.
fn subworkflow_scope(
    defs: &Map<String, Value>,
    name: &str,
    args: &Value,
) -> Result<Map<String, Value>, String> {
    let params = defs
        .get(name)
        .and_then(|d| d.get("params"))
        .and_then(Value::as_array);
    let mut scope = Map::new();
    if let Some(names) = params {
        for n in names {
            if let Some(pname) = n.as_str() {
                scope.insert(
                    pname.to_string(),
                    args.get(pname).cloned().unwrap_or(Value::Null),
                );
            }
        }
    }
    Ok(scope)
}

/// Assign into scope, supporting a simple dotted target (`a.b`) one level deep.
fn assign_target(scope: &mut Scope, target: &str, val: Value) -> Result<(), String> {
    if let Some((head, tail)) = target.split_once('.') {
        let entry = scope
            .entry(head.to_string())
            .or_insert_with(|| Value::Object(Map::new()));
        let obj = entry
            .as_object_mut()
            .ok_or_else(|| format!("cannot assign into non-object `{head}`"))?;
        obj.insert(tail.to_string(), val);
    } else {
        scope.insert(target.to_string(), val);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_sync(source: &str, arg: Value) -> Result<Value, String> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let client = reqwest::Client::new();
        rt.block_on(run(source, arg, &client))
    }

    #[test]
    fn assign_switch_for_return() {
        let src = r#"
main:
  params: [args]
  steps:
    - init:
        assign:
          - total: 0
          - n: ${args.n}
    - loop:
        for:
          value: i
          range: [1, 3]
          steps:
            - add:
                assign:
                  - total: ${total + i}
    - decide:
        switch:
          - condition: ${total > 5}
            return: "big"
          - condition: true
            return: "small"
"#;
        assert_eq!(run_sync(src, json!({"n": 9})).unwrap(), json!("big"));
    }

    #[test]
    fn subworkflow_call() {
        let src = r#"
main:
  steps:
    - call_double:
        call: double
        args:
          x: 21
        result: r
    - done:
        return: ${r}
double:
  params: [x]
  steps:
    - ret:
        return: ${x * 2}
"#;
        assert_eq!(run_sync(src, json!({})).unwrap(), json!(42));
    }

    #[test]
    fn try_except_catches() {
        // http call to a closed port fails -> except returns the fallback.
        let src = r#"
main:
  steps:
    - attempt:
        try:
          steps:
            - call_dead:
                call: http.get
                args:
                  url: "http://127.0.0.1:1/nope"
                result: r
        except:
          as: e
          steps:
            - fallback:
                return: "recovered"
"#;
        assert_eq!(run_sync(src, json!({})).unwrap(), json!("recovered"));
    }

    #[test]
    fn errors_do_not_panic() {
        assert!(run_sync("not: a: workflow", json!({})).is_err());
        assert!(run_sync("steps: []", json!({})).is_err()); // no main
    }
}
// CODEGEN-END
