/// datetime module for Mamba — backed by `chrono` crate.
///
/// Provides: datetime.now, datetime.new, date.today, timedelta.new,
///           datetime.strftime, datetime.timestamp, datetime.fromtimestamp

use std::collections::HashMap;
use rustc_hash::FxHashMap;
use indexmap::IndexMap;
use super::super::dict_ops::DictKey;
use chrono::{Datelike, Timelike, NaiveDate, NaiveDateTime, Utc, DateTime};
use super::super::value::MbValue;
use super::super::rc::{MbObject, ObjData};

fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data { Some(s.clone()) } else { None }
    })
}

fn is_dict(val: MbValue) -> bool {
    val.as_ptr().is_some_and(|ptr| unsafe {
        matches!((*ptr).data, ObjData::Dict(_))
    })
}

fn raise_type_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

fn raise_value_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

fn raise_not_implemented_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("NotImplementedError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

/// Build a `datetime.time` Instance with hour/minute/second/microsecond fields.
fn build_time_instance(h: i64, m: i64, s: i64, us: i64) -> MbValue {
    let mut fields = FxHashMap::default();
    fields.insert("hour".into(), MbValue::from_int(h));
    fields.insert("minute".into(), MbValue::from_int(m));
    fields.insert("second".into(), MbValue::from_int(s));
    fields.insert("microsecond".into(), MbValue::from_int(us));
    let obj = Box::new(super::super::rc::MbObject {
        header: super::super::rc::MbObjectHeader {
            rc: std::sync::atomic::AtomicU32::new(1),
            kind: super::super::rc::ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "datetime.time".to_string(),
            fields: crate::runtime::rc::MbRwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

/// Build a `datetime.timezone` Instance carrying an offset (seconds) and name.
fn build_timezone_instance(offset_seconds: i64, name: Option<String>) -> MbValue {
    let mut fields = FxHashMap::default();
    fields.insert("_offset_seconds".into(), MbValue::from_int(offset_seconds));
    if let Some(n) = name {
        fields.insert("_name".into(), MbValue::from_ptr(MbObject::new_str(n)));
    }
    let obj = Box::new(super::super::rc::MbObject {
        header: super::super::rc::MbObjectHeader {
            rc: std::sync::atomic::AtomicU32::new(1),
            kind: super::super::rc::ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "datetime.timezone".to_string(),
            fields: crate::runtime::rc::MbRwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

// ── Dispatch wrappers: native ABI ──

unsafe extern "C" fn dispatch_now(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    mb_datetime_now()
}

unsafe extern "C" fn dispatch_new(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    // mb_datetime_new expects a list wrapper — pack positional args into one.
    let args_list = MbValue::from_ptr(MbObject::new_list(a.to_vec()));
    mb_datetime_new(args_list)
}

unsafe extern "C" fn dispatch_today(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    mb_date_today()
}

unsafe extern "C" fn dispatch_timedelta(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let args_list = MbValue::from_ptr(MbObject::new_list(a.to_vec()));
    mb_timedelta_new(args_list)
}

unsafe extern "C" fn dispatch_strftime(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_datetime_strftime(
        a.get(0).copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
    )
}

unsafe extern "C" fn dispatch_timestamp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_datetime_timestamp(a.get(0).copied().unwrap_or_else(MbValue::none))
}

unsafe extern "C" fn dispatch_fromtimestamp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_datetime_fromtimestamp(a.get(0).copied().unwrap_or_else(MbValue::none))
}

unsafe extern "C" fn dispatch_isoformat(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_datetime_isoformat(a.get(0).copied().unwrap_or_else(MbValue::none))
}

unsafe extern "C" fn dispatch_fromisoformat(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_datetime_fromisoformat(a.get(0).copied().unwrap_or_else(MbValue::none))
}

unsafe extern "C" fn dispatch_date_isoformat(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_date_isoformat(a.get(0).copied().unwrap_or_else(MbValue::none))
}

unsafe extern "C" fn dispatch_strptime(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_datetime_strptime(
        a.get(0).copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
    )
}

unsafe extern "C" fn dispatch_combine(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_datetime_combine(
        a.get(0).copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
    )
}

/// `datetime.time(hour=0, minute=0, second=0, microsecond=0, tzinfo=None)`.
/// CPython validates each component range and raises `ValueError` when any
/// is out of bounds. Keyword args arrive as a trailing dict positional in
/// mamba's current call lowering; ranges-only validation is performed here.
unsafe extern "C" fn dispatch_time(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    // Read positional ints, skipping any trailing kwargs dict.
    let pos: Vec<i64> = a.iter().filter_map(|v| v.as_int()).collect();
    let hour = *pos.first().unwrap_or(&0);
    let minute = *pos.get(1).unwrap_or(&0);
    let second = *pos.get(2).unwrap_or(&0);
    let micro = *pos.get(3).unwrap_or(&0);
    if !(0..=23).contains(&hour) {
        return raise_value_error(&format!("hour must be in 0..23, not {hour}"));
    }
    if !(0..=59).contains(&minute) {
        return raise_value_error(&format!("minute must be in 0..59, not {minute}"));
    }
    if !(0..=59).contains(&second) {
        return raise_value_error(&format!("second must be in 0..59, not {second}"));
    }
    if !(0..=999_999).contains(&micro) {
        return raise_value_error(&format!("microsecond must be in 0..999999, not {micro}"));
    }
    build_time_instance(hour, minute, second, micro)
}

/// `datetime.timezone(offset, name=None)` where `offset` is a `timedelta`.
/// CPython requires `-timedelta(hours=24) < offset < timedelta(hours=24)`.
unsafe extern "C" fn dispatch_timezone(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let offset = a.first().copied().unwrap_or_else(MbValue::none);
    // Pull days/seconds out of a timedelta Instance argument.
    let (days, secs) = offset.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { ref class_name, ref fields } = (*ptr).data {
            if class_name == "datetime.timedelta" {
                let f = fields.read().ok()?;
                return Some((
                    f.get("days").and_then(|v| v.as_int()).unwrap_or(0),
                    f.get("seconds").and_then(|v| v.as_int()).unwrap_or(0),
                ));
            }
        }
        None
    }).unwrap_or((0, 0));
    let total_seconds = days * 86_400 + secs;
    if total_seconds <= -86_400 || total_seconds >= 86_400 {
        return raise_value_error(
            "offset must be a timedelta strictly between -timedelta(hours=24) and timedelta(hours=24)",
        );
    }
    let name = a.get(1).copied().and_then(extract_str);
    build_timezone_instance(total_seconds, name)
}

/// `datetime.tzinfo()` — bare abstract base. Constructed instances exist but
/// their query methods are abstract in CPython; that dispatch lives in
/// `class.rs`. Here we only need a constructible callable so `tzinfo()` and
/// `class X(datetime.tzinfo)` resolve.
unsafe extern "C" fn dispatch_tzinfo(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    let fields = FxHashMap::default();
    let obj = Box::new(super::super::rc::MbObject {
        header: super::super::rc::MbObjectHeader {
            rc: std::sync::atomic::AtomicU32::new(1),
            kind: super::super::rc::ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "datetime.tzinfo".to_string(),
            fields: crate::runtime::rc::MbRwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

// A bare `datetime.tzinfo` is the abstract base: its query methods are
// unimplemented and must raise `NotImplementedError` when called on a plain
// `tzinfo()` (CPython). Concrete subclasses override these in Python, so user
// subclasses never reach these native bodies. Fixed `(self, dt)` arity.
unsafe extern "C" fn tzinfo_method_tzname(_self_: MbValue, _dt: MbValue) -> MbValue {
    raise_not_implemented_error("tzinfo subclass must override tzname()")
}

unsafe extern "C" fn tzinfo_method_utcoffset(_self_: MbValue, _dt: MbValue) -> MbValue {
    raise_not_implemented_error("tzinfo subclass must override utcoffset()")
}

unsafe extern "C" fn tzinfo_method_dst(_self_: MbValue, _dt: MbValue) -> MbValue {
    raise_not_implemented_error("tzinfo subclass must override dst()")
}

/// Register the datetime module.
pub fn register() {
    let mut attrs = HashMap::new();
    // Python: `from datetime import datetime` → `datetime` is a class that
    // constructs a datetime. Register `datetime` and `date` as aliases for
    // `new` / `today` so the common `datetime(y, m, d)` / `date.today()`
    // idioms work against the module-as-dict dispatch.
    let dispatchers: [(&str, usize); 17] = [
        ("now", dispatch_now as *const () as usize),
        ("new", dispatch_new as *const () as usize),
        ("datetime", dispatch_new as *const () as usize),
        ("today", dispatch_today as *const () as usize),
        ("timedelta", dispatch_timedelta as *const () as usize),
        ("strftime", dispatch_strftime as *const () as usize),
        ("strptime", dispatch_strptime as *const () as usize),
        ("combine", dispatch_combine as *const () as usize),
        ("timestamp", dispatch_timestamp as *const () as usize),
        ("fromtimestamp", dispatch_fromtimestamp as *const () as usize),
        ("date", dispatch_date as *const () as usize),
        ("isoformat", dispatch_isoformat as *const () as usize),
        ("fromisoformat", dispatch_fromisoformat as *const () as usize),
        ("date_isoformat", dispatch_date_isoformat as *const () as usize),
        ("time", dispatch_time as *const () as usize),
        ("timezone", dispatch_timezone as *const () as usize),
        ("tzinfo", dispatch_tzinfo as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }
    // Module constants. CPython exposes MINYEAR/MAXYEAR as ints and UTC as the
    // canonical UTC timezone singleton (alias of timezone.utc).
    attrs.insert("MINYEAR".to_string(), MbValue::from_int(1));
    attrs.insert("MAXYEAR".to_string(), MbValue::from_int(9999));
    attrs.insert("UTC".to_string(), build_timezone_instance(0, Some("UTC".to_string())));

    // Bridge the `date` / `datetime` constructor funcs -> their class name so
    // accessing a registered classmethod on the class object
    // (`datetime.date.today`, `datetime.datetime.now`, `.combine`, `.strptime`,
    // `.fromisoformat`) resolves to a callable unbound method via mb_getattr's
    // func->native-class method bridge (which looks the func addr up in
    // NATIVE_TYPE_NAMES, then lookup_method in the table mb_class_register
    // populates below). Without this `callable(datetime.date.today)` is False.
    // The unqualified class names "date"/"datetime" do not collide with the
    // fully-qualified "datetime.datetime"/"datetime.timedelta" instance
    // dispatch in class.rs.
    super::super::module::NATIVE_TYPE_NAMES.with(|m| {
        let mut map = m.borrow_mut();
        map.insert(dispatch_date as *const () as usize as u64, "date".to_string());
        map.insert(dispatch_new as *const () as usize as u64, "datetime".to_string());
    });
    // `date` classmethods: today(), fromisoformat(), fromtimestamp(),
    // fromordinal(). `datetime` inherits date's plus now()/combine()/strptime().
    {
        let mut date_methods: HashMap<String, MbValue> = HashMap::new();
        date_methods.insert("today".to_string(), MbValue::from_func(dispatch_today as *const () as usize));
        date_methods.insert("fromisoformat".to_string(), MbValue::from_func(dispatch_fromisoformat as *const () as usize));
        date_methods.insert("fromtimestamp".to_string(), MbValue::from_func(dispatch_fromtimestamp as *const () as usize));
        date_methods.insert("isoformat".to_string(), MbValue::from_func(dispatch_date_isoformat as *const () as usize));
        date_methods.insert("strftime".to_string(), MbValue::from_func(dispatch_strftime as *const () as usize));
        super::super::class::mb_class_register("date", vec![], date_methods);

        let mut dt_methods: HashMap<String, MbValue> = HashMap::new();
        dt_methods.insert("now".to_string(), MbValue::from_func(dispatch_now as *const () as usize));
        dt_methods.insert("today".to_string(), MbValue::from_func(dispatch_today as *const () as usize));
        dt_methods.insert("combine".to_string(), MbValue::from_func(dispatch_combine as *const () as usize));
        dt_methods.insert("strptime".to_string(), MbValue::from_func(dispatch_strptime as *const () as usize));
        dt_methods.insert("fromisoformat".to_string(), MbValue::from_func(dispatch_fromisoformat as *const () as usize));
        dt_methods.insert("fromtimestamp".to_string(), MbValue::from_func(dispatch_fromtimestamp as *const () as usize));
        dt_methods.insert("isoformat".to_string(), MbValue::from_func(dispatch_isoformat as *const () as usize));
        dt_methods.insert("strftime".to_string(), MbValue::from_func(dispatch_strftime as *const () as usize));
        dt_methods.insert("timestamp".to_string(), MbValue::from_func(dispatch_timestamp as *const () as usize));
        super::super::class::mb_class_register("datetime", vec![], dt_methods);
    }

    // Instance methods, keyed by the *qualified* runtime class names that
    // `build_datetime_dict` / `build_time_instance` stamp onto constructed
    // values. `mb_call_method` falls through to `lookup_method(class_name, …)`
    // for any name its hardcoded datetime arm does not handle, so registering
    // these here makes `inst.method()` resolve. Variadic registration forces
    // the uniform `(self, args_list)` shape for methods that take optional
    // positional/keyword args.
    {
        // datetime / date instances (both carry class_name "datetime.datetime").
        let mut dt_inst: HashMap<String, MbValue> = HashMap::new();
        dt_inst.insert("isoformat".into(),    MbValue::from_func(dt_method_isoformat as *const () as usize));
        dt_inst.insert("ctime".into(),        MbValue::from_func(dt_method_ctime as *const () as usize));
        dt_inst.insert("weekday".into(),      MbValue::from_func(dt_method_weekday as *const () as usize));
        dt_inst.insert("isoweekday".into(),   MbValue::from_func(dt_method_isoweekday as *const () as usize));
        dt_inst.insert("isocalendar".into(),  MbValue::from_func(dt_method_isocalendar as *const () as usize));
        dt_inst.insert("timetuple".into(),    MbValue::from_func(dt_method_timetuple as *const () as usize));
        dt_inst.insert("replace".into(),      MbValue::from_func(dt_method_replace as *const () as usize));
        dt_inst.insert("time".into(),         MbValue::from_func(dt_method_time as *const () as usize));
        dt_inst.insert("timetz".into(),       MbValue::from_func(dt_method_timetz as *const () as usize));
        dt_inst.insert("__eq__".into(),       MbValue::from_func(dt_method_eq as *const () as usize));
        dt_inst.insert("__hash__".into(),     MbValue::from_func(dt_method_hash as *const () as usize));
        super::super::class::mb_class_register("datetime.datetime", vec![], dt_inst);

        // datetime.time instances.
        let mut time_inst: HashMap<String, MbValue> = HashMap::new();
        time_inst.insert("isoformat".into(),  MbValue::from_func(time_method_isoformat as *const () as usize));
        time_inst.insert("replace".into(),    MbValue::from_func(time_method_replace as *const () as usize));
        time_inst.insert("__eq__".into(),     MbValue::from_func(time_method_eq as *const () as usize));
        time_inst.insert("__hash__".into(),   MbValue::from_func(time_method_hash as *const () as usize));
        super::super::class::mb_class_register("datetime.time", vec![], time_inst);

        // datetime.tzinfo (abstract base) instances. The query methods are
        // unimplemented on a bare `tzinfo()`; each raises NotImplementedError.
        // Fixed `(self, dt)` arity — NOT registered variadic.
        let mut tzinfo_inst: HashMap<String, MbValue> = HashMap::new();
        tzinfo_inst.insert("tzname".into(),    MbValue::from_func(tzinfo_method_tzname as *const () as usize));
        tzinfo_inst.insert("utcoffset".into(), MbValue::from_func(tzinfo_method_utcoffset as *const () as usize));
        tzinfo_inst.insert("dst".into(),       MbValue::from_func(tzinfo_method_dst as *const () as usize));
        super::super::class::mb_class_register("datetime.tzinfo", vec![], tzinfo_inst);

        // Methods that accept optional positional/keyword args must run with the
        // variadic `(self, args_list)` shape. `__eq__` (self, other) and
        // `__hash__` (self) keep their fixed arity and are NOT variadic.
        for addr in [
            dt_method_isoformat as *const () as usize,
            dt_method_replace as *const () as usize,
            time_method_isoformat as *const () as usize,
            time_method_replace as *const () as usize,
        ] {
            super::super::module::register_variadic_func(addr as u64);
        }
    }

    super::register_module("datetime", attrs);
}

/// Dispatch for `date(year, month, day)` — constructs a date-only value.
///
/// Date and datetime instances currently share the `datetime.datetime`
/// class name (both built by `build_datetime_dict`). To let instance methods
/// such as `isoformat()` distinguish a pure date from a full datetime, tag the
/// constructed value with a private `_is_date` marker field.
unsafe extern "C" fn dispatch_date(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let args_list = MbValue::from_ptr(MbObject::new_list(a.to_vec()));
    let val = mb_datetime_new(args_list);
    if let Some(ptr) = val.as_ptr() {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            if let Ok(mut f) = fields.write() {
                f.insert("_is_date".into(), MbValue::from_bool(true));
            }
        }
    }
    val
}

// ── Instance-method helpers (bound `self`-first ABI) ──
//
// Methods registered via `mb_class_register` for the `datetime.datetime` and
// `datetime.time` class names are dispatched by `runtime::class::mb_call_method`
// with the per-value SystemV ABI (self [, args]). Variadic methods receive
// `(self, args_list)` where `args_list` collects positional/keyword call args
// (mamba folds keyword args into a single trailing dict positional).

/// Three-letter English weekday/month abbreviations (CPython ctime/asctime
/// use the C locale, never the host locale).
const WD_ABBR: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
const MO_ABBR: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun",
    "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// Read an integer field directly from an Instance's field map.
fn inst_int(val: MbValue, name: &str, def: i64) -> i64 {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            fields.read().ok()
                .and_then(|f| f.get(name).copied())
                .and_then(|v| v.as_int())
        } else { None }
    }).unwrap_or(def)
}

/// True iff the Instance carries a truthy `_is_date` marker (set by
/// `dispatch_date`).
fn inst_is_date(val: MbValue) -> bool {
    val.as_ptr().map(|ptr| unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            fields.read().ok()
                .and_then(|f| f.get("_is_date").copied())
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
        } else { false }
    }).unwrap_or(false)
}

/// Extract the trailing kwargs dict from a variadic method's `args_list`.
/// Returns the first list element that is a dict (mamba folds keyword
/// arguments into a single trailing dict positional).
fn kwargs_dict_of(args_list: MbValue) -> Option<MbValue> {
    let items = args_list.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::List(ref lock) = (*ptr).data {
            lock.read().ok().map(|g| g.to_vec())
        } else { None }
    })?;
    for v in items {
        if is_dict(v) { return Some(v); }
    }
    None
}

/// Read a value out of a kwargs dict by name.
fn kwarg_get(dict: MbValue, name: &str) -> Option<MbValue> {
    dict.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Dict(ref lock) = (*ptr).data {
            let guard = lock.read().ok()?;
            let key = super::super::dict_ops::DictKey::Str(name.to_string());
            guard.get(&key).copied()
        } else { None }
    })
}

