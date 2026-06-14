/// numbers module for Mamba (mamba-stdlib).
use std::collections::HashMap;
use super::super::value::MbValue;
use super::super::rc::{MbObject, ObjData};

macro_rules! dispatch_nullary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
            $fn()
        }
    };
}

dispatch_nullary!(dispatch_Number, mb_numbers_Number);
dispatch_nullary!(dispatch_Complex, mb_numbers_Complex);
dispatch_nullary!(dispatch_Real, mb_numbers_Real);
dispatch_nullary!(dispatch_Rational, mb_numbers_Rational);
dispatch_nullary!(dispatch_Integral, mb_numbers_Integral);

thread_local! {
    /// Function-pointer address → numeric-tower rank for the five numbers ABCs.
    /// Rank widens from Integral (4, narrowest) to Number (0, widest): a value
    /// satisfies `isinstance(v, ABC)` when its own rank is ≥ the ABC's rank.
    static NUMBERS_ABC_RANKS: std::cell::RefCell<std::collections::HashMap<u64, u8>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

/// Numeric-tower rank for a numbers ABC function pointer, if `addr` names one
/// of `numbers.{Number,Complex,Real,Rational,Integral}`.
pub fn numbers_abc_rank(addr: u64) -> Option<u8> {
    NUMBERS_ABC_RANKS.with(|m| m.borrow().get(&addr).copied())
}

pub fn register() {
    let mut attrs = HashMap::new();
    // (name, dispatcher, tower rank): Integral is narrowest (4), Number widest (0).
    let dispatchers: Vec<(&str, usize, u8)> = vec![
        ("Number", dispatch_Number as usize, 0),
        ("Complex", dispatch_Complex as usize, 1),
        ("Real", dispatch_Real as usize, 2),
        ("Rational", dispatch_Rational as usize, 3),
        ("Integral", dispatch_Integral as usize, 4),
    ];
    for (name, addr, rank) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
        NUMBERS_ABC_RANKS.with(|m| {
            m.borrow_mut().insert(addr as u64, rank);
        });
    }
    super::register_module("numbers", attrs);
}

pub fn mb_numbers_Number() -> MbValue {
    // The numeric-tower ABCs cannot be instantiated (CPython ABCMeta).
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(
            "Can't instantiate abstract class Number".to_string(),
        )),
    );
    MbValue::none()
}

pub fn mb_numbers_Complex() -> MbValue {
    // The numeric-tower ABCs cannot be instantiated (CPython ABCMeta).
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(
            "Can't instantiate abstract class Complex with abstract methods __abs__, __add__, __complex__, __eq__, __mul__, __neg__, __pos__, __pow__, __radd__, __rmul__, __rpow__, __rtruediv__, __truediv__, conjugate, imag, real".to_string(),
        )),
    );
    MbValue::none()
}

pub fn mb_numbers_Real() -> MbValue {
    // The numeric-tower ABCs cannot be instantiated (CPython ABCMeta).
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(
            "Can't instantiate abstract class Real with abstract methods __abs__, __add__, __ceil__, __eq__, __float__, __floor__, __floordiv__, __le__, __lt__, __mod__, __mul__, __neg__, __pos__, __pow__, __radd__, __rfloordiv__, __rmod__, __rmul__, __round__, __rpow__, __rtruediv__, __truediv__, __trunc__, conjugate, imag, real".to_string(),
        )),
    );
    MbValue::none()
}

pub fn mb_numbers_Rational() -> MbValue {
    // The numeric-tower ABCs cannot be instantiated (CPython ABCMeta).
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(
            "Can't instantiate abstract class Rational with abstract methods denominator, numerator".to_string(),
        )),
    );
    MbValue::none()
}

pub fn mb_numbers_Integral() -> MbValue {
    // The numeric-tower ABCs cannot be instantiated (CPython ABCMeta).
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(
            "Can't instantiate abstract class Integral with abstract methods __abs__, __add__, __and__, __ceil__, __eq__, __floor__, __index__, __int__, __invert__, __le__, __lshift__, __lt__, __mod__, __mul__, __neg__, __or__, __pos__, __pow__, __rshift__, __trunc__, __xor__".to_string(),
        )),
    );
    MbValue::none()
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_numbers_abcs_raise_on_instantiation() {
        // CPython: the numeric-tower ABCs cannot be instantiated; each call
        // raises TypeError and returns None. Calls run one at a time so each
        // pending exception is observed before the next raise.
        let cases: [(fn() -> MbValue, &str); 5] = [
            (mb_numbers_Number, "Number"),
            (mb_numbers_Complex, "Complex"),
            (mb_numbers_Real, "Real"),
            (mb_numbers_Rational, "Rational"),
            (mb_numbers_Integral, "Integral"),
        ];
        for (ctor, expected) in cases {
            let val = ctor();
            assert!(val.is_none(), "for {expected}");
            let exc = super::super::super::exception::current_exception_type();
            assert_eq!(exc.as_deref(), Some("TypeError"), "for {expected}");
            super::super::super::exception::mb_clear_exception();
        }
    }
}
