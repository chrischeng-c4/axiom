// SPEC-MANAGED: projects/vat/tech-design/semantic/vat-src.md#schema
// CODEGEN-BEGIN
//! The `${...}` expression evaluator for the built-in Cloud Workflows emulator.
//!
//! A small hand-rolled tokenizer + recursive-descent (Pratt-style) parser and
//! evaluator over `serde_json::Value`, supporting the bounded subset a local
//! workflow needs: variable/member/index access, literals, arithmetic,
//! comparison, boolean logic, string concat, a handful of builtins, and `${}`
//! string interpolation. Anything unsupported returns an `Err(String)` that the
//! interpreter turns into a workflow error — never a panic.
//!
//! @spec projects/vat/tech-design/logic/built-in-cloud-workflows-emulator.md#logic

use serde_json::{Map, Value};

/// Variable bindings in scope during evaluation.
pub type Scope = Map<String, Value>;

/// Resolve a workflow *value*: a `${...}` string evaluates to the inner
/// expression's value, a string with embedded `${...}` interpolates, and
/// arrays/objects are resolved recursively. Non-string scalars pass through.
pub fn eval_value(v: &Value, scope: &Scope) -> Result<Value, String> {
    match v {
        Value::String(s) => eval_string(s, scope),
        Value::Array(items) => {
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                out.push(eval_value(item, scope)?);
            }
            Ok(Value::Array(out))
        }
        Value::Object(map) => {
            let mut out = Map::new();
            for (k, val) in map {
                out.insert(k.clone(), eval_value(val, scope)?);
            }
            Ok(Value::Object(out))
        }
        other => Ok(other.clone()),
    }
}

/// A whole `${expr}` → the expression's value; embedded `${...}` → interpolated
/// string; no `${}` → the literal string.
fn eval_string(s: &str, scope: &Scope) -> Result<Value, String> {
    let trimmed = s.trim();
    if let Some(inner) = whole_expr(trimmed) {
        return eval_expr(inner, scope);
    }
    if !s.contains("${") {
        return Ok(Value::String(s.to_string()));
    }
    // Interpolation: replace each ${...} with the stringified value.
    let mut out = String::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'$' && i + 1 < bytes.len() && bytes[i + 1] == b'{' {
            let start = i + 2;
            let mut depth = 1;
            let mut j = start;
            while j < bytes.len() && depth > 0 {
                match bytes[j] {
                    b'{' => depth += 1,
                    b'}' => depth -= 1,
                    _ => {}
                }
                if depth == 0 {
                    break;
                }
                j += 1;
            }
            let expr = &s[start..j];
            let val = eval_expr(expr, scope)?;
            out.push_str(&to_text(&val));
            i = j + 1;
        } else {
            out.push(bytes[i] as char);
            i += 1;
        }
    }
    Ok(Value::String(out))
}

/// `${ ... }` covering the whole string → the inner expression text.
fn whole_expr(s: &str) -> Option<&str> {
    let inner = s.strip_prefix("${")?.strip_suffix('}')?;
    // Ensure the closing brace matches the opening one (no early close).
    let mut depth = 1;
    for c in inner.chars() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return None; // closed early -> embedded, not whole
                }
            }
            _ => {}
        }
    }
    Some(inner)
}

/// Evaluate an expression string against `scope`.
pub fn eval_expr(src: &str, scope: &Scope) -> Result<Value, String> {
    let tokens = tokenize(src)?;
    let mut p = Parser { tokens, pos: 0 };
    let v = p.parse_or(scope)?;
    if p.pos != p.tokens.len() {
        return Err(format!("trailing tokens in expression: {src}"));
    }
    Ok(v)
}

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Num(f64),
    Str(String),
    Ident(String),
    True,
    False,
    Null,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Not,
    Dot,
    LBrack,
    RBrack,
    LParen,
    RParen,
    Comma,
}