/// Build a `datetime.time` Instance carrying h/m/s/us and an explicit `fold`.
fn build_time_instance_fold(h: i64, m: i64, s: i64, us: i64, fold: i64) -> MbValue {
    let val = build_time_instance(h, m, s, us);
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                if let Ok(mut f) = fields.write() {
                    f.insert("fold".into(), MbValue::from_int(fold));
                }
            }
        }
    }
    val
}

/// `datetime.isoformat([sep][, timespec])` and `date.isoformat()`.
/// Variadic: `(self, args_list)`.
unsafe extern "C" fn dt_method_isoformat(self_: MbValue, args_list: MbValue) -> MbValue {
    let y = inst_int(self_, "year", 1970);
    let mo = inst_int(self_, "month", 1);
    let d = inst_int(self_, "day", 1);
    if inst_is_date(self_) {
        return MbValue::from_ptr(MbObject::new_str(format!("{y:04}-{mo:02}-{d:02}")));
    }
    let h = inst_int(self_, "hour", 0);
    let mi = inst_int(self_, "minute", 0);
    let s = inst_int(self_, "second", 0);
    let us = inst_int(self_, "microsecond", 0);
    // Default separator is 'T'; `sep=` (first positional or keyword) overrides.
    let mut sep = "T".to_string();
    let mut timespec: Option<String> = None;
    if let Some(dict) = kwargs_dict_of(args_list) {
        if let Some(v) = kwarg_get(dict, "sep") {
            if let Some(sep_str) = extract_str(v) { sep = sep_str; }
        }
        if let Some(v) = kwarg_get(dict, "timespec") {
            timespec = extract_str(v);
        }
    } else if let Some(ptr) = args_list.as_ptr() {
        // Positional `isoformat(sep)` form.
        if let ObjData::List(ref lock) = (*ptr).data {
            if let Ok(g) = lock.read() {
                if let Some(first) = g.first() {
                    if let Some(sep_str) = extract_str(*first) { sep = sep_str; }
                }
            }
        }
    }
    if let Some(spec) = timespec.as_deref() {
        if !is_valid_timespec(spec) {
            return raise_value_error(&format!("Unknown timespec value: {spec}"));
        }
    }
    let time_part = format_time_timespec(h, mi, s, us, timespec.as_deref());
    MbValue::from_ptr(MbObject::new_str(format!("{y:04}-{mo:02}-{d:02}{sep}{time_part}")))
}

