use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;

/// abs(value) — absolute value.
pub fn mb_abs(val: MbValue) -> MbValue {
    if super::is_decimal_handle_value(val) {
        return super::super::stdlib::decimal_mod::mb_decimal_abs(val);
    }
    if super::is_fraction_handle_value(val) {
        return super::super::stdlib::fractions_mod::mb_fraction_abs(val);
    }
    if let Some(i) = val.as_int() {
        MbValue::from_int(i.abs())
    } else if let Some(f) = val.as_float() {
        MbValue::from_float(f.abs())
    } else if let Some(b) = val.as_bool() {
        MbValue::from_int(if b { 1 } else { 0 })
    } else if let Some(ptr) = val.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Complex(re, im) => {
                    // abs(complex) = hypot(re, im); a finite input overflowing
                    // to inf raises OverflowError (CPython c_abs).
                    let m = re.hypot(*im);
                    if m.is_infinite() && re.is_finite() && im.is_finite() {
                        super::super::exception::mb_raise(
                            MbValue::from_ptr(MbObject::new_str("OverflowError".to_string())),
                            MbValue::from_ptr(MbObject::new_str(
                                "absolute value too large".to_string(),
                            )),
                        );
                        return MbValue::none();
                    }
                    return MbValue::from_float(m);
                }
                ObjData::BigInt(big) => {
                    use num_traits::Signed;
                    return super::super::bigint_ops::bigint_from_big(big.abs());
                }
                ObjData::Instance { class_name, .. } => {
                    // abs(timedelta) — exact microsecond magnitude.
                    if class_name == "datetime.timedelta" {
                        if let Some(us) =
                            super::super::stdlib::datetime_mod::timedelta_total_us(val)
                        {
                            return super::super::stdlib::datetime_mod::timedelta_from_us(us.abs());
                        }
                    }
                    let abs_method = super::super::class::lookup_method(class_name, "__abs__");
                    if !abs_method.is_none() {
                        let method_name =
                            MbValue::from_ptr(MbObject::new_str("__abs__".to_string()));
                        let args = MbValue::from_ptr(MbObject::new_list(vec![]));
                        return super::super::class::mb_call_method(val, method_name, args);
                    }
                }
                _ => {}
            }
        }
        super::super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
            MbValue::from_ptr(MbObject::new_str(format!(
                "bad operand type for abs(): '{}'",
                super::add_operand_type_name(val),
            ))),
        );
        MbValue::none()
    } else {
        // Preserve the legacy None fallback used by existing runtime tests.
        MbValue::from_int(0)
    }
}