fn tokenize(src: &str) -> Result<Vec<Tok>, String> {
    let mut toks = Vec::new();
    let chars: Vec<char> = src.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        match c {
            ' ' | '\t' | '\n' | '\r' => i += 1,
            '+' => {
                toks.push(Tok::Plus);
                i += 1;
            }
            '-' => {
                toks.push(Tok::Minus);
                i += 1;
            }
            '*' => {
                toks.push(Tok::Star);
                i += 1;
            }
            '/' => {
                toks.push(Tok::Slash);
                i += 1;
            }
            '%' => {
                toks.push(Tok::Percent);
                i += 1;
            }
            '.' => {
                toks.push(Tok::Dot);
                i += 1;
            }
            '[' => {
                toks.push(Tok::LBrack);
                i += 1;
            }
            ']' => {
                toks.push(Tok::RBrack);
                i += 1;
            }
            '(' => {
                toks.push(Tok::LParen);
                i += 1;
            }
            ')' => {
                toks.push(Tok::RParen);
                i += 1;
            }
            ',' => {
                toks.push(Tok::Comma);
                i += 1;
            }
            '=' if i + 1 < chars.len() && chars[i + 1] == '=' => {
                toks.push(Tok::Eq);
                i += 2;
            }
            '!' if i + 1 < chars.len() && chars[i + 1] == '=' => {
                toks.push(Tok::Ne);
                i += 2;
            }
            '<' if i + 1 < chars.len() && chars[i + 1] == '=' => {
                toks.push(Tok::Le);
                i += 2;
            }
            '>' if i + 1 < chars.len() && chars[i + 1] == '=' => {
                toks.push(Tok::Ge);
                i += 2;
            }
            '<' => {
                toks.push(Tok::Lt);
                i += 1;
            }
            '>' => {
                toks.push(Tok::Gt);
                i += 1;
            }
            '"' | '\'' => {
                let quote = c;
                i += 1;
                let mut s = String::new();
                while i < chars.len() && chars[i] != quote {
                    if chars[i] == '\\' && i + 1 < chars.len() {
                        i += 1;
                        s.push(match chars[i] {
                            'n' => '\n',
                            't' => '\t',
                            other => other,
                        });
                    } else {
                        s.push(chars[i]);
                    }
                    i += 1;
                }
                if i >= chars.len() {
                    return Err("unterminated string literal".to_string());
                }
                i += 1; // closing quote
                toks.push(Tok::Str(s));
            }
            c if c.is_ascii_digit() => {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    i += 1;
                }
                let lit: String = chars[start..i].iter().collect();
                let n: f64 = lit.parse().map_err(|_| format!("bad number: {lit}"))?;
                toks.push(Tok::Num(n));
            }
            c if c.is_alphabetic() || c == '_' => {
                let start = i;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let id: String = chars[start..i].iter().collect();
                toks.push(match id.as_str() {
                    "true" => Tok::True,
                    "false" => Tok::False,
                    "null" => Tok::Null,
                    "and" => Tok::And,
                    "or" => Tok::Or,
                    "not" => Tok::Not,
                    _ => Tok::Ident(id),
                });
            }
            other => return Err(format!("unexpected character: {other}")),
        }
    }
    Ok(toks)
}

struct Parser {
    tokens: Vec<Tok>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Tok> {
        self.tokens.get(self.pos)
    }

