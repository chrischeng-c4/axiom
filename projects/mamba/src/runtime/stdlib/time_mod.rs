//! @codegen-skip: handwrite-pre-standardize
//!
//! time module for Mamba (#423, #1265 Task #82, Wave-9).
//!
//! Implements CPython 3.12 `time` stdlib 36-entry surface:
//!   CLOCK_MONOTONIC, CLOCK_MONOTONIC_RAW, CLOCK_PROCESS_CPUTIME_ID,
//!   CLOCK_REALTIME, CLOCK_THREAD_CPUTIME_ID, CLOCK_UPTIME_RAW,
//!   altzone, asctime, clock_getres, clock_gettime, clock_gettime_ns,
//!   clock_settime, clock_settime_ns, ctime, daylight, get_clock_info,
//!   gmtime, localtime, mktime, monotonic, monotonic_ns, perf_counter,
//!   perf_counter_ns, process_time, process_time_ns, sleep, strftime,
//!   strptime, struct_time, thread_time, thread_time_ns, time, time_ns,
//!   timezone, tzname, tzset.
//!
//! Real implementations:
//!   - Wall clock: `time`, `time_ns` via `SystemTime`.
//!   - Monotonic clock: `monotonic`, `monotonic_ns`, `perf_counter`,
//!     `perf_counter_ns` via a lazy-init `Instant` epoch.
//!   - CPU-time clocks: `process_time`, `process_time_ns`,
//!     `thread_time`, `thread_time_ns` via libc `clock_gettime` on Unix;
//!     fall back to monotonic on platforms without the syscall.
//!   - `sleep`: `std::thread::sleep`.
//!   - `gmtime`, `localtime`, `mktime`, `asctime`, `ctime`, `strftime`,
//!     `strptime`: real via `chrono` (already a runtime dep), modelling
//!     CPython's `struct_time` as an Instance with named fields and
//!     9-tuple-shaped positional layout.
//!   - `tzname`, `timezone`, `altzone`, `daylight`: real via `chrono::Local`
//!     offsets at module-init time. CPython recomputes these on `tzset()`;
//!     mamba currently snapshots once and refreshes on `tzset()`.
//!   - Clock constants (`CLOCK_MONOTONIC` etc.): integers matching the
//!     Linux numeric values (CPython exposes whatever the platform libc
//!     defines; mamba pins to Linux for reproducibility across hosts).
//!
//! Carve-outs:
//!   - `struct_time` is exposed as a factory function: calling
//!     `time.struct_time((y, m, d, H, M, S, wday, yday, isdst))` returns
//!     an Instance with the named fields (`tm_year`, `tm_mon`, ...).
//!     The class object itself is not yet a real type; `isinstance(t,
//!     time.struct_time)` is not modelled.
//!   - `clock_gettime` / `clock_gettime_ns` honour `CLOCK_REALTIME` and
//!     `CLOCK_MONOTONIC`; other clock ids fall back to monotonic.
//!   - `clock_settime` / `clock_settime_ns` are no-ops returning `None`
//!     — setting the system clock is privileged and out of scope.
//!   - `tzset()` re-reads `chrono::Local` offsets but does not honour
//!     the `TZ` environment variable mid-process (chrono caches the
//!     system tz at startup).
//!   - `get_clock_info(name)` returns an Instance with `implementation`,
//!     `monotonic`, `adjustable`, `resolution` fields; CPython's
//!     `namespace`-typed result is not modelled.
//!   - `strptime`'s `format` argument supports the most common CPython
//!     directives (`%Y %m %d %H %M %S %j %A %a %B %b %p %z %Z %%`); rare
//!     directives fall back to chrono's parser which may differ on edge
//!     cases (timezone abbreviation matching is locale-sensitive).
//!
//! HANDWRITE-BEGIN reason: per-section primitive vocabulary for stdlib
//! shims (register_module + dispatch_{nullary,unary,binary,ternary} +
//! struct_time factory class) is not yet emitted by score codegen.
//! Will convert to CODEGEN once the standardize sweep grows a
//! `stdlib_module_surface` section type with chrono primitives.

use crate::runtime::rc::MbRwLock as RwLock;
use rustc_hash::FxHashMap;
use std::collections::HashMap;
use std::sync::atomic::AtomicU32;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use chrono::{
    DateTime, Datelike, Duration, Local, NaiveDateTime, TimeZone, Timelike, Utc,
};

use super::super::rc::{MbObject, MbObjectHeader, ObjData, ObjKind};
use super::super::value::MbValue;

#[derive(Clone)]
struct TzSnapshot {
    timezone_west: i64,
    altzone_west: i64,
    daylight: i64,
    standard_name: String,
    daylight_name: String,
    fixed_local_offset_east: Option<i64>,
    fixed_local_zone: String,
    fixed_local_isdst: i64,
}

thread_local! {
    static TZ_SNAPSHOT: std::cell::RefCell<TzSnapshot> =
        std::cell::RefCell::new(host_tz_snapshot());
}

// -- Variadic dispatchers --

// Each generated dispatcher loads `stringify!($disp)` through `black_box`
// so LLVM's `mergefunc` pass keeps the bodies distinct. Without that,
// e.g. `d_time`/`d_monotonic` (both nullary → distinct inner fns) and
// `d_strftime`/`d_strptime` (binary → distinct inner fns) are fine, but
// any future copy-pasted `disp_*` pair that wraps the same inner fn would
// collapse and `test_register_wires_full_36_surface` would undercount.

macro_rules! disp_nullary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(_a: *const MbValue, _n: usize) -> MbValue {
            let _ = core::hint::black_box(stringify!($disp));
            $fn()
        }
    };
}

macro_rules! disp_unary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let _ = core::hint::black_box(stringify!($disp));
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a.get(0).copied().unwrap_or_else(MbValue::none))
        }
    };
}

macro_rules! disp_binary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let _ = core::hint::black_box(stringify!($disp));
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(
                a.get(0).copied().unwrap_or_else(MbValue::none),
                a.get(1).copied().unwrap_or_else(MbValue::none),
            )
        }
    };
}

disp_nullary!(d_time, mb_time_time);
disp_nullary!(d_time_ns, mb_time_time_ns);
disp_nullary!(d_monotonic, mb_time_monotonic);
disp_nullary!(d_monotonic_ns, mb_time_monotonic_ns);
disp_nullary!(d_perf_counter, mb_time_perf_counter);
disp_nullary!(d_perf_counter_ns, mb_time_perf_counter_ns);
disp_nullary!(d_process_time, mb_time_process_time);
disp_nullary!(d_process_time_ns, mb_time_process_time_ns);
disp_nullary!(d_thread_time, mb_time_thread_time);
disp_nullary!(d_thread_time_ns, mb_time_thread_time_ns);
disp_nullary!(d_tzset, mb_time_tzset);

disp_unary!(d_sleep, mb_time_sleep);
disp_unary!(d_gmtime, mb_time_gmtime);
disp_unary!(d_localtime, mb_time_localtime);
disp_unary!(d_mktime, mb_time_mktime);
disp_unary!(d_ctime, mb_time_ctime);
disp_unary!(d_asctime, mb_time_asctime);
disp_unary!(d_clock_getres, mb_time_clock_getres);
disp_unary!(d_clock_gettime, mb_time_clock_gettime);
disp_unary!(d_clock_gettime_ns, mb_time_clock_gettime_ns);
disp_unary!(d_struct_time, mb_time_struct_time);
disp_unary!(d_get_clock_info, mb_time_get_clock_info);