/// `time.isoformat([timespec])`. Variadic: `(self, args_list)`.
unsafe extern "C" fn time_method_isoformat(self_: MbValue, args_list: MbValue) -> MbValue {
    let h = inst_int(self_, "hour", 0);
    let mi = inst_int(self_, "minute", 0);
    let s = inst_int(self_, "second", 0);
    let us = inst_int(self_, "microsecond", 0);
    let mut timespec: Option<String> = None;
    if let Some(dict) = kwargs_dict_of(args_list) {
        if let Some(v) = kwarg_get(dict, "timespec") {
            timespec = extract_str(v);
        }
    } else if let Some(ptr) = args_list.as_ptr() {
        if let ObjData::List(ref lock) = (*ptr).data {
            if let Ok(g) = lock.read() {
                if let Some(first) = g.first() {
                    timespec = extract_str(*first);
                }
            }
        }
    }
    // CPython rejects an explicit unknown `timespec` with ValueError; an absent
    // timespec (None) defaults to "auto" and never errors.
    if let Some(spec) = timespec.as_deref() {
        if !is_valid_timespec(spec) {
            return raise_value_error(&format!("Unknown timespec value: {spec}"));
        }
    }
    let out = format_time_timespec(h, mi, s, us, timespec.as_deref());
    MbValue::from_ptr(MbObject::new_str(out))
}

/// The `timespec` keyword values CPython's `isoformat` accepts. Any other
/// explicitly-supplied value raises `ValueError` (an absent timespec is `auto`).
fn is_valid_timespec(spec: &str) -> bool {
    matches!(
        spec,
        "auto" | "hours" | "minutes" | "seconds" | "milliseconds" | "microseconds"
    )
}

/// Render the time portion of an ISO string honoring CPython's `timespec`.
/// `None`/"auto" drop the microseconds when zero (matching `isoformat`).
fn format_time_timespec(h: i64, mi: i64, s: i64, us: i64, timespec: Option<&str>) -> String {
    match timespec.unwrap_or("auto") {
        "hours" => format!("{h:02}"),
        "minutes" => format!("{h:02}:{mi:02}"),
        "seconds" => format!("{h:02}:{mi:02}:{s:02}"),
        "milliseconds" => format!("{h:02}:{mi:02}:{s:02}.{:03}", us / 1000),
        "microseconds" => format!("{h:02}:{mi:02}:{s:02}.{us:06}"),
        // "auto" (and any unrecognized value): include microseconds only when nonzero.
        _ => {
            if us != 0 {
                format!("{h:02}:{mi:02}:{s:02}.{us:06}")
            } else {
                format!("{h:02}:{mi:02}:{s:02}")
            }
        }
    }
}

