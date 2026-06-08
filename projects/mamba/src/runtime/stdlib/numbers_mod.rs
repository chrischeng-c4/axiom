use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// numbers module for Mamba (mamba-stdlib).
use std::collections::HashMap;

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
    let dict = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut map = lock.write().unwrap();
            map.insert(
                "__name__".into(),
                MbValue::from_ptr(MbObject::new_str("Number".to_string())),
            );
            map.insert("__abstract__".into(), MbValue::from_bool(true));
        }
    }
    MbValue::from_ptr(dict)
}

pub fn mb_numbers_Complex() -> MbValue {
    let dict = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut map = lock.write().unwrap();
            map.insert(
                "__name__".into(),
                MbValue::from_ptr(MbObject::new_str("Complex".to_string())),
            );
            map.insert("__abstract__".into(), MbValue::from_bool(true));
        }
    }
    MbValue::from_ptr(dict)
}

pub fn mb_numbers_Real() -> MbValue {
    let dict = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut map = lock.write().unwrap();
            map.insert(
                "__name__".into(),
                MbValue::from_ptr(MbObject::new_str("Real".to_string())),
            );
            map.insert("__abstract__".into(), MbValue::from_bool(true));
        }
    }
    MbValue::from_ptr(dict)
}

pub fn mb_numbers_Rational() -> MbValue {
    let dict = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut map = lock.write().unwrap();
            map.insert(
                "__name__".into(),
                MbValue::from_ptr(MbObject::new_str("Rational".to_string())),
            );
            map.insert("__abstract__".into(), MbValue::from_bool(true));
        }
    }
    MbValue::from_ptr(dict)
}

pub fn mb_numbers_Integral() -> MbValue {
    let dict = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut map = lock.write().unwrap();
            map.insert(
                "__name__".into(),
                MbValue::from_ptr(MbObject::new_str("Integral".to_string())),
            );
            map.insert("__abstract__".into(), MbValue::from_bool(true));
        }
    }
    MbValue::from_ptr(dict)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn extract_name(d: MbValue) -> String {
        unsafe {
            let ObjData::Dict(ref lock) = (*d.as_ptr().unwrap()).data else {
                panic!("expected Dict");
            };
            let m = lock.read().unwrap();
            let ObjData::Str(ref s) = (*m["__name__"].as_ptr().unwrap()).data else {
                panic!("expected Str");
            };
            s.clone()
        }
    }

    fn extract_abstract(d: MbValue) -> bool {
        unsafe {
            let ObjData::Dict(ref lock) = (*d.as_ptr().unwrap()).data else {
                panic!("expected Dict");
            };
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
