//! The deliberately-tiny assertion expression evaluator.
//!
//! Grammar: `IDENT OP RHS` where `OP ∈ {== != < <= > >=}` and
//! `RHS = NUMBER ['*' IDENT] | IDENT`. Examples:
//!   `recovery_p99 <= 2 * baseline_p99`
//!   `partition_fail > 0`
//!   `errors == 0`
//! Resist growing this into an expression language until a real scenario
//! forces it.

use crate::scenario::interp::VarStore;

/// Evaluate one expression. `Ok(true)` = holds, `Ok(false)` = violated,
/// `Err` = malformed expression or unknown var (a scenario_error, not a
/// failed assertion).
pub fn evaluate(expr: &str, vars: &VarStore) -> Result<bool, String> {
    let tokens: Vec<&str> = expr.split_whitespace().collect();
    let (lhs_name, op, rhs_tokens) = match tokens.as_slice() {
        [lhs, op, rest @ ..] if !rest.is_empty() => (*lhs, *op, rest),
        _ => return Err(format!("malformed expression `{expr}` (want: IDENT OP RHS)")),
    };

    let lhs = vars
        .get_f64(lhs_name)
        .ok_or_else(|| format!("unknown or non-numeric var `{lhs_name}` in `{expr}`"))?;

    let rhs = match rhs_tokens {
        [single] => operand(single, vars, expr)?,
        [scalar, "*", ident] => {
            let s: f64 = scalar
                .parse()
                .map_err(|_| format!("non-numeric scalar `{scalar}` in `{expr}`"))?;
            let v = vars
                .get_f64(ident)
                .ok_or_else(|| format!("unknown or non-numeric var `{ident}` in `{expr}`"))?;
            s * v
        }
        _ => return Err(format!("unsupported RHS in `{expr}` (want: NUMBER ['*' IDENT] | IDENT)")),
    };

    match op {
        "==" => Ok(lhs == rhs),
        "!=" => Ok(lhs != rhs),
        "<" => Ok(lhs < rhs),
        "<=" => Ok(lhs <= rhs),
        ">" => Ok(lhs > rhs),
        ">=" => Ok(lhs >= rhs),
        other => Err(format!("unsupported op `{other}` in `{expr}`")),
    }
}

fn operand(token: &str, vars: &VarStore, expr: &str) -> Result<f64, String> {
    if let Ok(n) = token.parse::<f64>() {
        return Ok(n);
    }
    vars.get_f64(token)
        .ok_or_else(|| format!("unknown or non-numeric var `{token}` in `{expr}`"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn vars() -> VarStore {
        let mut v = VarStore::new();
        v.set("baseline_p99", json!(10.0));
        v.set("recovery_p99", json!(18.0));
        v.set("partition_fail", json!(3));
        v.set("errors", json!(0));
        v
    }

    #[test]
    fn scalar_multiply_rhs() {
        assert!(evaluate("recovery_p99 <= 2 * baseline_p99", &vars()).unwrap());
        assert!(!evaluate("recovery_p99 <= 1 * baseline_p99", &vars()).unwrap());
    }

    #[test]
    fn literal_and_var_rhs() {
        assert!(evaluate("partition_fail > 0", &vars()).unwrap());
        assert!(evaluate("errors == 0", &vars()).unwrap());
        assert!(evaluate("baseline_p99 < recovery_p99", &vars()).unwrap());
    }

    #[test]
    fn unknown_var_is_error_not_false() {
        assert!(evaluate("missing > 0", &vars()).is_err());
        assert!(evaluate("errors <= 2 * missing", &vars()).is_err());
    }

    #[test]
    fn malformed_is_error() {
        assert!(evaluate("just_one_token", &vars()).is_err());
        assert!(evaluate("a ~~ b", &vars()).is_err());
        assert!(evaluate("a <= 2 * b * c", &vars()).is_err());
    }
}
