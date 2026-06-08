//! PEP 508 environment-marker evaluator.
//!
//! Grammar (informal, after PEP 508 §"Environment markers"):
//!
//! ```text
//! marker      = or
//! or          = and ( "or" and )*
//! and         = expr ( "and" expr )*
//! expr        = "(" marker ")" | comparison
//! comparison  = var op var
//! op          = "==" | "!=" | "<" | "<=" | ">" | ">=" | "~=" | "==="
//!             | "in" | "not in"
//! var         = env_var | string_literal
//! env_var     = "python_version" | "python_full_version" | "os_name"
//!             | "sys_platform" | "platform_release" | "platform_system"
//!             | "platform_version" | "platform_machine"
//!             | "platform_python_implementation"
//!             | "implementation_name" | "implementation_version" | "extra"
//! ```
//!
//! Version-typed env vars (`python_version`, `python_full_version`,
//! `implementation_version`) are compared using the PEP 440 ordering from
//! [`crate::pkgmanage::pkgmgr::pep440`]; everything else is string-compared.
//! `~=` (compatible release) and `===` (arbitrary equality) are honored only
//! between version-typed left-hand sides.

use std::collections::BTreeSet;

use crate::pkgmanage::pkgmgr::pep440;

#[derive(Debug, thiserror::Error)]
pub enum MarkerError {
    #[error("unexpected end of marker at column {col}")]
    UnexpectedEof { col: usize },
    #[error("unexpected token `{got}` at column {col}: {detail}")]
    Unexpected { got: String, col: usize, detail: String },
    #[error("unknown environment variable `{name}`")]
    UnknownEnvVar { name: String },
    #[error("operator `{op}` requires a PEP 440 version on at least one side")]
    NeedsVersion { op: String },
    #[error("malformed string literal at column {col}")]
    BadString { col: usize },
}

/// Runtime environment exposed to PEP 508 markers.
#[derive(Debug, Clone)]
pub struct MarkerEnv {
    pub python_version: String,
    pub python_full_version: String,
    pub implementation_name: String,
    pub implementation_version: String,
    pub os_name: String,
    pub platform_machine: String,
    pub platform_release: String,
    pub platform_system: String,
    pub platform_version: String,
    pub platform_python_implementation: String,
    pub sys_platform: String,
    pub extras: BTreeSet<String>,
}

impl MarkerEnv {
    /// Detect from the current host. `python_version` defaults to the build
    /// target's stable Python; callers that need a different interpreter
    /// should override after construction.
    pub fn current_host() -> Self {
        Self {
            python_version: "3.12".to_string(),
            python_full_version: "3.12.0".to_string(),
            implementation_name: "cpython".to_string(),
            implementation_version: "3.12.0".to_string(),
            os_name: detect_os_name(),
            platform_machine: detect_platform_machine(),
            platform_release: String::new(),
            platform_system: detect_platform_system(),
            platform_version: String::new(),
            platform_python_implementation: "CPython".to_string(),
            sys_platform: detect_sys_platform(),
            extras: BTreeSet::new(),
        }
    }

    fn lookup(&self, name: &str) -> Option<&str> {
        match name {
            "python_version" => Some(&self.python_version),
            "python_full_version" => Some(&self.python_full_version),
            "implementation_name" => Some(&self.implementation_name),
            "implementation_version" => Some(&self.implementation_version),
            "os_name" => Some(&self.os_name),
            "platform_machine" => Some(&self.platform_machine),
            "platform_release" => Some(&self.platform_release),
            "platform_system" => Some(&self.platform_system),
            "platform_version" => Some(&self.platform_version),
            "platform_python_implementation" => Some(&self.platform_python_implementation),
            "sys_platform" => Some(&self.sys_platform),
            _ => None,
        }
    }

    fn is_version_typed(name: &str) -> bool {
        matches!(
            name,
            "python_version" | "python_full_version" | "implementation_version"
        )
    }
}

fn detect_os_name() -> String {
    if cfg!(target_family = "unix") {
        "posix".to_string()
    } else if cfg!(target_family = "windows") {
        "nt".to_string()
    } else {
        "java".to_string()
    }
}

fn detect_sys_platform() -> String {
    if cfg!(target_os = "macos") {
        "darwin".to_string()
    } else if cfg!(target_os = "linux") {
        "linux".to_string()
    } else if cfg!(target_os = "windows") {
        "win32".to_string()
    } else {
        std::env::consts::OS.to_string()
    }
}