/// `datetime.ctime()` / `date.ctime()` → "Www Mmm DD HH:MM:SS YYYY"
/// with a space-padded day-of-month (CPython, C locale).
unsafe extern "C" fn dt_method_ctime(self_: MbValue) -> MbValue {
    let y = inst_int(self_, "year", 1970);
    let mo = inst_int(self_, "month", 1);
    let d = inst_int(self_, "day", 1);
    let h = inst_int(self_, "hour", 0);
    let mi = inst_int(self_, "minute", 0);
    let s = inst_int(self_, "second", 0);
    let wd = match NaiveDate::from_ymd_opt(y as i32, mo as u32, d as u32) {
        Some(nd) => nd.weekday().num_days_from_monday() as usize,
        None => 0,
    };
    let wname = WD_ABBR.get(wd).copied().unwrap_or("Mon");
    let mname = MO_ABBR.get((mo as usize).saturating_sub(1)).copied().unwrap_or("Jan");
    // Day is space-padded to width 2 (e.g. "Mar  2").
    MbValue::from_ptr(MbObject::new_str(
        format!("{wname} {mname} {d:>2} {h:02}:{mi:02}:{s:02} {y:04}")
    ))
}

/// `date.weekday()` / `datetime.weekday()` → Monday=0 .. Sunday=6.
unsafe extern "C" fn dt_method_weekday(self_: MbValue) -> MbValue {
    let y = inst_int(self_, "year", 1970);
    let mo = inst_int(self_, "month", 1);
    let d = inst_int(self_, "day", 1);
    match NaiveDate::from_ymd_opt(y as i32, mo as u32, d as u32) {
        Some(nd) => MbValue::from_int(nd.weekday().num_days_from_monday() as i64),
        None => MbValue::from_int(0),
    }
}

/// `date.isoweekday()` / `datetime.isoweekday()` → Monday=1 .. Sunday=7.
unsafe extern "C" fn dt_method_isoweekday(self_: MbValue) -> MbValue {
    let y = inst_int(self_, "year", 1970);
    let mo = inst_int(self_, "month", 1);
    let d = inst_int(self_, "day", 1);
    match NaiveDate::from_ymd_opt(y as i32, mo as u32, d as u32) {
        Some(nd) => MbValue::from_int(nd.weekday().number_from_monday() as i64),
        None => MbValue::from_int(1),
    }
}

/// `date.isocalendar()` / `datetime.isocalendar()` → (ISO year, week, weekday).
/// CPython 3.12 returns an IsoCalendarDate named tuple; it compares equal to a
/// plain (year, week, weekday) tuple, so we return a tuple.
unsafe extern "C" fn dt_method_isocalendar(self_: MbValue) -> MbValue {
    let y = inst_int(self_, "year", 1970);
    let mo = inst_int(self_, "month", 1);
    let d = inst_int(self_, "day", 1);
    match NaiveDate::from_ymd_opt(y as i32, mo as u32, d as u32) {
        Some(nd) => {
            let iso = nd.iso_week();
            MbValue::from_ptr(MbObject::new_tuple(vec![
                MbValue::from_int(iso.year() as i64),
                MbValue::from_int(iso.week() as i64),
                MbValue::from_int(nd.weekday().number_from_monday() as i64),
            ]))
        }
        None => MbValue::from_ptr(MbObject::new_tuple(vec![
            MbValue::from_int(y), MbValue::from_int(1), MbValue::from_int(1),
        ])),
    }
}