    fn eat(&mut self, t: &Tok) -> bool {
        if self.peek() == Some(t) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn parse_or(&mut self, scope: &Scope) -> Result<Value, String> {
        let mut left = self.parse_and(scope)?;
        while self.eat(&Tok::Or) {
            let right = self.parse_and(scope)?;
            left = Value::Bool(truthy(&left) || truthy(&right));
        }
        Ok(left)
    }

    fn parse_and(&mut self, scope: &Scope) -> Result<Value, String> {
        let mut left = self.parse_cmp(scope)?;
        while self.eat(&Tok::And) {
            let right = self.parse_cmp(scope)?;
            left = Value::Bool(truthy(&left) && truthy(&right));
        }
        Ok(left)
    }

    fn parse_cmp(&mut self, scope: &Scope) -> Result<Value, String> {
        let left = self.parse_add(scope)?;
        let op = match self.peek() {
            Some(Tok::Eq) => Tok::Eq,
            Some(Tok::Ne) => Tok::Ne,
            Some(Tok::Lt) => Tok::Lt,
            Some(Tok::Le) => Tok::Le,
            Some(Tok::Gt) => Tok::Gt,
            Some(Tok::Ge) => Tok::Ge,
            _ => return Ok(left),
        };
        self.pos += 1;
        let right = self.parse_add(scope)?;
        compare(&left, &op, &right)
    }

    fn parse_add(&mut self, scope: &Scope) -> Result<Value, String> {
        let mut left = self.parse_mul(scope)?;
        loop {
            match self.peek() {
                Some(Tok::Plus) => {
                    self.pos += 1;
                    let right = self.parse_mul(scope)?;
                    left = add(&left, &right)?;
                }
                Some(Tok::Minus) => {
                    self.pos += 1;
                    let right = self.parse_mul(scope)?;
                    left = num_op(&left, &right, |a, b| a - b)?;
                }
                _ => return Ok(left),
            }
        }
    }

    fn parse_mul(&mut self, scope: &Scope) -> Result<Value, String> {
        let mut left = self.parse_unary(scope)?;
        loop {
            match self.peek() {
                Some(Tok::Star) => {
                    self.pos += 1;
                    let right = self.parse_unary(scope)?;
                    left = num_op(&left, &right, |a, b| a * b)?;
                }
                Some(Tok::Slash) => {
                    self.pos += 1;
                    let right = self.parse_unary(scope)?;
                    left = num_op(&left, &right, |a, b| a / b)?;
                }
                Some(Tok::Percent) => {
                    self.pos += 1;
                    let right = self.parse_unary(scope)?;
                    left = num_op(&left, &right, |a, b| a % b)?;
                }
                _ => return Ok(left),
            }
        }
    }

    fn parse_unary(&mut self, scope: &Scope) -> Result<Value, String> {
        if self.eat(&Tok::Minus) {
            let v = self.parse_unary(scope)?;
            return num_op(&Value::from(0), &v, |a, b| a - b);
        }
        if self.eat(&Tok::Not) {
            let v = self.parse_unary(scope)?;
            return Ok(Value::Bool(!truthy(&v)));
        }
        self.parse_postfix(scope)
    }

    fn parse_postfix(&mut self, scope: &Scope) -> Result<Value, String> {
        let mut v = self.parse_primary(scope)?;
        loop {
            match self.peek() {
                Some(Tok::Dot) => {
                    self.pos += 1;
                    let field = match self.peek() {
                        Some(Tok::Ident(id)) => id.clone(),
                        _ => return Err("expected field name after '.'".to_string()),
                    };
                    self.pos += 1;
                    v = member(&v, &field)?;
                }
                Some(Tok::LBrack) => {
                    self.pos += 1;
                    let idx = self.parse_or(scope)?;
                    if !self.eat(&Tok::RBrack) {
                        return Err("expected ']'".to_string());
                    }
                    v = index(&v, &idx)?;
                }
                _ => return Ok(v),
            }
        }
    }

    fn parse_primary(&mut self, scope: &Scope) -> Result<Value, String> {
        let tok = self.peek().cloned().ok_or("unexpected end of expression")?;
        match tok {
            Tok::Num(n) => {
                self.pos += 1;
                Ok(number(n))
            }
            Tok::Str(s) => {
                self.pos += 1;
                Ok(Value::String(s))
            }
            Tok::True => {
                self.pos += 1;
                Ok(Value::Bool(true))
            }
            Tok::False => {
                self.pos += 1;
                Ok(Value::Bool(false))
            }
            Tok::Null => {
                self.pos += 1;
                Ok(Value::Null)
            }
            Tok::LParen => {
                self.pos += 1;
                let v = self.parse_or(scope)?;
                if !self.eat(&Tok::RParen) {
                    return Err("expected ')'".to_string());
                }
                Ok(v)
            }
            Tok::Ident(id) => {
                self.pos += 1;
                // Function call?
                if self.eat(&Tok::LParen) {
                    let mut args = Vec::new();
                    if self.peek() != Some(&Tok::RParen) {
                        loop {
                            args.push(self.parse_or(scope)?);
                            if !self.eat(&Tok::Comma) {
                                break;
                            }
                        }
                    }
                    if !self.eat(&Tok::RParen) {
                        return Err("expected ')' after call args".to_string());
                    }
                    call_builtin(&id, &args)
                } else {
                    scope
                        .get(&id)
                        .cloned()
                        .ok_or_else(|| format!("undefined variable: {id}"))
                }
            }
            other => Err(format!("unexpected token: {other:?}")),
        }
    }
}

fn number(n: f64) -> Value {
    if n.fract() == 0.0 && n.abs() < 9e15 {
        Value::from(n as i64)
    } else {
        Value::from(n)
    }
}

fn as_f64(v: &Value) -> Result<f64, String> {
    v.as_f64().ok_or_else(|| format!("not a number: {v}"))
}

fn num_op(a: &Value, b: &Value, f: impl Fn(f64, f64) -> f64) -> Result<Value, String> {
    Ok(number(f(as_f64(a)?, as_f64(b)?)))
}

fn add(a: &Value, b: &Value) -> Result<Value, String> {
    if a.is_string() || b.is_string() {
        Ok(Value::String(format!("{}{}", to_text(a), to_text(b))))
    } else {
        num_op(a, b, |x, y| x + y)
    }
}

fn compare(a: &Value, op: &Tok, b: &Value) -> Result<Value, String> {
    let result = match op {
        Tok::Eq => a == b,
        Tok::Ne => a != b,
        _ => {
            let ord = if let (Some(x), Some(y)) = (a.as_f64(), b.as_f64()) {
                x.partial_cmp(&y)
            } else if let (Some(x), Some(y)) = (a.as_str(), b.as_str()) {
                Some(x.cmp(y))
            } else {
                return Err(format!("cannot compare {a} and {b}"));
            };
            let ord = ord.ok_or("uncomparable values")?;
            match op {
                Tok::Lt => ord.is_lt(),
                Tok::Le => ord.is_le(),
                Tok::Gt => ord.is_gt(),
                Tok::Ge => ord.is_ge(),
                _ => unreachable!(),
            }
        }
    };
    Ok(Value::Bool(result))
}

fn member(v: &Value, field: &str) -> Result<Value, String> {
    v.get(field)
        .cloned()
        .ok_or_else(|| format!("no field '{field}' on {v}"))
}

fn index(v: &Value, idx: &Value) -> Result<Value, String> {
    match (v, idx) {
        (Value::Array(arr), _) => {
            let i = idx.as_u64().ok_or("array index must be a number")? as usize;
            arr.get(i)
                .cloned()
                .ok_or_else(|| format!("index {i} out of bounds"))
        }
        (Value::Object(_), Value::String(k)) => member(v, k),
        _ => Err(format!("cannot index {v}")),
    }
}

fn truthy(v: &Value) -> bool {
    match v {
        Value::Bool(b) => *b,
        Value::Null => false,
        Value::Number(n) => n.as_f64().map(|x| x != 0.0).unwrap_or(false),
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
    }
}

/// Stringify a value for interpolation / `+` concat.
pub fn to_text(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => "null".to_string(),
        other => other.to_string(),
    }
}