fn detect_platform_system() -> String {
    if cfg!(target_os = "macos") {
        "Darwin".to_string()
    } else if cfg!(target_os = "linux") {
        "Linux".to_string()
    } else if cfg!(target_os = "windows") {
        "Windows".to_string()
    } else {
        std::env::consts::OS.to_string()
    }
}

fn detect_platform_machine() -> String {
    std::env::consts::ARCH.to_string()
}

/// Top-level: parse + evaluate `marker` against `env`.
pub fn evaluate(marker: &str, env: &MarkerEnv) -> Result<bool, MarkerError> {
    let tokens = lex(marker)?;
    let mut p = Parser { tokens: &tokens, pos: 0 };
    let ast = p.parse_or()?;
    if p.pos != tokens.len() {
        return Err(MarkerError::Unexpected {
            got: format!("{:?}", tokens[p.pos]),
            col: 0,
            detail: "trailing tokens".to_string(),
        });
    }
    eval_node(&ast, env)
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Tok {
    Ident(String),       // env_var name
    Str(String),         // string literal
    Op(String),          // ==, !=, <, <=, >, >=, ~=, ===, in, not in
    And,
    Or,
    LParen,
    RParen,
}

fn lex(src: &str) -> Result<Vec<Tok>, MarkerError> {
    let bytes = src.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if c.is_ascii_whitespace() {
            i += 1;
            continue;
        }
        match c {
            b'(' => {
                out.push(Tok::LParen);
                i += 1;
            }
            b')' => {
                out.push(Tok::RParen);
                i += 1;
            }
            b'\'' | b'"' => {
                let quote = c;
                let start = i + 1;
                let mut j = start;
                while j < bytes.len() && bytes[j] != quote {
                    j += 1;
                }
                if j >= bytes.len() {
                    return Err(MarkerError::BadString { col: i });
                }
                let s = std::str::from_utf8(&bytes[start..j])
                    .map_err(|_| MarkerError::BadString { col: i })?
                    .to_string();
                out.push(Tok::Str(s));
                i = j + 1;
            }
            b'=' | b'!' | b'<' | b'>' | b'~' => {
                // === / == / != / <= / >= / ~= / < / >
                let three = std::str::from_utf8(&bytes[i..(i + 3).min(bytes.len())])
                    .unwrap_or("");
                if three == "===" {
                    out.push(Tok::Op("===".into()));
                    i += 3;
                    continue;
                }
                let two = std::str::from_utf8(&bytes[i..(i + 2).min(bytes.len())])
                    .unwrap_or("");
                if matches!(two, "==" | "!=" | "<=" | ">=" | "~=") {
                    out.push(Tok::Op(two.into()));
                    i += 2;
                    continue;
                }
                if matches!(c, b'<' | b'>') {
                    out.push(Tok::Op(String::from(c as char)));
                    i += 1;
                    continue;
                }
                return Err(MarkerError::Unexpected {
                    got: (c as char).to_string(),
                    col: i,
                    detail: "stray operator char".to_string(),
                });
            }
            _ if c.is_ascii_alphabetic() || c == b'_' => {
                let start = i;
                while i < bytes.len()
                    && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_')
                {
                    i += 1;
                }
                let word = &src[start..i];
                match word {
                    "and" => out.push(Tok::And),
                    "or" => out.push(Tok::Or),
                    "in" => out.push(Tok::Op("in".into())),
                    "not" => {
                        // `not in` is two tokens — peek for `in`.
                        let mut k = i;
                        while k < bytes.len() && bytes[k].is_ascii_whitespace() {
                            k += 1;
                        }
                        if k + 2 <= bytes.len() && &src[k..k + 2] == "in" {
                            // Ensure it's a whole word, not a prefix.
                            let after = bytes.get(k + 2).copied().unwrap_or(b' ');
                            if !after.is_ascii_alphanumeric() && after != b'_' {
                                out.push(Tok::Op("not in".into()));
                                i = k + 2;
                                continue;
                            }
                        }
                        return Err(MarkerError::Unexpected {
                            got: "not".into(),
                            col: start,
                            detail: "expected `not in`".to_string(),
                        });
                    }
                    _ => out.push(Tok::Ident(word.to_string())),
                }
            }
            _ => {
                return Err(MarkerError::Unexpected {
                    got: (c as char).to_string(),
                    col: i,
                    detail: "unrecognized char".to_string(),
                })
            }
        }
    }
    Ok(out)
}