/// `date.timetuple()` / `datetime.timetuple()` → a `struct_time`-shaped
/// Instance (named tm_* fields). Mirrors `time.struct_time`'s field layout.
unsafe extern "C" fn dt_method_timetuple(self_: MbValue) -> MbValue {
    let y = inst_int(self_, "year", 1970);
    let mo = inst_int(self_, "month", 1);
    let d = inst_int(self_, "day", 1);
    let h = inst_int(self_, "hour", 0);
    let mi = inst_int(self_, "minute", 0);
    let s = inst_int(self_, "second", 0);
    let (wday, yday) = match NaiveDate::from_ymd_opt(y as i32, mo as u32, d as u32) {
        Some(nd) => (nd.weekday().num_days_from_monday() as i64, nd.ordinal() as i64),
        None => (0, 1),
    };
    let mut fields = FxHashMap::default();
    fields.insert("tm_year".into(),  MbValue::from_int(y));
    fields.insert("tm_mon".into(),   MbValue::from_int(mo));
    fields.insert("tm_mday".into(),  MbValue::from_int(d));
    fields.insert("tm_hour".into(),  MbValue::from_int(h));
    fields.insert("tm_min".into(),   MbValue::from_int(mi));
    fields.insert("tm_sec".into(),   MbValue::from_int(s));
    fields.insert("tm_wday".into(),  MbValue::from_int(wday));
    fields.insert("tm_yday".into(),  MbValue::from_int(yday));
    fields.insert("tm_isdst".into(), MbValue::from_int(-1));
    fields.insert("n_fields".into(),          MbValue::from_int(9));
    fields.insert("n_sequence_fields".into(), MbValue::from_int(9));
    fields.insert("n_unnamed_fields".into(),  MbValue::from_int(0));
    let obj = Box::new(super::super::rc::MbObject {
        header: super::super::rc::MbObjectHeader {
            rc: std::sync::atomic::AtomicU32::new(1),
            kind: super::super::rc::ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "struct_time".to_string(),
            fields: crate::runtime::rc::MbRwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

/// `datetime.replace(**changes)` → a new datetime with overridden components.
/// Variadic: `(self, args_list)`. The original instance is left unchanged.
unsafe extern "C" fn dt_method_replace(self_: MbValue, args_list: MbValue) -> MbValue {
    let mut y = inst_int(self_, "year", 1970);
    let mut mo = inst_int(self_, "month", 1);
    let mut d = inst_int(self_, "day", 1);
    let mut h = inst_int(self_, "hour", 0);
    let mut mi = inst_int(self_, "minute", 0);
    let mut s = inst_int(self_, "second", 0);
    let was_date = inst_is_date(self_);
    if let Some(dict) = kwargs_dict_of(args_list) {
        if let Some(v) = kwarg_get(dict, "year").and_then(|v| v.as_int()) { y = v; }
        if let Some(v) = kwarg_get(dict, "month").and_then(|v| v.as_int()) { mo = v; }
        if let Some(v) = kwarg_get(dict, "day").and_then(|v| v.as_int()) { d = v; }
        if let Some(v) = kwarg_get(dict, "hour").and_then(|v| v.as_int()) { h = v; }
        if let Some(v) = kwarg_get(dict, "minute").and_then(|v| v.as_int()) { mi = v; }
        if let Some(v) = kwarg_get(dict, "second").and_then(|v| v.as_int()) { s = v; }
        // `fold=` is accepted but does not affect the stored field set used for
        // equality/hash (CPython: fold never changes hash, and only affects
        // wall-clock-to-UTC conversions which mamba does not model here).
    }
    let new_val = match NaiveDate::from_ymd_opt(y as i32, mo as u32, d as u32)
        .and_then(|nd| nd.and_hms_opt(h as u32, mi as u32, s as u32))
    {
        Some(dt) => build_datetime_dict(dt),
        None => return raise_value_error(&format!(
            "invalid replace: {y}-{mo}-{d} {h}:{mi}:{s}"
        )),
    };
    if was_date {
        if let Some(ptr) = new_val.as_ptr() {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                if let Ok(mut f) = fields.write() {
                    f.insert("_is_date".into(), MbValue::from_bool(true));
                }
            }
        }
    }
    new_val
}

/// `datetime.time()` → a naive `datetime.time` projection (drops tzinfo).
/// `fold` propagates from the datetime. Single-arg: `(self)`.
unsafe extern "C" fn dt_method_time(self_: MbValue) -> MbValue {
    let h = inst_int(self_, "hour", 0);
    let mi = inst_int(self_, "minute", 0);
    let s = inst_int(self_, "second", 0);
    let us = inst_int(self_, "microsecond", 0);
    let fold = inst_int(self_, "fold", 0);
    build_time_instance_fold(h, mi, s, us, fold)
}

/// `datetime.timetz()` → a `datetime.time` projection carrying tzinfo.
/// mamba does not model tzinfo on these projections yet; `fold` propagates.
unsafe extern "C" fn dt_method_timetz(self_: MbValue) -> MbValue {
    dt_method_time(self_)
}

/// `datetime.__eq__(other)` — value equality over the y/m/d/h/m/s components.
/// Single positional arg: `(self, other)`.
unsafe extern "C" fn dt_method_eq(self_: MbValue, other: MbValue) -> MbValue {
    let is_dt_instance = other.as_ptr().map(|ptr| unsafe {
        if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
            class_name == "datetime.datetime"
        } else { false }
    }).unwrap_or(false);
    if !is_dt_instance {
        return MbValue::not_implemented();
    }
    let eq = inst_int(self_, "year", 1970) == inst_int(other, "year", 1970)
        && inst_int(self_, "month", 1) == inst_int(other, "month", 1)
        && inst_int(self_, "day", 1) == inst_int(other, "day", 1)
        && inst_int(self_, "hour", 0) == inst_int(other, "hour", 0)
        && inst_int(self_, "minute", 0) == inst_int(other, "minute", 0)
        && inst_int(self_, "second", 0) == inst_int(other, "second", 0)
        && inst_int(self_, "microsecond", 0) == inst_int(other, "microsecond", 0);
    MbValue::from_bool(eq)
}

/// `datetime.__hash__()` — value hash over the components, deliberately
/// independent of `fold` (CPython guarantees fold never affects the hash).
unsafe extern "C" fn dt_method_hash(self_: MbValue) -> MbValue {
    let mut acc: i64 = 0;
    for (name, def) in [
        ("year", 1970), ("month", 1), ("day", 1),
        ("hour", 0), ("minute", 0), ("second", 0), ("microsecond", 0),
    ] {
        acc = acc.wrapping_mul(1_000_003).wrapping_add(inst_int(self_, name, def));
    }
    // Confine to the 48-bit tagged-int payload (sign-extended).
    let h = (acc << 16) >> 16;
    MbValue::from_int(if h == -1 { -2 } else { h })
}

/// `time.replace(**changes)` → a new `datetime.time` with overrides.
/// Variadic: `(self, args_list)`. `fold` is accepted but not stored in the
/// hashed field set.
unsafe extern "C" fn time_method_replace(self_: MbValue, args_list: MbValue) -> MbValue {
    let mut h = inst_int(self_, "hour", 0);
    let mut mi = inst_int(self_, "minute", 0);
    let mut s = inst_int(self_, "second", 0);
    let mut us = inst_int(self_, "microsecond", 0);
    if let Some(dict) = kwargs_dict_of(args_list) {
        if let Some(v) = kwarg_get(dict, "hour").and_then(|v| v.as_int()) { h = v; }
        if let Some(v) = kwarg_get(dict, "minute").and_then(|v| v.as_int()) { mi = v; }
        if let Some(v) = kwarg_get(dict, "second").and_then(|v| v.as_int()) { s = v; }
        if let Some(v) = kwarg_get(dict, "microsecond").and_then(|v| v.as_int()) { us = v; }
    }
    build_time_instance(h, mi, s, us)
}

/// `time.__eq__(other)` — value equality over h/m/s/us (fold excluded).
unsafe extern "C" fn time_method_eq(self_: MbValue, other: MbValue) -> MbValue {
    let is_time_instance = other.as_ptr().map(|ptr| unsafe {
        if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
            class_name == "datetime.time"
        } else { false }
    }).unwrap_or(false);
    if !is_time_instance {
        return MbValue::not_implemented();
    }
    let eq = inst_int(self_, "hour", 0) == inst_int(other, "hour", 0)
        && inst_int(self_, "minute", 0) == inst_int(other, "minute", 0)
        && inst_int(self_, "second", 0) == inst_int(other, "second", 0)
        && inst_int(self_, "microsecond", 0) == inst_int(other, "microsecond", 0);
    MbValue::from_bool(eq)
}

/// `time.__hash__()` — value hash over h/m/s/us, independent of `fold`.
unsafe extern "C" fn time_method_hash(self_: MbValue) -> MbValue {
    let mut acc: i64 = 0;
    for (name, def) in [("hour", 0), ("minute", 0), ("second", 0), ("microsecond", 0)] {
        acc = acc.wrapping_mul(1_000_003).wrapping_add(inst_int(self_, name, def));
    }
    let h = (acc << 16) >> 16;
    MbValue::from_int(if h == -1 { -2 } else { h })
}

/// Build a datetime Instance with year/month/day/hour/minute/second fields.
/// `runtime::class::mb_call_method` short-circuits the
/// `datetime.datetime` class name to dispatch `.strftime()`, `.timestamp()`,
/// and related methods.
pub(crate) fn build_datetime_dict(dt: NaiveDateTime) -> MbValue {
    let mut fields = FxHashMap::default();
    fields.insert("year".into(), MbValue::from_int(dt.year() as i64));
    fields.insert("month".into(), MbValue::from_int(dt.month() as i64));
    fields.insert("day".into(), MbValue::from_int(dt.day() as i64));
    fields.insert("hour".into(), MbValue::from_int(dt.hour() as i64));
    fields.insert("minute".into(), MbValue::from_int(dt.minute() as i64));
    fields.insert("second".into(), MbValue::from_int(dt.second() as i64));
    let obj = Box::new(super::super::rc::MbObject {
        header: super::super::rc::MbObjectHeader {
            rc: std::sync::atomic::AtomicU32::new(1),
            kind: super::super::rc::ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "datetime.datetime".to_string(),
            fields: crate::runtime::rc::MbRwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

/// Extract datetime fields from a dict into NaiveDateTime.
#[allow(dead_code)]
fn dict_to_naive(map: &IndexMap<DictKey, MbValue>) -> Option<NaiveDateTime> {
    let year = map.get("year").and_then(|v| v.as_int()).unwrap_or(1970) as i32;
    let month = map.get("month").and_then(|v| v.as_int()).unwrap_or(1) as u32;
    let day = map.get("day").and_then(|v| v.as_int()).unwrap_or(1) as u32;
    let hour = map.get("hour").and_then(|v| v.as_int()).unwrap_or(0) as u32;
    let minute = map.get("minute").and_then(|v| v.as_int()).unwrap_or(0) as u32;
    let second = map.get("second").and_then(|v| v.as_int()).unwrap_or(0) as u32;

    NaiveDate::from_ymd_opt(year, month, day)
        .and_then(|d| d.and_hms_opt(hour, minute, second))
}

/// Extract datetime fields from either an Instance or a legacy dict, into a
/// `NaiveDateTime`. Returns `None` if the value is neither.
pub(crate) fn instance_to_naive(val: MbValue) -> Option<NaiveDateTime> {
    let ptr = val.as_ptr()?;
    unsafe {
        match &(*ptr).data {
            ObjData::Instance { ref fields, .. } => {
                let f = fields.read().ok()?;
                let get = |k: &str| f.get(k).and_then(|v| v.as_int());
                let year = get("year").unwrap_or(1970) as i32;
                let month = get("month").unwrap_or(1) as u32;
                let day = get("day").unwrap_or(1) as u32;
                let hour = get("hour").unwrap_or(0) as u32;
                let minute = get("minute").unwrap_or(0) as u32;
                let second = get("second").unwrap_or(0) as u32;
                NaiveDate::from_ymd_opt(year, month, day)
                    .and_then(|d| d.and_hms_opt(hour, minute, second))
            }
            ObjData::Dict(ref lock) => {
                let map = lock.read().unwrap();
                dict_to_naive(&map)
            }
            _ => None,
        }
    }
}

// ── Runtime functions ──

/// datetime.now() -> dict {year, month, day, hour, minute, second}
pub fn mb_datetime_now() -> MbValue {
    let now: DateTime<Utc> = Utc::now();
    build_datetime_dict(now.naive_utc())
}

/// datetime.new(args) -> dict
/// args = [year, month, day, hour, minute, second]
pub fn mb_datetime_new(args: MbValue) -> MbValue {
    let items = match args.as_ptr() {
        Some(ptr) => unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                lock.read().unwrap().clone()
            } else { return MbValue::none(); }
        },
        None => return MbValue::none(),
    };

    let get = |idx: usize, def: i64, name: &str| -> Result<i64, MbValue> {
        match items.get(idx) {
            Some(v) => {
                if let Some(n) = v.as_int() {
                    Ok(n)
                } else if is_dict(*v) {
                    // Keyword arguments arrive as a trailing dict in mamba's
                    // current call lowering. Ignore unsupported kwargs here
                    // instead of treating that dict as a bad positional field.
                    Ok(def)
                } else {
                    Err(raise_type_error(&format!("{name} must be an integer")))
                }
            }
            None => Ok(def),
        }
    };

    let year = match get(0, 1970, "year") { Ok(n) => n as i32, Err(e) => return e };
    let month = match get(1, 1, "month") { Ok(n) => n as u32, Err(e) => return e };
    let day = match get(2, 1, "day") { Ok(n) => n as u32, Err(e) => return e };
    let hour = match get(3, 0, "hour") { Ok(n) => n as u32, Err(e) => return e };
    let minute = match get(4, 0, "minute") { Ok(n) => n as u32, Err(e) => return e };
    let second = match get(5, 0, "second") { Ok(n) => n as u32, Err(e) => return e };

    // `fold=` (and `microsecond=`) may arrive in the trailing kwargs dict.
    // Capture them so `.fold`/`.microsecond` reads and `.time()`/`.timetz()`
    // projections see the constructed value (CPython stores fold on the
    // datetime; it never affects equality or hashing).
    let mut fold = 0i64;
    let mut micro = 0i64;
    for v in &items {
        if is_dict(*v) {
            if let Some(f) = kwarg_get(*v, "fold").and_then(|x| x.as_int()) { fold = f; }
            if let Some(m) = kwarg_get(*v, "microsecond").and_then(|x| x.as_int()) { micro = m; }
        }
    }

    match NaiveDate::from_ymd_opt(year, month, day)
        .and_then(|d| d.and_hms_opt(hour, minute, second))
    {
        Some(dt) => {
            let val = build_datetime_dict(dt);
            if (fold != 0 || micro != 0) && val.as_ptr().is_some() {
                let ptr = val.as_ptr().unwrap();
                unsafe {
                    if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                        if let Ok(mut f) = fields.write() {
                            if fold != 0 { f.insert("fold".into(), MbValue::from_int(fold)); }
                            if micro != 0 { f.insert("microsecond".into(), MbValue::from_int(micro)); }
                        }
                    }
                }
            }
            val
        }
        None => {
            super::super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                MbValue::from_ptr(MbObject::new_str(
                    format!("invalid datetime: {year}-{month}-{day} {hour}:{minute}:{second}")
                )),
            );
            MbValue::none()
        }
    }
}

/// date.today() -> dict {year, month, day}
pub fn mb_date_today() -> MbValue {
    let today = Utc::now().naive_utc().date();
    let dict = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut map = lock.write().unwrap();
            map.insert("year".into(), MbValue::from_int(today.year() as i64));
            map.insert("month".into(), MbValue::from_int(today.month() as i64));
            map.insert("day".into(), MbValue::from_int(today.day() as i64));
        }
    }
    MbValue::from_ptr(dict)
}

