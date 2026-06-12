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

pub fn register() {
    let mut attrs = HashMap::new();
    let dispatchers: Vec<(&str, usize)> = vec![
        ("Number", dispatch_Number as usize),
        ("Complex", dispatch_Complex as usize),
        ("Real", dispatch_Real as usize),
        ("Rational", dispatch_Rational as usize),
        ("Integral", dispatch_Integral as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
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

    fn extract_name(d: MbValue) -> String {
        unsafe {
            let ObjData::Dict(ref lock) = (*d.as_ptr().unwrap()).data else { panic!("expected Dict"); };
            let m = lock.read().unwrap();
            let ObjData::Str(ref s) = (*m["__name__"].as_ptr().unwrap()).data else { panic!("expected Str"); };
            s.clone()
        }
    }

    fn extract_abstract(d: MbValue) -> bool {
        unsafe {
            let ObjData::Dict(ref lock) = (*d.as_ptr().unwrap()).data else { panic!("expected Dict"); };
            lock.read().unwrap()["__abstract__"].as_bool().unwrap()
        }
    }

    #[test]
    fn test_numbers_abc_hierarchy() {
        for (val, expected) in [
            (mb_numbers_Number(), "Number"),
            (mb_numbers_Complex(), "Complex"),
            (mb_numbers_Real(), "Real"),
            (mb_numbers_Rational(), "Rational"),
            (mb_numbers_Integral(), "Integral"),
        ] {
            assert!(extract_abstract(val));
            assert_eq!(extract_name(val), expected);
        }
    }
}
