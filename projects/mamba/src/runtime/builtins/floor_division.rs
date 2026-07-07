use super::super::rc::MbObject;
use super::super::value::MbValue;

/// floor division: a // b
pub fn mb_floordiv(a: MbValue, b: MbValue) -> MbValue {
    let a = super::int_enum_like_value(a).unwrap_or(a);
    let b = super::int_enum_like_value(b).unwrap_or(b);
    // int/float-SUBCLASS operand unwrap (#1030).
    if let Some((na, nb)) = super::numeric_subclass_operands(a, b, "__floordiv__") {
        return mb_floordiv(na, nb);
    }
    if let Some(r) = super::numeric_handle_binop("//", a, b) {
        return r;
    }
    if let Some(r) = super::bigint_numeric_binop("//", a, b) {
        return r;
    }
    // complex doesn't support floor division (CPython TypeError).
    if super::is_complex_obj(a) || super::is_complex_obj(b) {
        super::raise_type_error(format!(
            "unsupported operand type(s) for //: '{}' and '{}'",
            super::value_type_name(a),
            super::value_type_name(b)
        ));
        return MbValue::none();
    }
    // timedelta // timedelta -> int; timedelta // int -> timedelta.
    if let Some(ua) = super::super::stdlib::datetime_mod::timedelta_total_us(a) {
        if let Some(ub) = super::super::stdlib::datetime_mod::timedelta_total_us(b) {
            if ub == 0 {
                super::super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("ZeroDivisionError".to_string())),
                    MbValue::from_ptr(MbObject::new_str("division by zero".to_string())),
                );
                return MbValue::none();
            }
            return MbValue::from_int(super::floor_divmod_i128(ua, ub).0 as i64);
        }
        if let Some(d) = b.as_int() {
            if d == 0 {
                super::super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("ZeroDivisionError".to_string())),
                    MbValue::from_ptr(MbObject::new_str("division by zero".to_string())),
                );
                return MbValue::none();
            }
            return super::super::stdlib::datetime_mod::timedelta_from_us(
                super::floor_divmod_i128(ua, d as i128).0,
            );
        }
    }
    // Integer fast path — Python floor division (round towards -∞)
    if let (Some(ai), Some(bi)) = (a.as_int(), b.as_int()) {
        if bi != 0 {
            let d = ai / bi;
            let r = ai % bi;
            // Adjust: if remainder is non-zero and signs of remainder and divisor differ,
            // subtract 1 to get floor division (rounds towards -∞, not towards 0).
            let floored = if r != 0 && ((r ^ bi) < 0) { d - 1 } else { d };
            return MbValue::from_int(floored);
        }
        // ZeroDivisionError: integer division or modulo by zero
        super::super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("ZeroDivisionError".to_string())),
            MbValue::from_ptr(MbObject::new_str(
                "integer division or modulo by zero".to_string(),
            )),
        );
        return MbValue::none();
    }
    // Float path
    let af = a.as_int().map(|i| i as f64).or(a.as_float());
    let bf = b.as_int().map(|i| i as f64).or(b.as_float());
    match (af, bf) {
        (Some(af), Some(bf)) if bf != 0.0 => MbValue::from_float((af / bf).floor()),
        (Some(_), Some(_)) => {
            // ZeroDivisionError: float floor division by zero
            super::super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("ZeroDivisionError".to_string())),
                MbValue::from_ptr(MbObject::new_str(
                    "float floor division by zero".to_string(),
                )),
            );
            MbValue::none()
        }
        _ => {
            if super::raise_datetime_op_type_error("//", a, b) {
                return MbValue::none();
            }
            MbValue::none()
        }
    }
}