/// timedelta.new(args) -> datetime.timedelta Instance with days + seconds.
pub fn mb_timedelta_new(args: MbValue) -> MbValue {
    let items = match args.as_ptr() {
        Some(ptr) => unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                lock.read().unwrap().clone()
            } else { return MbValue::none(); }
        },
        None => return MbValue::none(),
    };

    // The mamba call lowering folds keyword arguments into a trailing dict
    // positional. `timedelta(days=7)` arrives here as
    // `[{"days": 7}]` rather than `[7]`, so a positional-only read would
    // see no int and fall back to 0. Probe each positional slot for either
    // a raw int or a kwargs dict containing the corresponding name.
    let pull_int = |idx: usize, name: &str| -> i64 {
        if let Some(v) = items.get(idx) {
            if let Some(n) = v.as_int() { return n; }
            if let Some(ptr) = v.as_ptr() {
                unsafe {
                    if let ObjData::Dict(ref lock) = (*ptr).data {
                        let guard = lock.read().unwrap();
                        let key = super::super::dict_ops::DictKey::Str(name.to_string());
                        if let Some(found) = guard.get(&key) {
                            if let Some(n) = found.as_int() { return n; }
                        }
                    }
                }
            }
        }
        // Also scan all slots for a kwargs dict — the dict may not be at the
        // exact positional index when mixed positional/keyword forms are used.
        for v in items.iter() {
            if let Some(ptr) = v.as_ptr() {
                unsafe {
                    if let ObjData::Dict(ref lock) = (*ptr).data {
                        let guard = lock.read().unwrap();
                        let key = super::super::dict_ops::DictKey::Str(name.to_string());
                        if let Some(found) = guard.get(&key) {
                            if let Some(n) = found.as_int() { return n; }
                        }
                    }
                }
            }
        }
        0
    };
    let days = pull_int(0, "days");
    let seconds = pull_int(1, "seconds");

    let mut fields = FxHashMap::default();
    fields.insert("days".into(), MbValue::from_int(days));
    fields.insert("seconds".into(), MbValue::from_int(seconds));
    let obj = Box::new(super::super::rc::MbObject {
        header: super::super::rc::MbObjectHeader {
            rc: std::sync::atomic::AtomicU32::new(1),
            kind: super::super::rc::ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "datetime.timedelta".to_string(),
            fields: crate::runtime::rc::MbRwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

/// Add a timedelta to a datetime. Returns a new datetime.datetime Instance
/// with the shifted date. Called by `mb_add` when both operands are
/// datetime/timedelta Instances.
pub fn mb_datetime_add_timedelta(dt: MbValue, td: MbValue) -> MbValue {
    let naive = match instance_to_naive(dt) {
        Some(n) => n,
        None => return MbValue::none(),
    };
    let (days, seconds) = match td.as_ptr() {
        Some(ptr) => unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let f = fields.read().unwrap();
                (
                    f.get("days").and_then(|v| v.as_int()).unwrap_or(0),
                    f.get("seconds").and_then(|v| v.as_int()).unwrap_or(0),
                )
            } else {
                return MbValue::none();
            }
        },
        None => return MbValue::none(),
    };
    let shifted = naive + chrono::Duration::days(days) + chrono::Duration::seconds(seconds);
    build_datetime_dict(shifted)
}

/// datetime.strftime(dt, fmt) -> formatted string
pub fn mb_datetime_strftime(dt: MbValue, fmt: MbValue) -> MbValue {
    let fmt_str = match extract_str(fmt) { Some(s) => s, None => return MbValue::none() };
    match instance_to_naive(dt) {
        Some(naive) => {
            let formatted = naive.format(&fmt_str).to_string();
            MbValue::from_ptr(MbObject::new_str(formatted))
        }
        None => MbValue::none(),
    }
}

/// datetime.strptime(date_string, fmt) -> datetime.datetime Instance.
/// Parses `date_string` against the strftime-style `fmt`, mirroring CPython's
/// `datetime.datetime.strptime`. Raises `ValueError` on a parse mismatch.
pub fn mb_datetime_strptime(date_string: MbValue, fmt: MbValue) -> MbValue {
    let s = match extract_str(date_string) {
        Some(v) => v,
        None => return raise_type_error("strptime() argument 1 must be str"),
    };
    let f = match extract_str(fmt) {
        Some(v) => v,
        None => return raise_type_error("strptime() argument 2 must be str"),
    };
    match NaiveDateTime::parse_from_str(&s, &f) {
        Ok(dt) => build_datetime_dict(dt),
        Err(_) => {
            // Date-only formats parse into a NaiveDate; promote to midnight.
            match NaiveDate::parse_from_str(&s, &f) {
                Ok(d) => build_datetime_dict(d.and_hms_opt(0, 0, 0).unwrap()),
                Err(_) => raise_value_error(&format!(
                    "time data '{s}' does not match format '{f}'"
                )),
            }
        }
    }
}

/// datetime.combine(date, time) -> datetime.datetime Instance.
/// Takes the year/month/day from `date` and the hour/minute/second from
/// `time`, mirroring CPython's `datetime.datetime.combine`.
pub fn mb_datetime_combine(date: MbValue, time: MbValue) -> MbValue {
    let d = instance_to_naive(date).unwrap_or_else(|| {
        NaiveDate::from_ymd_opt(1970, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap()
    });
    let read_t = |name: &str| -> i64 {
        time.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                fields.read().ok()
                    .and_then(|fl| fl.get(name).copied())
                    .and_then(|v| v.as_int())
            } else { None }
        }).unwrap_or(0)
    };
    let h = read_t("hour") as u32;
    let mi = read_t("minute") as u32;
    let sec = read_t("second") as u32;
    let combined = NaiveDate::from_ymd_opt(d.year(), d.month(), d.day())
        .and_then(|nd| nd.and_hms_opt(h, mi, sec))
        .unwrap_or(d);
    build_datetime_dict(combined)
}