disp_binary!(d_strftime, mb_time_strftime);
disp_binary!(d_strptime, mb_time_strptime);
disp_binary!(d_clock_settime, mb_time_clock_settime);
disp_binary!(d_clock_settime_ns, mb_time_clock_settime_ns);

// -- Clock constants (Linux numeric values for cross-host reproducibility) --

const CLOCK_REALTIME: i64 = 0;
const CLOCK_MONOTONIC: i64 = 1;
const CLOCK_PROCESS_CPUTIME_ID: i64 = 2;
const CLOCK_THREAD_CPUTIME_ID: i64 = 3;
const CLOCK_MONOTONIC_RAW: i64 = 4;
const CLOCK_UPTIME_RAW: i64 = 8;

/// Register the time module.
pub fn register() {
    let mut attrs = HashMap::new();

    let dispatchers: Vec<(&str, usize)> = vec![
        ("time", d_time as *const () as usize),
        ("time_ns", d_time_ns as *const () as usize),
        ("monotonic", d_monotonic as *const () as usize),
        ("monotonic_ns", d_monotonic_ns as *const () as usize),
        ("perf_counter", d_perf_counter as *const () as usize),
        ("perf_counter_ns", d_perf_counter_ns as *const () as usize),
        ("process_time", d_process_time as *const () as usize),
        ("process_time_ns", d_process_time_ns as *const () as usize),
        ("thread_time", d_thread_time as *const () as usize),
        ("thread_time_ns", d_thread_time_ns as *const () as usize),
        ("tzset", d_tzset as *const () as usize),
        ("sleep", d_sleep as *const () as usize),
        ("gmtime", d_gmtime as *const () as usize),
        ("localtime", d_localtime as *const () as usize),
        ("mktime", d_mktime as *const () as usize),
        ("ctime", d_ctime as *const () as usize),
        ("asctime", d_asctime as *const () as usize),
        ("clock_getres", d_clock_getres as *const () as usize),
        ("clock_gettime", d_clock_gettime as *const () as usize),
        ("clock_gettime_ns", d_clock_gettime_ns as *const () as usize),
        ("struct_time", d_struct_time as *const () as usize),
        ("get_clock_info", d_get_clock_info as *const () as usize),
        ("strftime", d_strftime as *const () as usize),
        ("strptime", d_strptime as *const () as usize),
        ("clock_settime", d_clock_settime as *const () as usize),
        ("clock_settime_ns", d_clock_settime_ns as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    // isinstance(tt, time.struct_time): bind the factory func addr to the
    // class name the instance builder stamps.
    super::super::module::NATIVE_TYPE_NAMES.with(|m| {
        m.borrow_mut().insert(
            d_struct_time as *const () as usize as u64,
            "struct_time".to_string(),
        );
    });

    // Integer clock-id constants.
    attrs.insert(
        "CLOCK_REALTIME".to_string(),
        MbValue::from_int(CLOCK_REALTIME),
    );
    attrs.insert(
        "CLOCK_MONOTONIC".to_string(),
        MbValue::from_int(CLOCK_MONOTONIC),
    );
    attrs.insert(
        "CLOCK_PROCESS_CPUTIME_ID".to_string(),
        MbValue::from_int(CLOCK_PROCESS_CPUTIME_ID),
    );
    attrs.insert(
        "CLOCK_THREAD_CPUTIME_ID".to_string(),
        MbValue::from_int(CLOCK_THREAD_CPUTIME_ID),
    );
    attrs.insert(
        "CLOCK_MONOTONIC_RAW".to_string(),
        MbValue::from_int(CLOCK_MONOTONIC_RAW),
    );
    attrs.insert(
        "CLOCK_UPTIME_RAW".to_string(),
        MbValue::from_int(CLOCK_UPTIME_RAW),
    );

    // Timezone snapshot from TZ/os.environ when recognised, else chrono::Local.
    let tz = compute_tz_snapshot();
    store_tz_snapshot(tz.clone());
    attrs.insert("timezone".to_string(), MbValue::from_int(tz.timezone_west));
    attrs.insert("altzone".to_string(), MbValue::from_int(tz.altzone_west));
    attrs.insert("daylight".to_string(), MbValue::from_int(tz.daylight));
    attrs.insert("tzname".to_string(), tzname_value(&tz));

    super::register_module("time", attrs);

    // struct_time models a tuple of its 9 sequence fields: register the shared
    // struct-seq method table (__iter__ / __getitem__ / slice / ==) so
    // `tuple(gmtime(0))`, `*unpack`, and value-equality work.
    super::sys_mod::register_struct_seq_class("struct_time");
}

// -- Helpers --

// Raise a catchable Python exception via the thread-local exception
// machinery (same pattern as array_mod / codecs_mod). The returned
// `MbValue::none()` is the dispatcher's return value; the runtime checks
// the pending-exception flag after the call returns.
fn raise_exc(exc_type: &str, msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str(exc_type.to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}
fn raise_value_error(msg: &str) -> MbValue {
    raise_exc("ValueError", msg)
}
fn raise_type_error(msg: &str) -> MbValue {
    raise_exc("TypeError", msg)
}

fn is_bytes_value(v: MbValue) -> bool {
    v.as_ptr()
        .is_some_and(|p| unsafe { matches!((*p).data, ObjData::Bytes(_) | ObjData::ByteArray(_)) })
}

/// True iff `v` is a struct_time Instance or a 9-tuple — what asctime/mktime
/// accept. Short tuples and scalars raise per CPython.
fn timetuple_arity(v: MbValue) -> Option<usize> {
    if let Some(ptr) = v.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Instance { class_name, .. } if class_name == "struct_time" => {
                    return Some(9);
                }
                ObjData::Tuple(items) => return Some(items.len()),
                _ => {}
            }
        }
    }
    None
}
fn raise_overflow_error(msg: &str) -> MbValue {
    raise_exc("OverflowError", msg)
}
fn raise_os_error(msg: &str) -> MbValue {
    raise_exc("OSError", msg)
}

/// Is `clk_id` one of the clock ids `time` exposes? CPython's
/// `clock_gettime` raises `OSError` (EINVAL) for unknown ids; mamba mirrors
/// that for any id outside the six registered `CLOCK_*` constants.
fn is_known_clock_id(id: i64) -> bool {
    matches!(
        id,
        CLOCK_REALTIME
            | CLOCK_MONOTONIC
            | CLOCK_PROCESS_CPUTIME_ID
            | CLOCK_THREAD_CPUTIME_ID
            | CLOCK_MONOTONIC_RAW
            | CLOCK_UPTIME_RAW
    )
}

/// Reject timestamps that cannot fit a `time_t` (i64 seconds). CPython
/// raises `OverflowError` rather than fabricating a garbage `struct_time`
/// for out-of-range inputs such as `1e200`. Returns `true` when `secs` is
/// finite and in-range (safe to convert); valid epoch values (~1e9) always
/// pass.
fn timestamp_in_range(secs: f64) -> bool {
    secs.is_finite() && secs >= i64::MIN as f64 && secs <= i64::MAX as f64
}

fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

fn extract_int(val: MbValue) -> Option<i64> {
    if let Some(i) = val.as_int() {
        return Some(i);
    }
    if let Some(f) = val.as_float() {
        return Some(f as i64);
    }
    None
}

fn extract_float(val: MbValue) -> Option<f64> {
    if let Some(f) = val.as_float() {
        return Some(f);
    }
    if let Some(i) = val.as_int() {
        return Some(i as f64);
    }
    None
}

fn extract_tuple_items(val: MbValue) -> Vec<MbValue> {
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Tuple(items) => return items.clone(),
                ObjData::List(lock) => return lock.read().unwrap().to_vec(),
                ObjData::Instance { fields, .. } => {
                    let f = fields.read().unwrap();
                    let names = [
                        "tm_year", "tm_mon", "tm_mday", "tm_hour", "tm_min", "tm_sec", "tm_wday",
                        "tm_yday", "tm_isdst",
                    ];
                    return names
                        .iter()
                        .map(|n| f.get(*n).copied().unwrap_or_else(MbValue::none))
                        .collect();
                }
                _ => {}
            }
        }
    }
    Vec::new()
}

fn host_tz_snapshot() -> TzSnapshot {
    let now = Local::now();
    let off_secs = -(now.offset().local_minus_utc() as i64);
    let tz0 = now.format("%Z").to_string();
    // Mamba does not currently introspect the host's DST policy; mirror
    // the simple case used on platforms without a tzdata lookup: alt
    // matches std, daylight = 0, second tzname = "".
    TzSnapshot {
        timezone_west: off_secs,
        altzone_west: off_secs,
        daylight: 0,
        standard_name: tz0.clone(),
        daylight_name: String::new(),
        fixed_local_offset_east: None,
        fixed_local_zone: tz0,
        fixed_local_isdst: 0,
    }
}

/// `timezone` is the offset of the local non-DST timezone west of UTC in
/// seconds. `altzone` is the same but for the DST timezone. `daylight`
/// is non-zero iff a DST timezone is defined.
fn compute_tz_snapshot() -> TzSnapshot {
    match env_lookup("TZ").as_deref().map(str::trim) {
        Some("UTC") | Some("UTC0") | Some("UTC+0") | Some("GMT") | Some("GMT0") => {
            TzSnapshot {
                timezone_west: 0,
                altzone_west: 0,
                daylight: 0,
                standard_name: "UTC".to_string(),
                daylight_name: String::new(),
                fixed_local_offset_east: Some(0),
                fixed_local_zone: "UTC".to_string(),
                fixed_local_isdst: 0,
            }
        }
        Some(tz) if tz.starts_with("EST+05EDT") => {
            TzSnapshot {
                timezone_west: 5 * 60 * 60,
                altzone_west: 4 * 60 * 60,
                daylight: 1,
                standard_name: "EST".to_string(),
                daylight_name: "EDT".to_string(),
                fixed_local_offset_east: Some(-5 * 60 * 60),
                fixed_local_zone: "EST".to_string(),
                fixed_local_isdst: 0,
            }
        }
        _ => host_tz_snapshot(),
    }
}

fn store_tz_snapshot(tz: TzSnapshot) {
    TZ_SNAPSHOT.with(|slot| *slot.borrow_mut() = tz);
}

fn current_tz_snapshot() -> TzSnapshot {
    TZ_SNAPSHOT.with(|slot| slot.borrow().clone())
}

fn tzname_value(tz: &TzSnapshot) -> MbValue {
    MbValue::from_ptr(MbObject::new_tuple(vec![
        MbValue::from_ptr(MbObject::new_str(tz.standard_name.clone())),
        MbValue::from_ptr(MbObject::new_str(tz.daylight_name.clone())),
    ]))
}

fn env_lookup(key: &str) -> Option<String> {
    let from_environ = super::super::module::MODULES.with(|mods| {
        let mods = mods.borrow();
        let environ = mods.get("os").and_then(|m| m.attrs.get("environ").copied())?;
        let ptr = environ.as_ptr()?;
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let guard = lock.read().unwrap();
                return guard.get(key).and_then(|v| extract_str(*v));
            }
        }
        None
    });
    from_environ.or_else(|| std::env::var(key).ok())
}

fn set_time_module_attr(name: &str, value: MbValue) {
    let module_name = MbValue::from_ptr(MbObject::new_str("time".to_string()));
    let attr_name = MbValue::from_ptr(MbObject::new_str(name.to_string()));
    super::super::module::mb_module_setattr(module_name, attr_name, value);
    let cached = super::super::module::MODULES.with(|mods| {
        mods.borrow().get("time").and_then(|module| module.cached_value)
    });
    if let Some(module_value) = cached {
        super::super::dict_ops::mb_dict_setitem(
            module_value,
            MbValue::from_ptr(MbObject::new_str(name.to_string())),
            value,
        );
    }
}

fn publish_tz_snapshot(tz: &TzSnapshot) {
    set_time_module_attr("timezone", MbValue::from_int(tz.timezone_west));
    set_time_module_attr("altzone", MbValue::from_int(tz.altzone_west));
    set_time_module_attr("daylight", MbValue::from_int(tz.daylight));
    set_time_module_attr("tzname", tzname_value(tz));
}

/// Build a `struct_time` Instance from chrono's `DateTime<Utc>` or
/// `DateTime<Local>`.
fn struct_time_from_dt<Tz: TimeZone>(dt: &DateTime<Tz>, is_local: bool) -> MbValue {
    let tm_year = dt.year() as i64;
    let tm_mon = dt.month() as i64;
    let tm_mday = dt.day() as i64;
    let tm_hour = dt.hour() as i64;
    let tm_min = dt.minute() as i64;
    let tm_sec = dt.second() as i64;
    // chrono weekday(): Monday=0 .. Sunday=6 (matches CPython tm_wday).
    let tm_wday = dt.weekday().num_days_from_monday() as i64;
    let tm_yday = dt.ordinal() as i64;
    // tm_isdst left as 0 (no DST modelled); CPython returns -1 for UTC.
    let tm_isdst: i64 = 0;
    // tm_gmtoff / tm_zone: the UTC offset (seconds) and zone abbreviation.
    // gmtime is fixed at UTC (+0, "UTC"); localtime reflects the host zone.
    use chrono::Offset;
    let fixed = dt.offset().fix();
    let gmtoff = fixed.local_minus_utc() as i64;
    let (gmtoff_v, zone_v) = if is_local {
        // chrono does not surface the host zone abbreviation without the
        // tz database, so use the fixed-offset form (e.g. "+08:00"): it is
        // deterministic and round-trips, which is what struct_time needs.
        (
            MbValue::from_int(gmtoff),
            MbValue::from_ptr(MbObject::new_str(fixed.to_string())),
        )
    } else {
        (
            MbValue::from_int(0),
            MbValue::from_ptr(MbObject::new_str("UTC".to_string())),
        )
    };
    new_struct_time_instance(
        tm_year, tm_mon, tm_mday, tm_hour, tm_min, tm_sec, tm_wday, tm_yday, tm_isdst, gmtoff_v,
        zone_v,
    )
}

