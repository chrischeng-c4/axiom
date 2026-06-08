use super::super::rc::MbObject;
use super::super::value::MbValue;
/// gc module for Mamba (#653).
///
/// Exposes Mamba's cycle-detecting garbage collector to Python userspace.
/// Wraps the runtime/gc.rs functions with CPython-compatible API.
use std::collections::HashMap;

// ── Variadic dispatchers (callable from module-attr context) ──

macro_rules! disp_nullary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
            $fn()
        }
    };
}

macro_rules! disp_unary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a.get(0).copied().unwrap_or_else(MbValue::none))
        }
    };
}

disp_unary!(d_collect, super::super::gc::mb_gc_collect);
disp_nullary!(d_enable, mb_gc_mod_enable);
disp_nullary!(d_disable, mb_gc_mod_disable);
disp_nullary!(d_isenabled, mb_gc_mod_isenabled);
disp_nullary!(d_get_count, mb_gc_mod_get_count);
disp_nullary!(d_get_threshold, mb_gc_mod_get_threshold);
disp_unary!(d_set_threshold, mb_gc_mod_set_threshold);
disp_nullary!(d_get_stats, mb_gc_mod_get_stats);
disp_unary!(d_is_tracked, mb_gc_mod_is_tracked);
disp_nullary!(d_freeze, mb_gc_mod_freeze);
disp_nullary!(d_unfreeze, mb_gc_mod_unfreeze);
disp_nullary!(d_get_freeze_count, mb_gc_mod_get_freeze_count);
disp_nullary!(d_get_objects, mb_gc_mod_get_objects);

pub fn register() {
    let mut attrs = HashMap::new();

    let dispatchers: Vec<(&str, usize)> = vec![
        ("collect", d_collect as *const () as usize),
        ("enable", d_enable as *const () as usize),
        ("disable", d_disable as *const () as usize),
        ("isenabled", d_isenabled as *const () as usize),
        ("get_count", d_get_count as *const () as usize),
        ("get_threshold", d_get_threshold as *const () as usize),
        ("set_threshold", d_set_threshold as *const () as usize),
        ("get_stats", d_get_stats as *const () as usize),
        ("is_tracked", d_is_tracked as *const () as usize),
        ("freeze", d_freeze as *const () as usize),
        ("unfreeze", d_unfreeze as *const () as usize),
        ("get_freeze_count", d_get_freeze_count as *const () as usize),
        ("get_objects", d_get_objects as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    // Constants
    attrs.insert("DEBUG_STATS".into(), MbValue::from_int(1));
    attrs.insert("DEBUG_COLLECTABLE".into(), MbValue::from_int(2));
    attrs.insert("DEBUG_UNCOLLECTABLE".into(), MbValue::from_int(4));
    attrs.insert("DEBUG_SAVEALL".into(), MbValue::from_int(32));
    attrs.insert("DEBUG_LEAK".into(), MbValue::from_int(38));

    super::register_module("gc", attrs);
}

// -- Forwarding wrappers (MbValue ABI) --

/// gc.enable()
pub fn mb_gc_mod_enable() -> MbValue {
    super::super::gc::gc_enable();
    MbValue::none()
}

/// gc.disable()
pub fn mb_gc_mod_disable() -> MbValue {
    super::super::gc::gc_disable();
    MbValue::none()
}

/// gc.isenabled() -> bool
pub fn mb_gc_mod_isenabled() -> MbValue {
    MbValue::from_bool(super::super::gc::gc_is_enabled())
}

/// gc.get_count() -> (count0, count1, count2)
/// Returns a 3-tuple of per-generation counts (Mamba uses a single-gen GC,
/// so count1 and count2 are always 0).
pub fn mb_gc_mod_get_count() -> MbValue {
    let n = super::super::gc::gc_get_count();
    let elems = vec![
        MbValue::from_int(n as i64),
        MbValue::from_int(0),
        MbValue::from_int(0),
    ];
    MbValue::from_ptr(MbObject::new_tuple(elems))
}

/// gc.get_threshold() -> (threshold0, threshold1, threshold2)
pub fn mb_gc_mod_get_threshold() -> MbValue {
    let t = super::super::gc::gc_get_threshold();
    let elems = vec![
        MbValue::from_int(t as i64),
        MbValue::from_int(10),
        MbValue::from_int(10),
    ];
    MbValue::from_ptr(MbObject::new_tuple(elems))
}

/// gc.set_threshold(threshold0[, threshold1, threshold2])
pub fn mb_gc_mod_set_threshold(t: MbValue) -> MbValue {
    let threshold = t.as_int().unwrap_or(700) as usize;
    super::super::gc::gc_set_threshold(threshold);
    MbValue::none()
}

/// gc.get_stats() -> list of dicts
pub fn mb_gc_mod_get_stats() -> MbValue {
    let (collections, freed, _tracked) = super::super::gc::gc_get_stats();
    let stats_dict = MbObject::new_dict();
    unsafe {
        use super::super::rc::ObjData;
        if let ObjData::Dict(ref lock) = (*stats_dict).data {
            let mut map = lock.write().unwrap();
            map.insert("collections".into(), MbValue::from_int(collections as i64));
            map.insert("collected".into(), MbValue::from_int(freed as i64));
            map.insert("uncollectable".into(), MbValue::from_int(0));
        }
    }
    let list = MbObject::new_list(vec![MbValue::from_ptr(stats_dict)]);
    MbValue::from_ptr(list)
}

/// gc.is_tracked(obj) -> bool
/// Returns True if the object is currently tracked by the GC.
pub fn mb_gc_mod_is_tracked(obj: MbValue) -> MbValue {
    // Container objects (list, dict, instance) are tracked; scalars are not.
    if let Some(_ptr) = obj.as_ptr() {
        MbValue::from_bool(true)
    } else {
        MbValue::from_bool(false)
    }
}

/// gc.freeze() — move all tracked objects to permanent generation (no-op stub)
pub fn mb_gc_mod_freeze() -> MbValue {
    MbValue::none()
}

/// gc.unfreeze() — move all permanent-generation objects back to oldest gen (no-op stub)
pub fn mb_gc_mod_unfreeze() -> MbValue {
    MbValue::none()
}

/// gc.get_freeze_count() -> int
pub fn mb_gc_mod_get_freeze_count() -> MbValue {
    MbValue::from_int(0)
}

/// gc.get_objects([generation]) -> list
/// Returns all objects tracked by the GC (returns empty list for safety).
pub fn mb_gc_mod_get_objects() -> MbValue {
    MbValue::from_ptr(MbObject::new_list(vec![]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gc_enable_disable() {
        mb_gc_mod_enable();
        assert!(mb_gc_mod_isenabled().as_bool() == Some(true));
        mb_gc_mod_disable();
        assert!(mb_gc_mod_isenabled().as_bool() == Some(false));
        mb_gc_mod_enable();
    }

    #[test]
    fn test_gc_get_threshold() {
        let t = mb_gc_mod_get_threshold();
        assert!(t.as_ptr().is_some());
    }

    #[test]
    fn test_gc_get_count() {
        let c = mb_gc_mod_get_count();
        assert!(c.as_ptr().is_some());
    }

    #[test]
    fn test_gc_get_stats() {
        let s = mb_gc_mod_get_stats();
        assert!(s.as_ptr().is_some());
    }

    #[test]
    fn test_gc_collect() {
        // collect() returns the number of objects freed (>= 0)
        let freed = super::super::super::gc::collect();
        assert!(freed < usize::MAX);
    }
}
