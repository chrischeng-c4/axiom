use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;

pub fn mb_neg(a: MbValue) -> MbValue {
    if super::is_decimal_handle_value(a) {
        return super::super::stdlib::decimal_mod::mb_decimal_neg(a);
    }
    if super::is_fraction_handle_value(a) {
        return super::super::stdlib::fractions_mod::mb_fraction_neg(a);
    }
    // Codegen routes ANY non-Int/Float/Bool unary Neg operand straight to
    // this function (hir_to_mir's Neg lowering), bypassing
    // `mb_dispatch_unaryop`'s usual dunder lookup — so a user-overridden
    // `__neg__` on a numeric-derived class must be invoked here directly,
    // before the subclass-payload unwrap below. `try_get_dunder` is a cheap
    // no-op for non-Instance (raw int/float) operands. (#1030)
    if let Some(method) = super::super::class::try_get_dunder(a, "__neg__") {
        return super::super::class::mb_call_method1(method, a);
    }
    // int/float-SUBCLASS operand unwrap (#1030 — without this `-P(int)(7)`
    // produced None instead of CPython's plain-int `-7`).
    if let Some(v) = super::numeric_subclass_unary_operand(a, "__neg__") {
        return mb_neg(v);
    }
    if let Some(i) = a.as_int() {
        MbValue::from_int(-i)
    } else if let Some(f) = a.as_float() {
        MbValue::from_float(-f)
    } else if let Some(ptr) = a.as_ptr() {
        // -complex → complex with both components negated. (#1256)
        unsafe {
            if let ObjData::Complex(re, im) = (*ptr).data {
                return MbValue::from_ptr(MbObject::new_complex(-re, -im));
            }
            // -bigint → negated big integer. Without this, `-(2**63)` leaks the
            // BigInt pointer bits as a bogus small int (breaks every negative
            // out-of-48-bit literal, e.g. plistlib's signed-int range checks).
            if let ObjData::BigInt(ref big) = (*ptr).data {
                let neg = -big.clone();
                // Re-narrow to an inline int when it fits (e.g. -(2**47)).
                use num_traits::ToPrimitive;
                if let Some(i) = neg.to_i64() {
                    if (-(1i64 << 47)..(1i64 << 47)).contains(&i) {
                        return MbValue::from_int(i);
                    }
                }
                return super::super::bigint_ops::bigint_from_big(neg);
            }
            // -Counter — flip every count, then drop the now-non-positive ones
            // (CPython multiset semantics). `+c` routes through the generic
            // unary dispatcher's Counter arm; `-c` is lowered straight to
            // mb_neg, so it needs the same handling here.
            if super::super::stdlib::collections_mod::is_counter_instance(a) {
                return super::super::stdlib::collections_mod::mb_counter_unary(a, true);
            }
            // -timedelta — negate the exact microsecond total.
            if let Some(us) = super::super::stdlib::datetime_mod::timedelta_total_us(a) {
                return super::super::stdlib::datetime_mod::timedelta_from_us(-us);
            }
            // -NormalDist — flipped mean, fresh object.
            if let Some(r) = super::super::stdlib::statistics_mod::normaldist_neg(a) {
                return r;
            }
        }
        MbValue::none()
    } else {
        MbValue::none()
    }
}