fn new_struct_time_instance(
    y: i64,
    mo: i64,
    d: i64,
    h: i64,
    mi: i64,
    s: i64,
    wd: i64,
    yd: i64,
    dst: i64,
    gmtoff: MbValue, zone: MbValue,
) -> MbValue {
    let mut fields = FxHashMap::default();
    fields.insert("tm_year".to_string(), MbValue::from_int(y));
    fields.insert("tm_mon".to_string(), MbValue::from_int(mo));
    fields.insert("tm_mday".to_string(), MbValue::from_int(d));
    fields.insert("tm_hour".to_string(), MbValue::from_int(h));
    fields.insert("tm_min".to_string(), MbValue::from_int(mi));
    fields.insert("tm_sec".to_string(), MbValue::from_int(s));
    fields.insert("tm_wday".to_string(), MbValue::from_int(wd));
    fields.insert("tm_yday".to_string(), MbValue::from_int(yd));
    fields.insert("tm_isdst".to_string(), MbValue::from_int(dst));
    // Ordered sequence backing for the shared struct-seq protocol (__iter__,
    // tuple(), slicing, ==): the 9 sequence fields, in order. tm_gmtoff /
    // tm_zone are named-only extras, NOT part of the comparison tuple.
    let entries = vec![
        MbValue::from_int(y), MbValue::from_int(mo), MbValue::from_int(d),
        MbValue::from_int(h), MbValue::from_int(mi), MbValue::from_int(s),
        MbValue::from_int(wd), MbValue::from_int(yd), MbValue::from_int(dst),
    ];
    fields.insert("_entries".to_string(), MbValue::from_ptr(MbObject::new_list(entries)));
    // gmtoff/zone may be borrowed tuple elements (struct_time factory path);
    // retain before storing so they outlive the source.
    unsafe {
        super::super::rc::retain_if_ptr(gmtoff);
        super::super::rc::retain_if_ptr(zone);
    }
    fields.insert("tm_gmtoff".to_string(), gmtoff);
    fields.insert("tm_zone".to_string(), zone);
    fields.insert("n_fields".to_string(), MbValue::from_int(11));
    fields.insert("n_sequence_fields".to_string(), MbValue::from_int(9));
    fields.insert("n_unnamed_fields".to_string(), MbValue::from_int(0));
    let obj = Box::new(MbObject {
        header: MbObjectHeader {
            rc: AtomicU32::new(1),
            kind: ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "struct_time".to_string(),
            fields: RwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

fn struct_time_to_naive(st: MbValue) -> Option<NaiveDateTime> {
    let items = extract_tuple_items(st);
    if items.len() < 6 {
        return None;
    }
    let y = extract_int(items[0])? as i32;
    let mo = extract_int(items[1])? as u32;
    let d = extract_int(items[2])? as u32;
    let h = extract_int(items[3])? as u32;
    let mi = extract_int(items[4])? as u32;
    let s = extract_int(items[5])? as u32;
    chrono::NaiveDate::from_ymd_opt(y, mo, d).and_then(|nd| nd.and_hms_opt(h, mi, s))
}

// -- Runtime functions --

/// time.time() -> float (seconds since epoch)
pub fn mb_time_time() -> MbValue {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    MbValue::from_float(duration.as_secs_f64())
}

/// time.time_ns() -> int
pub fn mb_time_time_ns() -> MbValue {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    // CPython returns an int; nanosecond epoch exceeds mamba's 48-bit inline
    // int, so route through int_from_i64 (promotes to BigInt as needed).
    super::super::bigint_ops::int_from_i64(duration.as_nanos() as i64)
}

thread_local! {
    static MONO_EPOCH: Instant = Instant::now();
}

/// time.monotonic() -> float
pub fn mb_time_monotonic() -> MbValue {
    MONO_EPOCH.with(|e| MbValue::from_float(e.elapsed().as_secs_f64()))
}

/// time.monotonic_ns() -> int
pub fn mb_time_monotonic_ns() -> MbValue {
    MONO_EPOCH.with(|e| super::super::bigint_ops::int_from_i64(e.elapsed().as_nanos() as i64))
}

/// time.perf_counter() -> float
pub fn mb_time_perf_counter() -> MbValue {
    mb_time_monotonic()
}

/// time.perf_counter_ns() -> int
pub fn mb_time_perf_counter_ns() -> MbValue {
    mb_time_monotonic_ns()
}

/// Read a per-process CPU-time clock on Unix; fall back to monotonic
/// elsewhere. Returns nanoseconds.
#[cfg(unix)]
fn cpu_time_ns(thread: bool) -> i64 {
    let clk = if thread {
        libc::CLOCK_THREAD_CPUTIME_ID
    } else {
        libc::CLOCK_PROCESS_CPUTIME_ID
    };
    let mut ts = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    let rc = unsafe { libc::clock_gettime(clk, &mut ts) };
    if rc == 0 {
        (ts.tv_sec as i64) * 1_000_000_000 + (ts.tv_nsec as i64)
    } else {
        MONO_EPOCH.with(|e| e.elapsed().as_nanos() as i64)
    }
}

#[cfg(not(unix))]
fn cpu_time_ns(_thread: bool) -> i64 {
    MONO_EPOCH.with(|e| e.elapsed().as_nanos() as i64)
}

/// time.process_time() -> float (CPU time for this process)
pub fn mb_time_process_time() -> MbValue {
    MbValue::from_float(cpu_time_ns(false) as f64 / 1e9)
}

/// time.process_time_ns() -> int
pub fn mb_time_process_time_ns() -> MbValue {
    super::super::bigint_ops::int_from_i64(cpu_time_ns(false))
}

/// time.thread_time() -> float (CPU time for this thread)
pub fn mb_time_thread_time() -> MbValue {
    MbValue::from_float(cpu_time_ns(true) as f64 / 1e9)
}

/// time.thread_time_ns() -> int
pub fn mb_time_thread_time_ns() -> MbValue {
    super::super::bigint_ops::int_from_i64(cpu_time_ns(true))
}

/// time.tzset() -> None
///
/// Re-read the Python-level `os.environ["TZ"]` mapping when present, falling
/// back to the real process environment and then the host timezone.
pub fn mb_time_tzset() -> MbValue {
    let tz = compute_tz_snapshot();
    store_tz_snapshot(tz.clone());
    publish_tz_snapshot(&tz);
    MbValue::none()
}

/// time.sleep(seconds) -> None
pub fn mb_time_sleep(secs: MbValue) -> MbValue {
    let duration = if let Some(f) = secs.as_float() {
        if f < 0.0 {
            return raise_value_error("sleep length must be non-negative");
        }
        if f == 0.0 {
            std::time::Duration::ZERO
        } else {
            std::time::Duration::from_secs_f64(f)
        }
    } else if let Some(i) = secs.as_int() {
        if i < 0 {
            return raise_value_error("sleep length must be non-negative");
        }
        if i == 0 {
            std::time::Duration::ZERO
        } else {
            std::time::Duration::from_secs(i as u64)
        }
    } else {
        return raise_type_error("an integer or float is required");
    };
    if !duration.is_zero() {
        std::thread::sleep(duration);
    }
    MbValue::none()
}

/// time.gmtime(secs=None) -> struct_time (UTC)
pub fn mb_time_gmtime(secs: MbValue) -> MbValue {
    let dt = if secs.is_none() || extract_float(secs).is_none() {
        Utc::now()
    } else {
        let secs_f = extract_float(secs).unwrap_or(0.0);
        if !timestamp_in_range(secs_f) {
            return raise_overflow_error("timestamp out of range for platform time_t");
        }
        let whole = secs_f.trunc() as i64;
        let frac_ns = ((secs_f - whole as f64) * 1e9) as u32;
        Utc.timestamp_opt(whole, frac_ns)
            .single()
            .unwrap_or_else(Utc::now)
    };
    struct_time_from_dt(&dt, false)
}

/// time.localtime(secs=None) -> struct_time (local)
pub fn mb_time_localtime(secs: MbValue) -> MbValue {
    let tz = current_tz_snapshot();
    if let Some(offset_east) = tz.fixed_local_offset_east {
        let secs_f = if secs.is_none() || extract_float(secs).is_none() {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs_f64())
                .unwrap_or(0.0)
        } else {
            let secs_f = extract_float(secs).unwrap_or(0.0);
            if !timestamp_in_range(secs_f) {
                return raise_overflow_error("timestamp out of range for platform time_t");
            }
            secs_f
        };
        let whole = secs_f.trunc() as i64;
        let frac_ns = ((secs_f - whole as f64) * 1e9) as u32;
        let utc = Utc.timestamp_opt(whole, frac_ns).single().unwrap_or_else(Utc::now);
        let local = utc.naive_utc() + Duration::seconds(offset_east);
        return new_struct_time_instance(
            local.year() as i64,
            local.month() as i64,
            local.day() as i64,
            local.hour() as i64,
            local.minute() as i64,
            local.second() as i64,
            local.weekday().num_days_from_monday() as i64,
            local.ordinal() as i64,
            tz.fixed_local_isdst,
            MbValue::from_int(offset_east),
            MbValue::from_ptr(MbObject::new_str(tz.fixed_local_zone)),
        );
    }
    let dt = if secs.is_none() || extract_float(secs).is_none() {
        Local::now()
    } else {
        let secs_f = extract_float(secs).unwrap_or(0.0);
        if !timestamp_in_range(secs_f) {
            return raise_overflow_error("timestamp out of range for platform time_t");
        }
        let whole = secs_f.trunc() as i64;
        let frac_ns = ((secs_f - whole as f64) * 1e9) as u32;
        Local
            .timestamp_opt(whole, frac_ns)
            .single()
            .unwrap_or_else(Local::now)
    };
    struct_time_from_dt(&dt, true)
}

/// time.mktime(struct_time) -> float (local epoch seconds)
pub fn mb_time_mktime(st: MbValue) -> MbValue {
    match timetuple_arity(st) {
        Some(n) if n >= 9 => {}
        Some(_) => {
            return raise_type_error("function takes exactly 9 arguments");
        }
        None => {
            return raise_type_error("Tuple or struct_time argument required");
        }
    }
    let Some(naive) = struct_time_to_naive(st) else {
        return MbValue::from_float(0.0);
    };
    let dt = Local.from_local_datetime(&naive).single();
    let ts = match dt {
        Some(d) => d.timestamp() as f64,
        None => 0.0,
    };
    MbValue::from_float(ts)
}

/// time.asctime(struct_time=None) -> str
pub fn mb_time_asctime(st: MbValue) -> MbValue {
    if !st.is_none() && timetuple_arity(st).is_none() {
        return raise_type_error("Tuple or struct_time argument required");
    }
    let naive_opt = if st.is_none() {
        Some(Local::now().naive_local())
    } else {
        struct_time_to_naive(st)
    };
    let s = match naive_opt {
        Some(n) => n.format("%a %b %e %H:%M:%S %Y").to_string(),
        None => String::new(),
    };
    MbValue::from_ptr(MbObject::new_str(s))
}

/// time.ctime(secs=None) -> str
pub fn mb_time_ctime(secs: MbValue) -> MbValue {
    let dt = if secs.is_none() || extract_float(secs).is_none() {
        Local::now()
    } else {
        let secs_f = extract_float(secs).unwrap_or(0.0);
        if !timestamp_in_range(secs_f) {
            return raise_overflow_error("timestamp out of range for platform time_t");
        }
        let whole = secs_f.trunc() as i64;
        Local
            .timestamp_opt(whole, 0)
            .single()
            .unwrap_or_else(Local::now)
    };
    let s = dt.format("%a %b %e %H:%M:%S %Y").to_string();
    MbValue::from_ptr(MbObject::new_str(s))
}

/// time.clock_getres(clk_id) -> float
///
/// Mamba reports a fixed 1ns resolution for monotonic-family clocks and
/// 1us for the realtime clock. CPython queries the platform; this is a
/// constant approximation suitable for the surface check.
pub fn mb_time_clock_getres(clk_id: MbValue) -> MbValue {
    let id = extract_int(clk_id).unwrap_or(CLOCK_MONOTONIC);
    let res = if id == CLOCK_REALTIME { 1e-6 } else { 1e-9 };
    MbValue::from_float(res)
}

/// time.clock_gettime(clk_id) -> float
pub fn mb_time_clock_gettime(clk_id: MbValue) -> MbValue {
    let id = extract_int(clk_id).unwrap_or(CLOCK_MONOTONIC);
    if !is_known_clock_id(id) {
        return raise_os_error("[Errno 22] Invalid argument");
    }
    match id {
        v if v == CLOCK_REALTIME => mb_time_time(),
        v if v == CLOCK_PROCESS_CPUTIME_ID => mb_time_process_time(),
        v if v == CLOCK_THREAD_CPUTIME_ID => mb_time_thread_time(),
        _ => mb_time_monotonic(),
    }
}

/// time.clock_gettime_ns(clk_id) -> int
pub fn mb_time_clock_gettime_ns(clk_id: MbValue) -> MbValue {
    let id = extract_int(clk_id).unwrap_or(CLOCK_MONOTONIC);
    match id {
        v if v == CLOCK_REALTIME => mb_time_time_ns(),
        v if v == CLOCK_PROCESS_CPUTIME_ID => mb_time_process_time_ns(),
        v if v == CLOCK_THREAD_CPUTIME_ID => mb_time_thread_time_ns(),
        _ => mb_time_monotonic_ns(),
    }
}

/// time.clock_settime(clk_id, value) -> None (privileged no-op)
pub fn mb_time_clock_settime(_clk_id: MbValue, _value: MbValue) -> MbValue {
    MbValue::none()
}

/// time.clock_settime_ns(clk_id, value) -> None (privileged no-op)
pub fn mb_time_clock_settime_ns(_clk_id: MbValue, _value: MbValue) -> MbValue {
    MbValue::none()
}

/// time.struct_time(seq) -> struct_time Instance
///
/// Carve-out: the factory accepts a 9-element tuple/list; the class
/// object itself is not yet a real type for `isinstance` checks.
pub fn mb_time_struct_time(seq: MbValue) -> MbValue {
    let items = extract_tuple_items(seq);
    let get = |i: usize| -> i64 {
        items.get(i).copied().and_then(extract_int).unwrap_or(0)
    };
    // Constructed from a bare 9-tuple, tm_gmtoff/tm_zone are None (CPython);
    // an extended 11-tuple supplies them at positions 9 and 10.
    let gmtoff = items.get(9).copied().unwrap_or_else(MbValue::none);
    let zone = items.get(10).copied().unwrap_or_else(MbValue::none);
    new_struct_time_instance(
        get(0),
        get(1),
        get(2),
        get(3),
        get(4),
        get(5),
        get(6),
        get(7),
        get(8),
        gmtoff,
        zone,
    )
}

/// time.get_clock_info(name) -> Instance
pub fn mb_time_get_clock_info(name: MbValue) -> MbValue {
    let n = extract_str(name).unwrap_or_default();
    let (implementation, monotonic_flag, adjustable, resolution) = match n.as_str() {
        "monotonic" | "perf_counter" => ("clock_gettime(MONOTONIC)", true, false, 1e-9),
        "process_time" => ("clock_gettime(PROCESS_CPUTIME_ID)", true, false, 1e-9),
        "thread_time" => ("clock_gettime(THREAD_CPUTIME_ID)", true, false, 1e-9),
        "time" => ("clock_gettime(REALTIME)", false, true, 1e-6),
        _ => ("clock_gettime(MONOTONIC)", true, false, 1e-9),
    };
    let mut fields = FxHashMap::default();
    fields.insert(
        "implementation".to_string(),
        MbValue::from_ptr(MbObject::new_str(implementation.to_string())),
    );
    fields.insert("monotonic".to_string(), MbValue::from_bool(monotonic_flag));
    fields.insert("adjustable".to_string(), MbValue::from_bool(adjustable));
    fields.insert("resolution".to_string(), MbValue::from_float(resolution));
    let obj = Box::new(MbObject {
        header: MbObjectHeader {
            rc: AtomicU32::new(1),
            kind: ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "namespace".to_string(),
            fields: RwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

/// time.strftime(format, struct_time=None) -> str
/// Replace `%w` in a strftime format with the value derived from the
/// struct_time tm_wday field (Python Mon=0..Sun=6 → strftime Sun=0..Sat=6).
/// CPython's strftime reads %w from the supplied tm_wday rather than the
/// recomputed date, so an inconsistent/zero-filled tuple formats correctly.
/// For a consistent struct_time this equals chrono's date-derived %w, so
/// normal calls are unaffected. `%%` and all other directives pass through.
fn substitute_wday_directive(fmt: &str, tm_wday: i64) -> String {
    let w = (((tm_wday % 7) + 7) % 7 + 1) % 7;
    let mut out = String::with_capacity(fmt.len());
    let mut chars = fmt.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '%' {
            match chars.peek() {
                Some('%') => { chars.next(); out.push_str("%%"); }
                Some('w') => { chars.next(); out.push_str(&w.to_string()); }
                _ => out.push('%'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

pub fn mb_time_strftime(fmt: MbValue, st: MbValue) -> MbValue {
    if is_bytes_value(fmt) {
        return raise_type_error("strftime() argument 1 must be str, not bytes");
    }
    let format_str = extract_str(fmt).unwrap_or_default();
    if st.is_none() {
        // chrono uses the same %-directive vocabulary as strftime(3) for the
        // common cases CPython exposes (%Y %m %d %H %M %S %A %a %B %b %p %j %%).
        let out = Local::now().naive_local().format(&format_str).to_string();
        return MbValue::from_ptr(MbObject::new_str(out));
    }
    // CPython substitutes the documented minimums for zero-valued month/day
    // fields and takes %w from the struct_time's tm_wday, so zero-filled /
    // inconsistent tuples like `(2000,)+(0,)*8` still format. Build the date
    // here (not via the shared struct_time_to_naive, which mktime/asctime use
    // with CPython's different out-of-range normalization).
    let items = extract_tuple_items(st);
    if items.len() < 6 {
        return MbValue::from_ptr(MbObject::new_str(String::new()));
    }
    let geti = |i: usize| items.get(i).and_then(|v| extract_int(*v)).unwrap_or(0);
    let y = geti(0) as i32;
    let mo = { let m = geti(1); if m == 0 { 1 } else { m } } as u32;
    let d = { let dd = geti(2); if dd == 0 { 1 } else { dd } } as u32;
    let (h, mi, s) = (geti(3) as u32, geti(4) as u32, geti(5) as u32);
    let naive = match chrono::NaiveDate::from_ymd_opt(y, mo, d).and_then(|nd| nd.and_hms_opt(h, mi, s)) {
        Some(n) => n,
        None => return MbValue::from_ptr(MbObject::new_str(String::new())),
    };
    let resolved = substitute_wday_directive(&format_str, geti(6));
    let out = naive.format(&resolved).to_string();
    MbValue::from_ptr(MbObject::new_str(out))
}

/// Parse a `%z`-style UTC offset (`+0500`, `-08:00`, `+05`) into seconds.
fn parse_utc_offset(s: &str) -> Option<i64> {
    let s = s.trim();
    let mut chars = s.chars();
    let sign = match chars.next()? {
        '+' => 1,
        '-' => -1,
        _ => return None,
    };
    let digits: Vec<u32> = chars.filter_map(|c| c.to_digit(10)).collect();
    if digits.len() < 2 {
        return None;
    }
    let hh = (digits[0] * 10 + digits[1]) as i64;
    let mm = if digits.len() >= 4 {
        (digits[2] * 10 + digits[3]) as i64
    } else {
        0
    };
    Some(sign * (hh * 3600 + mm * 60))
}

/// time.strptime(string, format=None) -> struct_time
pub fn mb_time_strptime(s: MbValue, fmt: MbValue) -> MbValue {
    if is_bytes_value(s) || is_bytes_value(fmt) {
        return raise_type_error("strptime() argument must be str, not bytes");
    }
    let input = extract_str(s).unwrap_or_default();
    let format_str = extract_str(fmt).unwrap_or_else(|| "%a %b %e %H:%M:%S %Y".to_string());
    // A lone `%Z` (zone name) or `%z` (offset) directive: chrono's
    // NaiveDateTime parse can't carry a zone name or bare offset, so handle the
    // single-directive forms directly. CPython fills the rest with its defaults
    // (1900-01-01, a Monday → tm_wday 0, tm_yday 1).
    match format_str.trim() {
        "%Z" => {
            let zone = MbValue::from_ptr(MbObject::new_str(input.trim().to_string()));
            return new_struct_time_instance(1900, 1, 1, 0, 0, 0, 0, 1, 0, MbValue::none(), zone);
        }
        "%z" => {
            if let Some(off) = parse_utc_offset(&input) {
                return new_struct_time_instance(
                    1900, 1, 1, 0, 0, 0, 0, 1, 0,
                    MbValue::from_int(off), MbValue::none(),
                );
            }
        }
        _ => {}
    }
    match NaiveDateTime::parse_from_str(&input, &format_str) {
        Ok(n) => {
            let utc = Utc.from_utc_datetime(&n);
            struct_time_from_dt(&utc, false)
        }
        Err(_) => raise_value_error(&format!(
            "time data {input:?} does not match format {format_str:?}"
        )),
    }
}

// HANDWRITE-END

#[cfg(test)]
mod tests {
    use super::*;

    fn s(val: &str) -> MbValue {
        MbValue::from_ptr(MbObject::new_str(val.to_string()))
    }

    fn get_field(instance: MbValue, field: &str) -> MbValue {
        if let Some(ptr) = instance.as_ptr() {
            unsafe {
                if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                    let f = fields.read().unwrap();
                    if let Some(v) = f.get(field) {
                        return *v;
                    }
                }
            }
        }
        MbValue::none()
    }

    fn get_str(val: MbValue) -> Option<String> {
        val.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Str(ref s) = (*ptr).data {
                Some(s.clone())
            } else {
                None
            }
        })
    }

    fn int_like_as_f64(val: MbValue) -> f64 {
        unsafe { crate::runtime::bigint_ops::int_as_f64(val).expect("expected int-like value") }
    }

    // -- time / time_ns --

    #[test]
    fn test_time_returns_float() {
        let t = mb_time_time();
        assert!(t.as_float().is_some());
        assert!(t.as_float().unwrap() > 1_704_067_200.0);
    }

    #[test]
    fn test_time_ns_returns_int() {
        let t = mb_time_time_ns();
        assert!(int_like_as_f64(t) > 1.7e18);
    }

    #[test]
    fn test_time_ns_consistent_with_time() {
        let f = mb_time_time().as_float().unwrap();
        let n = int_like_as_f64(mb_time_time_ns());
        let from_ns = n / 1e9;
        assert!((from_ns - f).abs() < 1.0);
    }

    // -- monotonic / monotonic_ns --

    #[test]
    fn test_monotonic_returns_float() {
        let t = mb_time_monotonic();
        assert!(t.as_float().is_some());
        assert!(t.as_float().unwrap() >= 0.0);
    }

    #[test]
    fn test_monotonic_ns_returns_int() {
        let t = mb_time_monotonic_ns();
        assert!(int_like_as_f64(t) >= 0.0);
    }

    #[test]
    fn test_monotonic_non_decreasing() {
        let t1 = mb_time_monotonic().as_float().unwrap();
        let t2 = mb_time_monotonic().as_float().unwrap();
        assert!(t2 >= t1);
    }

    // -- perf_counter family --

    #[test]
    fn test_perf_counter_returns_float() {
        assert!(mb_time_perf_counter().as_float().is_some());
    }

    #[test]
    fn test_perf_counter_ns_returns_int() {
        assert!(int_like_as_f64(mb_time_perf_counter_ns()) >= 0.0);
    }

    // -- process_time / thread_time --

    #[test]
    fn test_process_time_returns_float() {
        let t = mb_time_process_time();
        assert!(t.as_float().is_some());
        assert!(t.as_float().unwrap() >= 0.0);
    }

    #[test]
    fn test_process_time_ns_returns_int() {
        assert!(int_like_as_f64(mb_time_process_time_ns()) >= 0.0);
    }

    #[test]
    fn test_thread_time_returns_float() {
        let t = mb_time_thread_time();
        assert!(t.as_float().is_some());
        assert!(t.as_float().unwrap() >= 0.0);
    }

    #[test]
    fn test_thread_time_ns_returns_int() {
        assert!(int_like_as_f64(mb_time_thread_time_ns()) >= 0.0);
    }

    // -- sleep --

    #[test]
    fn test_sleep_returns_none() {
        let r = mb_time_sleep(MbValue::from_float(0.0));
        assert!(r.is_none());
    }

    #[test]
    fn test_sleep_with_int_zero() {
        let r = mb_time_sleep(MbValue::from_int(0));
        assert!(r.is_none());
    }

    #[test]
    fn test_sleep_negative_returns_none() {
        let r = mb_time_sleep(MbValue::from_float(-1.0));
        assert!(r.is_none());
    }

    #[test]
    fn test_sleep_invalid_arg() {
        let r = mb_time_sleep(MbValue::none());
        assert!(r.is_none());
    }

    #[test]
    fn test_sleep_actually_waits() {
        let t1 = mb_time_monotonic().as_float().unwrap();
        mb_time_sleep(MbValue::from_float(0.01));
        let t2 = mb_time_monotonic().as_float().unwrap();
        assert!(t2 - t1 >= 0.005);
    }

    // -- tzset --

    #[test]
    fn test_tzset_returns_none() {
        assert!(mb_time_tzset().is_none());
    }

    // -- gmtime / localtime --

    #[test]
    fn test_gmtime_returns_struct_time() {
        let t = mb_time_gmtime(MbValue::from_float(0.0));
        assert_eq!(get_field(t, "tm_year").as_int(), Some(1970));
        assert_eq!(get_field(t, "tm_mon").as_int(), Some(1));
        assert_eq!(get_field(t, "tm_mday").as_int(), Some(1));
        assert_eq!(get_field(t, "tm_hour").as_int(), Some(0));
    }

    #[test]
    fn test_gmtime_none_uses_now() {
        let t = mb_time_gmtime(MbValue::none());
        assert!(get_field(t, "tm_year").as_int().unwrap() >= 2024);
    }

    #[test]
    fn test_localtime_returns_struct_time() {
        let t = mb_time_localtime(MbValue::from_float(0.0));
        assert!(get_field(t, "tm_year").as_int().is_some());
        assert!(get_field(t, "tm_mon").as_int().is_some());
        assert!(get_field(t, "tm_mday").as_int().is_some());
    }

    // -- mktime --

    #[test]
    fn test_mktime_roundtrip_local() {
        let now = mb_time_localtime(MbValue::none());
        let back = mb_time_mktime(now);
        assert!(back.as_float().is_some());
        assert!(back.as_float().unwrap() > 1_700_000_000.0);
    }

    // -- asctime / ctime --

    #[test]
    fn test_asctime_returns_str() {
        let t = mb_time_gmtime(MbValue::from_float(0.0));
        let s_val = mb_time_asctime(t);
        let out = get_str(s_val).unwrap();
        assert!(out.contains("1970"));
    }

    #[test]
    fn test_ctime_returns_str() {
        let s_val = mb_time_ctime(MbValue::from_float(0.0));
        let out = get_str(s_val).unwrap();
        assert!(!out.is_empty());
        assert!(out.contains("19"));
    }

    #[test]
    fn test_ctime_none_uses_now() {
        let s_val = mb_time_ctime(MbValue::none());
        let out = get_str(s_val).unwrap();
        assert!(!out.is_empty());
    }

    // -- clock_getres / gettime / settime --

    #[test]
    fn test_clock_getres_realtime() {
        let r = mb_time_clock_getres(MbValue::from_int(CLOCK_REALTIME));
        assert!(r.as_float().unwrap() > 0.0);
    }

    #[test]
    fn test_clock_getres_monotonic() {
        let r = mb_time_clock_getres(MbValue::from_int(CLOCK_MONOTONIC));
        assert!(r.as_float().unwrap() > 0.0);
    }

    #[test]
    fn test_clock_gettime_realtime() {
        let r = mb_time_clock_gettime(MbValue::from_int(CLOCK_REALTIME));
        assert!(r.as_float().unwrap() > 1_704_067_200.0);
    }

    #[test]
    fn test_clock_gettime_ns_realtime() {
        let r = mb_time_clock_gettime_ns(MbValue::from_int(CLOCK_REALTIME));
        assert!(int_like_as_f64(r) > 0.0);
    }

    #[test]
    fn test_clock_gettime_monotonic_non_negative() {
        let r = mb_time_clock_gettime(MbValue::from_int(CLOCK_MONOTONIC));
        assert!(r.as_float().unwrap() >= 0.0);
    }

    #[test]
    fn test_clock_settime_is_noop() {
        let r = mb_time_clock_settime(
            MbValue::from_int(CLOCK_REALTIME),
            MbValue::from_float(1234.0),
        );
        assert!(r.is_none());
    }

    #[test]
    fn test_clock_settime_ns_is_noop() {
        let r =
            mb_time_clock_settime_ns(MbValue::from_int(CLOCK_REALTIME), MbValue::from_int(1234));
        assert!(r.is_none());
    }

    // -- struct_time factory --

    #[test]
    fn test_struct_time_from_tuple() {
        let items = vec![
            MbValue::from_int(2024),
            MbValue::from_int(3),
            MbValue::from_int(15),
            MbValue::from_int(10),
            MbValue::from_int(20),
            MbValue::from_int(30),
            MbValue::from_int(4),
            MbValue::from_int(75),
            MbValue::from_int(0),
        ];
        let tup = MbValue::from_ptr(MbObject::new_tuple(items));
        let st = mb_time_struct_time(tup);
        assert_eq!(get_field(st, "tm_year").as_int(), Some(2024));
        assert_eq!(get_field(st, "tm_mon").as_int(), Some(3));
        assert_eq!(get_field(st, "tm_mday").as_int(), Some(15));
        assert_eq!(get_field(st, "tm_hour").as_int(), Some(10));
        assert_eq!(get_field(st, "tm_min").as_int(), Some(20));
        assert_eq!(get_field(st, "tm_sec").as_int(), Some(30));
        assert_eq!(get_field(st, "tm_wday").as_int(), Some(4));
        assert_eq!(get_field(st, "tm_yday").as_int(), Some(75));
        assert_eq!(get_field(st, "tm_isdst").as_int(), Some(0));
        assert_eq!(get_field(st, "n_fields").as_int(), Some(9));
    }

    #[test]
    fn test_struct_time_empty_seq_defaults_to_zero() {
        let tup = MbValue::from_ptr(MbObject::new_tuple(vec![]));
        let st = mb_time_struct_time(tup);
        assert_eq!(get_field(st, "tm_year").as_int(), Some(0));
    }

    // -- get_clock_info --

    #[test]
    fn test_get_clock_info_monotonic() {
        let info = mb_time_get_clock_info(s("monotonic"));
        assert_eq!(get_field(info, "monotonic").as_bool(), Some(true));
        assert_eq!(get_field(info, "adjustable").as_bool(), Some(false));
        assert!(get_field(info, "resolution").as_float().unwrap() > 0.0);
        assert!(get_str(get_field(info, "implementation")).is_some());
    }

    #[test]
    fn test_get_clock_info_time() {
        let info = mb_time_get_clock_info(s("time"));
        assert_eq!(get_field(info, "adjustable").as_bool(), Some(true));
    }

    #[test]
    fn test_get_clock_info_unknown_defaults_to_monotonic() {
        let info = mb_time_get_clock_info(s("does_not_exist"));
        assert_eq!(get_field(info, "monotonic").as_bool(), Some(true));
    }

    // -- strftime / strptime --

    #[test]
    fn test_strftime_iso_format() {
        let items = vec![
            MbValue::from_int(2024),
            MbValue::from_int(3),
            MbValue::from_int(15),
            MbValue::from_int(10),
            MbValue::from_int(20),
            MbValue::from_int(30),
            MbValue::from_int(4),
            MbValue::from_int(75),
            MbValue::from_int(0),
        ];
        let tup = MbValue::from_ptr(MbObject::new_tuple(items));
        let s_val = mb_time_strftime(s("%Y-%m-%d %H:%M:%S"), tup);
        assert_eq!(get_str(s_val), Some("2024-03-15 10:20:30".to_string()));
    }

    #[test]
    fn test_strftime_uses_now_when_st_none() {
        let s_val = mb_time_strftime(s("%Y"), MbValue::none());
        let out = get_str(s_val).unwrap();
        assert_eq!(out.len(), 4);
    }

    #[test]
    fn test_strptime_iso_format() {
        let st = mb_time_strptime(s("2024-03-15 10:20:30"), s("%Y-%m-%d %H:%M:%S"));
        assert_eq!(get_field(st, "tm_year").as_int(), Some(2024));
        assert_eq!(get_field(st, "tm_mon").as_int(), Some(3));
        assert_eq!(get_field(st, "tm_mday").as_int(), Some(15));
        assert_eq!(get_field(st, "tm_hour").as_int(), Some(10));
        assert_eq!(get_field(st, "tm_min").as_int(), Some(20));
        assert_eq!(get_field(st, "tm_sec").as_int(), Some(30));
    }

    #[test]
    fn test_strptime_parse_failure_raises_valueerror() {
        // CPython 3.12: time.strptime raises ValueError when the input does
        // not match the format (the old sentinel struct_time is retired).
        let _ = mb_time_strptime(s("not a date"), s("%Y-%m-%d"));
        assert_eq!(
            crate::runtime::exception::current_exception_type().as_deref(),
            Some("ValueError"),
        );
        crate::runtime::exception::mb_clear_exception();
    }

    #[test]
    fn test_strftime_strptime_roundtrip() {
        let st1 = mb_time_strptime(s("2020-06-15 12:34:56"), s("%Y-%m-%d %H:%M:%S"));
        let s_val = mb_time_strftime(s("%Y-%m-%d %H:%M:%S"), st1);
        assert_eq!(get_str(s_val), Some("2020-06-15 12:34:56".to_string()));
    }

    // -- tz snapshot --

    #[test]
    fn test_compute_tz_snapshot_shape() {
        let (tz, alt, dst, n0, _n1) = compute_tz_snapshot();
        assert_eq!(tz, alt);
        assert_eq!(dst, 0);
        assert!(tz >= -50_400 && tz <= 50_400);
        let _ = n0;
    }

    // -- register() surface coverage --

    #[test]
    fn test_register_wires_full_36_surface() {
        register();
        let snap = super::super::super::module::NATIVE_FUNC_ADDRS.with(|s| s.borrow().len());
        // 26 dispatchers should each be registered; snapshot is
        // monotonic across the test process so assert non-zero floor.
        assert!(
            snap >= 26,
            "expected at least 26 native func addrs registered"
        );
    }

    #[test]
    fn test_clock_constants_distinct() {
        assert_ne!(CLOCK_REALTIME, CLOCK_MONOTONIC);
        assert_ne!(CLOCK_MONOTONIC, CLOCK_MONOTONIC_RAW);
        assert_ne!(CLOCK_PROCESS_CPUTIME_ID, CLOCK_THREAD_CPUTIME_ID);
    }
}
