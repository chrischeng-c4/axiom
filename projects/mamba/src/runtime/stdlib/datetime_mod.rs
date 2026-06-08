use super::super::dict_ops::DictKey;
use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, Timelike, Utc};
use indexmap::IndexMap;
use rustc_hash::FxHashMap;
/// datetime module for Mamba — backed by `chrono` crate.
///
/// Provides: datetime.now, datetime.new, date.today, timedelta.new,
///           datetime.strftime, datetime.timestamp, datetime.fromtimestamp
use std::collections::HashMap;

fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

fn is_dict(val: MbValue) -> bool {
    val.as_ptr()
        .is_some_and(|ptr| unsafe { matches!((*ptr).data, ObjData::Dict(_)) })
}

fn raise_type_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
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

/// Register the datetime module.
pub fn register() {
    let mut attrs = HashMap::new();
    // Python: `from datetime import datetime` → `datetime` is a class that
    // constructs a datetime. Register `datetime` and `date` as aliases for
    // `new` / `today` so the common `datetime(y, m, d)` / `date.today()`
    // idioms work against the module-as-dict dispatch.
    let dispatchers: [(&str, usize); 12] = [
        ("now", dispatch_now as *const () as usize),
        ("new", dispatch_new as *const () as usize),
        ("datetime", dispatch_new as *const () as usize),
        ("today", dispatch_today as *const () as usize),
        ("timedelta", dispatch_timedelta as *const () as usize),
        ("strftime", dispatch_strftime as *const () as usize),
        ("timestamp", dispatch_timestamp as *const () as usize),
        (
            "fromtimestamp",
            dispatch_fromtimestamp as *const () as usize,
        ),
        ("date", dispatch_date as *const () as usize),
        ("isoformat", dispatch_isoformat as *const () as usize),
        (
            "fromisoformat",
            dispatch_fromisoformat as *const () as usize,
        ),
        (
            "date_isoformat",
            dispatch_date_isoformat as *const () as usize,
        ),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }
    super::register_module("datetime", attrs);
}