/// datetime.timestamp(dt) -> float (Unix timestamp)
pub fn mb_datetime_timestamp(dt: MbValue) -> MbValue {
    match instance_to_naive(dt) {
        Some(naive) => {
            let ts = naive.and_utc().timestamp() as f64;
            MbValue::from_float(ts)
        }
        None => MbValue::none(),
    }
}

/// datetime.isoformat(dt) -> "YYYY-MM-DDTHH:MM:SS"
/// REQ: R6
pub fn mb_datetime_isoformat(dt: MbValue) -> MbValue {
    match instance_to_naive(dt) {
        Some(naive) => {
            let s = naive.format("%Y-%m-%dT%H:%M:%S").to_string();
            MbValue::from_ptr(MbObject::new_str(s))
        }
        None => MbValue::from_ptr(MbObject::new_str(String::new())),
    }
}

/// datetime.fromisoformat(s) -> datetime Instance
/// Accepts "YYYY-MM-DDTHH:MM:SS" or "YYYY-MM-DD" (midnight).
/// REQ: R6
pub fn mb_datetime_fromisoformat(s: MbValue) -> MbValue {
    let raw = match extract_str(s) {
        Some(v) => v,
        None => {
            super::super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                MbValue::from_ptr(MbObject::new_str("Invalid isoformat string".to_string())),
            );
            return MbValue::none();
        }
    };
    // Try full datetime first, then date-only at midnight.
    if let Ok(dt) = NaiveDateTime::parse_from_str(&raw, "%Y-%m-%dT%H:%M:%S") {
        return build_datetime_dict(dt);
    }
    if let Ok(d) = NaiveDate::parse_from_str(&raw, "%Y-%m-%d") {
        let dt = d.and_hms_opt(0, 0, 0).unwrap();
        return build_datetime_dict(dt);
    }
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
        MbValue::from_ptr(MbObject::new_str("Invalid isoformat string".to_string())),
    );
    MbValue::none()
}

/// date.isoformat(d) -> "YYYY-MM-DD"
/// REQ: R6
pub fn mb_date_isoformat(d: MbValue) -> MbValue {
    match instance_to_naive(d) {
        Some(naive) => {
            let s = naive.format("%Y-%m-%d").to_string();
            MbValue::from_ptr(MbObject::new_str(s))
        }
        None => MbValue::from_ptr(MbObject::new_str(String::new())),
    }
}

/// datetime.fromtimestamp(ts) -> dict
pub fn mb_datetime_fromtimestamp(ts: MbValue) -> MbValue {
    let secs = if let Some(f) = ts.as_float() {
        f as i64
    } else if let Some(i) = ts.as_int() {
        i
    } else {
        return MbValue::none();
    };

    match DateTime::from_timestamp(secs, 0) {
        Some(dt) => build_datetime_dict(dt.naive_utc()),
        None => MbValue::none(),
    }
}

// ── repr / str helpers (#1644) ──

fn read_int_field(val: MbValue, name: &str) -> i64 {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            fields.read().ok()
                .and_then(|f| f.get(name).copied())
                .and_then(|v| v.as_int())
        } else { None }
    }).unwrap_or(0)
}

/// CPython-style `repr(datetime.datetime(...))`.
/// Always emits year, month, day, hour, minute; emits second only when nonzero.
pub fn datetime_repr(val: MbValue) -> String {
    let y = read_int_field(val, "year");
    let mo = read_int_field(val, "month");
    let d = read_int_field(val, "day");
    let h = read_int_field(val, "hour");
    let mi = read_int_field(val, "minute");
    let s = read_int_field(val, "second");
    if s != 0 {
        format!("datetime.datetime({y}, {mo}, {d}, {h}, {mi}, {s})")
    } else {
        format!("datetime.datetime({y}, {mo}, {d}, {h}, {mi})")
    }
}

/// CPython-style `str(datetime.datetime(...))` → `YYYY-MM-DD HH:MM:SS`.
pub fn datetime_str(val: MbValue) -> String {
    let y = read_int_field(val, "year");
    let mo = read_int_field(val, "month");
    let d = read_int_field(val, "day");
    let h = read_int_field(val, "hour");
    let mi = read_int_field(val, "minute");
    let s = read_int_field(val, "second");
    format!("{y:04}-{mo:02}-{d:02} {h:02}:{mi:02}:{s:02}")
}

/// CPython-style `repr(datetime.timedelta(...))`. Drops zero components;
/// renders `datetime.timedelta(0)` when both `days` and `seconds` are zero.
pub fn timedelta_repr(val: MbValue) -> String {
    let days = read_int_field(val, "days");
    let secs = read_int_field(val, "seconds");
    if days == 0 && secs == 0 {
        return "datetime.timedelta(0)".to_string();
    }
    let mut parts: Vec<String> = Vec::new();
    if days != 0 { parts.push(format!("days={days}")); }
    if secs != 0 { parts.push(format!("seconds={secs}")); }
    format!("datetime.timedelta({})", parts.join(", "))
}