fn call_builtin(name: &str, args: &[Value]) -> Result<Value, String> {
    let arg = |i: usize| args.get(i).cloned().unwrap_or(Value::Null);
    match name {
        "len" => match &arg(0) {
            Value::String(s) => Ok(Value::from(s.chars().count())),
            Value::Array(a) => Ok(Value::from(a.len())),
            Value::Object(o) => Ok(Value::from(o.len())),
            other => Err(format!("len() of {other}")),
        },
        "int" => Ok(number(
            as_f64(&arg(0))
                .or_else(|_| {
                    arg(0)
                        .as_str()
                        .and_then(|s| s.trim().parse::<f64>().ok())
                        .ok_or_else(|| "int() of non-number".to_string())
                })?
                .trunc(),
        )),
        "double" => Ok(Value::from(as_f64(&arg(0)).or_else(|_| {
            arg(0)
                .as_str()
                .and_then(|s| s.trim().parse::<f64>().ok())
                .ok_or_else(|| "double() of non-number".to_string())
        })?)),
        "string" => Ok(Value::String(to_text(&arg(0)))),
        "lower" => Ok(Value::String(to_text(&arg(0)).to_lowercase())),
        "upper" => Ok(Value::String(to_text(&arg(0)).to_uppercase())),
        "keys" => match &arg(0) {
            Value::Object(o) => Ok(Value::Array(
                o.keys().map(|k| Value::String(k.clone())).collect(),
            )),
            other => Err(format!("keys() of {other}")),
        },
        "default" => {
            let a = arg(0);
            Ok(if a.is_null() { arg(1) } else { a })
        }
        other => Err(format!("unknown function: {other}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn scope() -> Scope {
        let mut s = Scope::new();
        s.insert("x".into(), json!(5));
        s.insert("name".into(), json!("vat"));
        s.insert("m".into(), json!({"a": 1, "b": [10, 20]}));
        s
    }

    fn e(src: &str) -> Value {
        eval_expr(src, &scope()).unwrap()
    }

    #[test]
    fn literals_and_vars() {
        assert_eq!(e("5"), json!(5));
        assert_eq!(e("true"), json!(true));
        assert_eq!(e("null"), json!(null));
        assert_eq!(e("x"), json!(5));
        assert_eq!(e("name"), json!("vat"));
    }

    #[test]
    fn member_and_index() {
        assert_eq!(e("m.a"), json!(1));
        assert_eq!(e("m.b[1]"), json!(20));
        assert_eq!(e("m[\"a\"]"), json!(1));
    }

    #[test]
    fn arithmetic_and_compare_and_logic() {
        assert_eq!(e("x + 3"), json!(8));
        assert_eq!(e("x * 2 - 1"), json!(9));
        assert_eq!(e("10 % 3"), json!(1));
        assert_eq!(e("x > 3 and x < 10"), json!(true));
        assert_eq!(e("x == 5 or false"), json!(true));
        assert_eq!(e("not (x == 5)"), json!(false));
    }

    #[test]
    fn string_concat_and_builtins() {
        assert_eq!(e("name + \"-1\""), json!("vat-1"));
        assert_eq!(e("\"n=\" + x"), json!("n=5"));
        assert_eq!(e("len(name)"), json!(3));
        assert_eq!(e("len(m.b)"), json!(2));
        assert_eq!(e("upper(name)"), json!("VAT"));
        assert_eq!(e("default(null, 7)"), json!(7));
        assert_eq!(e("default(x, 7)"), json!(5));
        assert_eq!(e("int(\"42\")"), json!(42));
    }

    #[test]
    fn value_interpolation() {
        let s = scope();
        assert_eq!(
            eval_value(&json!("${name}/${x}"), &s).unwrap(),
            json!("vat/5")
        );
        // whole-expr keeps the typed value
        assert_eq!(eval_value(&json!("${x + 1}"), &s).unwrap(), json!(6));
        // plain string passes through
        assert_eq!(eval_value(&json!("plain"), &s).unwrap(), json!("plain"));
        // nested object resolved recursively
        assert_eq!(
            eval_value(&json!({"u": "${name}", "n": "${x}"}), &s).unwrap(),
            json!({"u": "vat", "n": 5})
        );
    }

    #[test]
    fn errors_do_not_panic() {
        let s = scope();
        assert!(eval_expr("nope", &s).is_err());
        assert!(eval_expr("x +", &s).is_err());
        assert!(eval_expr("bogus(1)", &s).is_err());
        assert!(eval_expr("m.missing", &s).is_err());
    }
}
// CODEGEN-END