#[derive(Debug)]
enum Node {
    Or(Vec<Node>),
    And(Vec<Node>),
    Cmp { lhs: Operand, op: String, rhs: Operand },
}

#[derive(Debug, Clone)]
enum Operand {
    Var(String),
    Lit(String),
}

struct Parser<'a> {
    tokens: &'a [Tok],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<&Tok> {
        self.tokens.get(self.pos)
    }
    fn bump(&mut self) -> Option<&Tok> {
        let t = self.tokens.get(self.pos);
        self.pos += 1;
        t
    }

    fn parse_or(&mut self) -> Result<Node, MarkerError> {
        let mut acc = vec![self.parse_and()?];
        while matches!(self.peek(), Some(Tok::Or)) {
            self.bump();
            acc.push(self.parse_and()?);
        }
        if acc.len() == 1 {
            Ok(acc.pop().unwrap())
        } else {
            Ok(Node::Or(acc))
        }
    }

    fn parse_and(&mut self) -> Result<Node, MarkerError> {
        let mut acc = vec![self.parse_expr()?];
        while matches!(self.peek(), Some(Tok::And)) {
            self.bump();
            acc.push(self.parse_expr()?);
        }
        if acc.len() == 1 {
            Ok(acc.pop().unwrap())
        } else {
            Ok(Node::And(acc))
        }
    }

    fn parse_expr(&mut self) -> Result<Node, MarkerError> {
        match self.peek() {
            Some(Tok::LParen) => {
                self.bump();
                let inner = self.parse_or()?;
                match self.bump() {
                    Some(Tok::RParen) => Ok(inner),
                    other => Err(MarkerError::Unexpected {
                        got: format!("{other:?}"),
                        col: 0,
                        detail: "expected `)`".to_string(),
                    }),
                }
            }
            Some(_) => self.parse_cmp(),
            None => Err(MarkerError::UnexpectedEof { col: 0 }),
        }
    }

    fn parse_cmp(&mut self) -> Result<Node, MarkerError> {
        let lhs = self.parse_operand()?;
        let op = match self.bump() {
            Some(Tok::Op(s)) => s.clone(),
            other => {
                return Err(MarkerError::Unexpected {
                    got: format!("{other:?}"),
                    col: 0,
                    detail: "expected comparison operator".to_string(),
                })
            }
        };
        let rhs = self.parse_operand()?;
        Ok(Node::Cmp { lhs, op, rhs })
    }

    fn parse_operand(&mut self) -> Result<Operand, MarkerError> {
        match self.bump() {
            Some(Tok::Ident(s)) => Ok(Operand::Var(s.clone())),
            Some(Tok::Str(s)) => Ok(Operand::Lit(s.clone())),
            other => Err(MarkerError::Unexpected {
                got: format!("{other:?}"),
                col: 0,
                detail: "expected env var or string".to_string(),
            }),
        }
    }
}