/// CPython-style `str(datetime.timedelta(...))`.
/// `{D} day(s), {H}:{MM}:{SS}` when days != 0, else `{H}:{MM}:{SS}`.
/// Negative seconds are normalised the same way CPython does (carry into days).
pub fn timedelta_str(val: MbValue) -> String {
    let days = read_int_field(val, "days");
    let secs_total = read_int_field(val, "seconds");
    // Normalise: CPython keeps 0 <= seconds < 86400 with days carrying the rest.
    let mut d = days;
    let mut s = secs_total;
    if s >= 86_400 || s < 0 {
        let carry = s.div_euclid(86_400);
        d += carry;
        s -= carry * 86_400;
    }
    let h = s / 3600;
    let mi = (s % 3600) / 60;
    let sec = s % 60;
    if d == 0 {
        format!("{h}:{mi:02}:{sec:02}")
    } else if d == 1 || d == -1 {
        format!("{d} day, {h}:{mi:02}:{sec:02}")
    } else {
        format!("{d} days, {h}:{mi:02}:{sec:02}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(val: &str) -> MbValue {
        MbValue::from_ptr(MbObject::new_str(val.to_string()))
    }

    #[test]
    fn test_datetime_new_and_strftime() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(2025), MbValue::from_int(3), MbValue::from_int(15),
            MbValue::from_int(10), MbValue::from_int(30), MbValue::from_int(45),
        ]));
        let dt = mb_datetime_new(args);
        let formatted = mb_datetime_strftime(dt, s("%Y-%m-%d %H:%M:%S"));
        assert_eq!(extract_str(formatted).unwrap(), "2025-03-15 10:30:45");
    }

    #[test]
    fn test_timedelta_new() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(5), MbValue::from_int(3600),
        ]));
        let td = mb_timedelta_new(args);
        unsafe {
            if let ObjData::Dict(ref lock) = (*td.as_ptr().unwrap()).data {
                let map = lock.read().unwrap();
                assert_eq!(map.get("days").and_then(|v| v.as_int()), Some(5));
                assert_eq!(map.get("seconds").and_then(|v| v.as_int()), Some(3600));
            }
        }
    }

    #[test]
    fn test_datetime_timestamp_roundtrip() {
        // 2000-01-01 00:00:00 UTC = 946684800
        let args = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(2000), MbValue::from_int(1), MbValue::from_int(1),
            MbValue::from_int(0), MbValue::from_int(0), MbValue::from_int(0),
        ]));
        let dt = mb_datetime_new(args);
        let ts = mb_datetime_timestamp(dt);
        assert_eq!(ts.as_float().unwrap(), 946684800.0);
    }

    #[test]
    fn test_leap_year_feb29() {
        // 2024 is a leap year — Feb 28 + 1 day = Feb 29
        let args = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(2024), MbValue::from_int(2), MbValue::from_int(29),
            MbValue::from_int(0), MbValue::from_int(0), MbValue::from_int(0),
        ]));
        let dt = mb_datetime_new(args);
        unsafe {
            if let ObjData::Dict(ref lock) = (*dt.as_ptr().unwrap()).data {
                let map = lock.read().unwrap();
                assert_eq!(map.get("day").and_then(|v| v.as_int()), Some(29));
                assert_eq!(map.get("month").and_then(|v| v.as_int()), Some(2));
            }
        }
    }

    #[test]
    fn test_fromtimestamp() {
        let dt = mb_datetime_fromtimestamp(MbValue::from_int(0));
        unsafe {
            if let ObjData::Dict(ref lock) = (*dt.as_ptr().unwrap()).data {
                let map = lock.read().unwrap();
                assert_eq!(map.get("year").and_then(|v| v.as_int()), Some(1970));
                assert_eq!(map.get("month").and_then(|v| v.as_int()), Some(1));
                assert_eq!(map.get("day").and_then(|v| v.as_int()), Some(1));
            }
        }
    }

    #[test]
    fn test_datetime_returns_ptr() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(2024), MbValue::from_int(6), MbValue::from_int(15),
            MbValue::from_int(12), MbValue::from_int(0), MbValue::from_int(0),
        ]));
        let dt = mb_datetime_new(args);
        assert!(dt.is_ptr(), "datetime should return a ptr");
    }

    #[test]
    fn test_datetime_fields_year_month_day() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(2023), MbValue::from_int(11), MbValue::from_int(7),
            MbValue::from_int(9), MbValue::from_int(15), MbValue::from_int(30),
        ]));
        let dt = mb_datetime_new(args);
        unsafe {
            if let ObjData::Dict(ref lock) = (*dt.as_ptr().unwrap()).data {
                let map = lock.read().unwrap();
                assert_eq!(map.get("year").and_then(|v| v.as_int()), Some(2023));
                assert_eq!(map.get("month").and_then(|v| v.as_int()), Some(11));
                assert_eq!(map.get("day").and_then(|v| v.as_int()), Some(7));
                assert_eq!(map.get("hour").and_then(|v| v.as_int()), Some(9));
                assert_eq!(map.get("minute").and_then(|v| v.as_int()), Some(15));
                assert_eq!(map.get("second").and_then(|v| v.as_int()), Some(30));
            }
        }
    }

    #[test]
    fn test_timedelta_days_and_seconds() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(0), MbValue::from_int(7200),
        ]));
        let td = mb_timedelta_new(args);
        unsafe {
            if let ObjData::Dict(ref lock) = (*td.as_ptr().unwrap()).data {
                let map = lock.read().unwrap();
                assert_eq!(map.get("days").and_then(|v| v.as_int()), Some(0));
                assert_eq!(map.get("seconds").and_then(|v| v.as_int()), Some(7200));
            }
        }
    }

    #[test]
    fn test_strftime_date_only() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(2026), MbValue::from_int(3), MbValue::from_int(22),
            MbValue::from_int(0), MbValue::from_int(0), MbValue::from_int(0),
        ]));
        let dt = mb_datetime_new(args);
        let formatted = mb_datetime_strftime(dt, s("%Y/%m/%d"));
        assert_eq!(extract_str(formatted).unwrap(), "2026/03/22");
    }

    #[test]
    fn test_fromtimestamp_returns_ptr() {
        // Unix timestamp 1_000_000_000 → year 2001
        let result = mb_datetime_fromtimestamp(MbValue::from_int(1_000_000_000));
        assert!(result.is_ptr(), "fromtimestamp should return a ptr");
        unsafe {
            if let ObjData::Dict(ref lock) = (*result.as_ptr().unwrap()).data {
                let map = lock.read().unwrap();
                assert_eq!(map.get("year").and_then(|v| v.as_int()), Some(2001));
            }
        }
    }

    #[test]
    fn test_date_today_returns_ptr() {
        // date.today() should always return a valid dict ptr with year/month/day
        let today = mb_date_today();
        assert!(today.is_ptr(), "date.today() should return a ptr");
        unsafe {
            if let ObjData::Dict(ref lock) = (*today.as_ptr().unwrap()).data {
                let map = lock.read().unwrap();
                // Year must be a reasonable value (>= 2020)
                let year = map.get("year").and_then(|v| v.as_int()).unwrap();
                assert!(year >= 2020, "year should be >= 2020, got {year}");
                let month = map.get("month").and_then(|v| v.as_int()).unwrap();
                assert!((1..=12).contains(&month), "month out of range: {month}");
                let day = map.get("day").and_then(|v| v.as_int()).unwrap();
                assert!((1..=31).contains(&day), "day out of range: {day}");
            }
        }
    }

    #[test]
    fn test_datetime_invalid_date_returns_none() {
        // Feb 30 does not exist — mb_datetime_new should return None
        let args = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(2023), MbValue::from_int(2), MbValue::from_int(30),
            MbValue::from_int(0), MbValue::from_int(0), MbValue::from_int(0),
        ]));
        let result = mb_datetime_new(args);
        assert!(result.is_none(), "Feb 30 should produce None (invalid date)");
        // Clear the raised ValueError so it doesn't bleed into other tests
        // From inside mod tests: super=datetime_mod, super::super=stdlib, super::super::super=runtime
        super::super::super::exception::mb_clear_exception();
    }

    // REQ: R6
    #[test]
    fn test_isoformat() {
        use chrono::NaiveDate;
        let dt = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap().and_hms_opt(10, 30, 0).unwrap();
        let val = build_datetime_dict(dt);
        let result = mb_datetime_isoformat(val);
        assert_eq!(extract_str(result).unwrap(), "2024-01-15T10:30:00");
    }

    // REQ: R6
    #[test]
    fn test_fromisoformat_datetime() {
        let s_val = MbValue::from_ptr(MbObject::new_str("2024-01-15T10:30:00".to_string()));
        let dt = mb_datetime_fromisoformat(s_val);
        let naive = instance_to_naive(dt).expect("should parse datetime string");
        assert_eq!(naive.year(), 2024);
        assert_eq!(naive.month(), 1);
        assert_eq!(naive.day(), 15);
        assert_eq!(naive.hour(), 10);
        assert_eq!(naive.minute(), 30);
    }

    // REQ: R6
    #[test]
    fn test_fromisoformat_date_only() {
        let s_val = MbValue::from_ptr(MbObject::new_str("2024-06-15".to_string()));
        let dt = mb_datetime_fromisoformat(s_val);
        let naive = instance_to_naive(dt).expect("should parse date-only string");
        assert_eq!(naive.year(), 2024);
        assert_eq!(naive.month(), 6);
        assert_eq!(naive.day(), 15);
        assert_eq!(naive.hour(), 0);
    }

    // REQ: R6
    #[test]
    fn test_date_isoformat() {
        use chrono::NaiveDate;
        let dt = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap().and_hms_opt(0, 0, 0).unwrap();
        let val = build_datetime_dict(dt);
        let result = mb_date_isoformat(val);
        assert_eq!(extract_str(result).unwrap(), "2024-06-15");
    }
}