/// Dispatch for `date(year, month, day)` — constructs a date-only value.
unsafe extern "C" fn dispatch_date(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let args_list = MbValue::from_ptr(MbObject::new_list(a.to_vec()));
    mb_datetime_new(args_list)
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

    NaiveDate::from_ymd_opt(year, month, day).and_then(|d| d.and_hms_opt(hour, minute, second))
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
            } else {
                return MbValue::none();
            }
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

    let year = match get(0, 1970, "year") {
        Ok(n) => n as i32,
        Err(e) => return e,
    };
    let month = match get(1, 1, "month") {
        Ok(n) => n as u32,
        Err(e) => return e,
    };
    let day = match get(2, 1, "day") {
        Ok(n) => n as u32,
        Err(e) => return e,
    };
    let hour = match get(3, 0, "hour") {
        Ok(n) => n as u32,
        Err(e) => return e,
    };
    let minute = match get(4, 0, "minute") {
        Ok(n) => n as u32,
        Err(e) => return e,
    };
    let second = match get(5, 0, "second") {
        Ok(n) => n as u32,
        Err(e) => return e,
    };

    match NaiveDate::from_ymd_opt(year, month, day)
        .and_then(|d| d.and_hms_opt(hour, minute, second))
    {
        Some(dt) => build_datetime_dict(dt),
        None => {
            super::super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                MbValue::from_ptr(MbObject::new_str(format!(
                    "invalid datetime: {year}-{month}-{day} {hour}:{minute}:{second}"
                ))),
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
            } else {
                return MbValue::none();
            }
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
            if let Some(n) = v.as_int() {
                return n;
            }
            if let Some(ptr) = v.as_ptr() {
                unsafe {
                    if let ObjData::Dict(ref lock) = (*ptr).data {
                        let guard = lock.read().unwrap();
                        let key = super::super::dict_ops::DictKey::Str(name.to_string());
                        if let Some(found) = guard.get(&key) {
                            if let Some(n) = found.as_int() {
                                return n;
                            }
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
                            if let Some(n) = found.as_int() {
                                return n;
                            }
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
    let fmt_str = match extract_str(fmt) {
        Some(s) => s,
        None => return MbValue::none(),
    };
    match instance_to_naive(dt) {
        Some(naive) => {
            let formatted = naive.format(&fmt_str).to_string();
            MbValue::from_ptr(MbObject::new_str(formatted))
        }
        None => MbValue::none(),
    }
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
    val.as_ptr()
        .and_then(|ptr| unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                fields
                    .read()
                    .ok()
                    .and_then(|f| f.get(name).copied())
                    .and_then(|v| v.as_int())
            } else {
                None
            }
        })
        .unwrap_or(0)
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
    if days != 0 {
        parts.push(format!("days={days}"));
    }
    if secs != 0 {
        parts.push(format!("seconds={secs}"));
    }
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
            MbValue::from_int(2025),
            MbValue::from_int(3),
            MbValue::from_int(15),
            MbValue::from_int(10),
            MbValue::from_int(30),
            MbValue::from_int(45),
        ]));
        let dt = mb_datetime_new(args);
        let formatted = mb_datetime_strftime(dt, s("%Y-%m-%d %H:%M:%S"));
        assert_eq!(extract_str(formatted).unwrap(), "2025-03-15 10:30:45");
    }

    #[test]
    fn test_timedelta_new() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(5),
            MbValue::from_int(3600),
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
            MbValue::from_int(2000),
            MbValue::from_int(1),
            MbValue::from_int(1),
            MbValue::from_int(0),
            MbValue::from_int(0),
            MbValue::from_int(0),
        ]));
        let dt = mb_datetime_new(args);
        let ts = mb_datetime_timestamp(dt);
        assert_eq!(ts.as_float().unwrap(), 946684800.0);
    }

    #[test]
    fn test_leap_year_feb29() {
        // 2024 is a leap year — Feb 28 + 1 day = Feb 29
        let args = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(2024),
            MbValue::from_int(2),
            MbValue::from_int(29),
            MbValue::from_int(0),
            MbValue::from_int(0),
            MbValue::from_int(0),
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
            MbValue::from_int(2024),
            MbValue::from_int(6),
            MbValue::from_int(15),
            MbValue::from_int(12),
            MbValue::from_int(0),
            MbValue::from_int(0),
        ]));
        let dt = mb_datetime_new(args);
        assert!(dt.is_ptr(), "datetime should return a ptr");
    }

    #[test]
    fn test_datetime_fields_year_month_day() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(2023),
            MbValue::from_int(11),
            MbValue::from_int(7),
            MbValue::from_int(9),
            MbValue::from_int(15),
            MbValue::from_int(30),
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
            MbValue::from_int(0),
            MbValue::from_int(7200),
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
            MbValue::from_int(2026),
            MbValue::from_int(3),
            MbValue::from_int(22),
            MbValue::from_int(0),
            MbValue::from_int(0),
            MbValue::from_int(0),
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
            MbValue::from_int(2023),
            MbValue::from_int(2),
            MbValue::from_int(30),
            MbValue::from_int(0),
            MbValue::from_int(0),
            MbValue::from_int(0),
        ]));
        let result = mb_datetime_new(args);
        assert!(
            result.is_none(),
            "Feb 30 should produce None (invalid date)"
        );
        // Clear the raised ValueError so it doesn't bleed into other tests
        // From inside mod tests: super=datetime_mod, super::super=stdlib, super::super::super=runtime
        super::super::super::exception::mb_clear_exception();
    }

    // REQ: R6
    #[test]
    fn test_isoformat() {
        use chrono::NaiveDate;
        let dt = NaiveDate::from_ymd_opt(2024, 1, 15)
            .unwrap()
            .and_hms_opt(10, 30, 0)
            .unwrap();
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
        let dt = NaiveDate::from_ymd_opt(2024, 6, 15)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let val = build_datetime_dict(dt);
        let result = mb_date_isoformat(val);
        assert_eq!(extract_str(result).unwrap(), "2024-06-15");
    }
}