fn eval_node(node: &Node, env: &MarkerEnv) -> Result<bool, MarkerError> {
    match node {
        Node::Or(items) => {
            for it in items {
                if eval_node(it, env)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        Node::And(items) => {
            for it in items {
                if !eval_node(it, env)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        Node::Cmp { lhs, op, rhs } => eval_cmp(lhs, op, rhs, env),
    }
}

fn eval_cmp(
    lhs: &Operand,
    op: &str,
    rhs: &Operand,
    env: &MarkerEnv,
) -> Result<bool, MarkerError> {
    // `extra` is always evaluated against the active extras set.
    if matches!(lhs, Operand::Var(n) if n == "extra")
        || matches!(rhs, Operand::Var(n) if n == "extra")
    {
        let lit = match (lhs, rhs) {
            (Operand::Var(_), Operand::Lit(s)) | (Operand::Lit(s), Operand::Var(_)) => s.clone(),
            _ => {
                return Err(MarkerError::Unexpected {
                    got: format!("{op}"),
                    col: 0,
                    detail: "`extra` requires a string literal on the other side".to_string(),
                })
            }
        };
        return Ok(match op {
            "==" => env.extras.contains(&lit),
            "!=" => !env.extras.contains(&lit),
            other => {
                return Err(MarkerError::Unexpected {
                    got: other.to_string(),
                    col: 0,
                    detail: "`extra` only supports == / !=".to_string(),
                })
            }
        });
    }

    let l = resolve_operand(lhs, env)?;
    let r = resolve_operand(rhs, env)?;
    let lhs_is_version = matches!(lhs, Operand::Var(n) if MarkerEnv::is_version_typed(n));
    let rhs_is_version = matches!(rhs, Operand::Var(n) if MarkerEnv::is_version_typed(n));

    // `in` / `not in` are substring tests (PEP 508 inheritance from setuptools).
    match op {
        "in" => return Ok(r.contains(l.as_str())),
        "not in" => return Ok(!r.contains(l.as_str())),
        _ => {}
    }

    if op == "~=" || op == "===" {
        if !lhs_is_version && !rhs_is_version {
            return Err(MarkerError::NeedsVersion { op: op.to_string() });
        }
    }

    if lhs_is_version || rhs_is_version {
        return version_cmp(&l, op, &r);
    }

    Ok(match op {
        "==" => l == r,
        "!=" => l != r,
        "<" => l < r,
        "<=" => l <= r,
        ">" => l > r,
        ">=" => l >= r,
        other => {
            return Err(MarkerError::Unexpected {
                got: other.to_string(),
                col: 0,
                detail: "operator not valid for string comparison".to_string(),
            })
        }
    })
}

fn resolve_operand(op: &Operand, env: &MarkerEnv) -> Result<String, MarkerError> {
    match op {
        Operand::Lit(s) => Ok(s.clone()),
        Operand::Var(name) => env
            .lookup(name)
            .map(str::to_string)
            .ok_or_else(|| MarkerError::UnknownEnvVar { name: name.clone() }),
    }
}

fn version_cmp(l: &str, op: &str, r: &str) -> Result<bool, MarkerError> {
    if op == "===" {
        // Arbitrary equality — strict string compare.
        return Ok(l == r);
    }
    if op == "~=" {
        return compatible_release(l, r);
    }
    let lv = pep440::parse(l);
    let rv = pep440::parse(r);
    match (lv, rv) {
        (Some(a), Some(b)) => Ok(match op {
            "==" => a == b,
            "!=" => a != b,
            "<" => a < b,
            "<=" => a <= b,
            ">" => a > b,
            ">=" => a >= b,
            _ => {
                return Err(MarkerError::Unexpected {
                    got: op.to_string(),
                    col: 0,
                    detail: "unsupported version operator".to_string(),
                })
            }
        }),
        _ => Ok(match op {
            "==" => l == r,
            "!=" => l != r,
            "<" => l < r,
            "<=" => l <= r,
            ">" => l > r,
            ">=" => l >= r,
            _ => false,
        }),
    }
}

/// `~= X.Y` is equivalent to `>= X.Y, < X+1.0`. For `~= X.Y.Z`, it's
/// `>= X.Y.Z, < X.(Y+1).0`. See PEP 440 §"Compatible release".
fn compatible_release(left: &str, bound: &str) -> Result<bool, MarkerError> {
    let lv = pep440::parse(left).ok_or_else(|| MarkerError::NeedsVersion {
        op: "~=".to_string(),
    })?;
    let bv = pep440::parse(bound).ok_or_else(|| MarkerError::NeedsVersion {
        op: "~=".to_string(),
    })?;
    if lv < bv {
        return Ok(false);
    }
    // Upper bound: drop the last release segment, bump the preceding one.
    let segs: Vec<&str> = bound.split('.').collect();
    if segs.len() < 2 {
        return Err(MarkerError::NeedsVersion { op: "~=".to_string() });
    }
    let prefix_len = segs.len() - 1;
    let mut bumped: Vec<String> = segs[..prefix_len].iter().map(|s| (*s).to_string()).collect();
    let last_prefix = bumped.last_mut().ok_or_else(|| MarkerError::NeedsVersion {
        op: "~=".to_string(),
    })?;
    let n: u64 = last_prefix.parse().unwrap_or(0);
    *last_prefix = (n + 1).to_string();
    let upper_str = bumped.join(".");
    let upper = pep440::parse(&upper_str).ok_or_else(|| MarkerError::NeedsVersion {
        op: "~=".to_string(),
    })?;
    Ok(lv < upper)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cpython312() -> MarkerEnv {
        MarkerEnv {
            python_version: "3.12".into(),
            python_full_version: "3.12.1".into(),
            implementation_name: "cpython".into(),
            implementation_version: "3.12.1".into(),
            os_name: "posix".into(),
            platform_machine: "arm64".into(),
            platform_release: "23.6.0".into(),
            platform_system: "Darwin".into(),
            platform_version: "Darwin Kernel".into(),
            platform_python_implementation: "CPython".into(),
            sys_platform: "darwin".into(),
            extras: BTreeSet::new(),
        }
    }

    #[test]
    fn string_eq_sys_platform() {
        let env = cpython312();
        assert!(evaluate("sys_platform == 'darwin'", &env).unwrap());
        assert!(!evaluate("sys_platform == 'linux'", &env).unwrap());
    }

    #[test]
    fn version_compare_python_version() {
        let env = cpython312();
        assert!(evaluate("python_version >= '3.10'", &env).unwrap());
        assert!(evaluate("python_version < '4.0'", &env).unwrap());
        assert!(!evaluate("python_version < '3.10'", &env).unwrap());
    }

    #[test]
    fn and_or_precedence() {
        let env = cpython312();
        // and binds tighter than or
        assert!(
            evaluate(
                "sys_platform == 'linux' or sys_platform == 'darwin' and python_version >= '3.10'",
                &env
            )
            .unwrap()
        );
        assert!(
            !evaluate(
                "sys_platform == 'linux' and python_version >= '3.10' or sys_platform == 'win32'",
                &env
            )
            .unwrap()
        );
    }

    #[test]
    fn parens_override_precedence() {
        let env = cpython312();
        assert!(
            evaluate(
                "(sys_platform == 'linux' or sys_platform == 'darwin') and python_version >= '3.10'",
                &env
            )
            .unwrap()
        );
    }

    #[test]
    fn in_and_not_in() {
        let env = cpython312();
        assert!(evaluate("'darwin' in sys_platform", &env).unwrap());
        assert!(evaluate("'linux' not in sys_platform", &env).unwrap());
    }

    #[test]
    fn extra_marker_picks_against_extras_set() {
        let mut env = cpython312();
        env.extras.insert("fast".to_string());
        env.extras.insert("ssl".to_string());
        assert!(evaluate("extra == 'fast'", &env).unwrap());
        assert!(evaluate("extra == 'ssl'", &env).unwrap());
        assert!(!evaluate("extra == 'docs'", &env).unwrap());
        assert!(evaluate("extra != 'docs'", &env).unwrap());
    }

    #[test]
    fn compatible_release() {
        let env = cpython312();
        // ~= 3.10 means >= 3.10, < 4.0 — 3.12 fits
        assert!(evaluate("python_version ~= '3.10'", &env).unwrap());
        // ~= 4.0 means >= 4.0, < 5.0 — 3.12 does not fit
        assert!(!evaluate("python_version ~= '4.0'", &env).unwrap());
    }

    #[test]
    fn arbitrary_equality_triple_eq() {
        let env = cpython312();
        assert!(evaluate("python_full_version === '3.12.1'", &env).unwrap());
        // === is strict string — different surface form does not match.
        assert!(!evaluate("python_full_version === '3.12.1+local'", &env).unwrap());
    }

    #[test]
    fn unknown_env_var_errors() {
        let env = cpython312();
        let err = evaluate("not_a_real_var == 'x'", &env).unwrap_err();
        assert!(matches!(err, MarkerError::UnknownEnvVar { .. }));
    }

    #[test]
    fn pip_style_real_world_markers() {
        let env = cpython312();
        // pip's actual requires_dist line for typing-extensions:
        //   typing-extensions>=4.4.0; python_version < "3.11"
        assert!(!evaluate("python_version < '3.11'", &env).unwrap());

        // Conditional Windows-only dep
        assert!(!evaluate("sys_platform == 'win32'", &env).unwrap());

        // Conditional on extra activation
        let mut env_extra = env.clone();
        env_extra.extras.insert("dev".to_string());
        assert!(evaluate("extra == 'dev'", &env_extra).unwrap());
    }

    #[test]
    fn current_host_smoke() {
        let env = MarkerEnv::current_host();
        assert!(!env.sys_platform.is_empty());
        assert!(!env.platform_machine.is_empty());
        // The host always satisfies python_version >= 3.0
        assert!(evaluate("python_version >= '3.0'", &env).unwrap());
    }
}
