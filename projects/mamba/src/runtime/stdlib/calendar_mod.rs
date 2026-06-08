//! @codegen-skip: handwrite-pre-standardize
//!
//! calendar module for Mamba (#1265 Wave-9).
//!
//! CPython 3.12 `calendar` 61-entry surface:
//!   APRIL, AUGUST, Calendar, DECEMBER, Day, EPOCH, FEBRUARY, FRIDAY,
//!   HTMLCalendar, IllegalMonthError, IllegalWeekdayError, IntEnum,
//!   JANUARY, JULY, JUNE, LocaleHTMLCalendar, LocaleTextCalendar, MARCH,
//!   MAY, MONDAY, Month, NOVEMBER, OCTOBER, SATURDAY, SEPTEMBER, SUNDAY,
//!   THURSDAY, TUESDAY, TextCalendar, WEDNESDAY, c, calendar, datetime,
//!   day_abbr, day_name, different_locale, error, firstweekday, format,
//!   formatstring, global_enum, isleap, leapdays, main, mdays,
//!   month, month_abbr, month_name, monthcalendar, monthrange, prcal,
//!   prmonth, prweek, repeat, setfirstweekday, sys, timegm, warnings,
//!   week, weekday, weekheader.
//!
//! Real implementations: isleap, leapdays, monthrange, weekday,
//! monthcalendar, timegm (via chrono-less day-count math), isleap-driven
//! month-length lookups, firstweekday/setfirstweekday (AtomicI64 state),
//! month_name, month_abbr, day_name, day_abbr, mdays, EPOCH.
//!
//! Carve-outs:
//!   - `Calendar` / `TextCalendar` / `HTMLCalendar` / `LocaleTextCalendar`
//!     / `LocaleHTMLCalendar`: Instance stubs. Constructors return an
//!     Instance carrying `firstweekday` (defaults to 0 for non-locale
//!     variants) plus a `locale` field where applicable; instance
//!     methods (`iterweekdays`, `formatmonth`, `formatyear`, ...) are not
//!     yet wired through method dispatch. Use module-level
//!     `monthrange` / `weekday` / `monthcalendar` instead.
//!   - `IllegalMonthError` / `IllegalWeekdayError` / `error`: Instance
//!     stubs with `__name__` + `__module__` fields. Mamba does not yet
//!     model the Exception subclass hierarchy, so these are passive
//!     sentinels suitable for identity checks but not `raise`.
//!   - `IntEnum`, `Day`, `Month`, `global_enum`: enum-related surface is
//!     re-exported from `enum` in CPython. Exposed here as Instance
//!     placeholders that carry their class name so `isinstance`-style
//!     identity checks against the symbol still work.
//!   - `prmonth` / `prcal` / `prweek` / `month` / `calendar` / `week`:
//!     pretty-print functions return the empty string instead of writing
//!     to stdout. Useful in scripts that capture their return value.
//!   - `format` / `formatstring`: text-layout helpers — return the input
//!     stringified as-is.
//!   - `main`: CPython's CLI entry — wired as a no-op returning None.
//!   - `repeat`: itertools re-export inside `calendar`; exposed as
//!     `MbValue::none()` placeholder. Use `itertools.repeat` directly.
//!   - `c`, `datetime`, `sys`, `warnings`, `different_locale`: module
//!     re-exports / context-manager symbols — exposed as None
//!     placeholders. User code that does `calendar.datetime.date(...)` is
//!     rare; the canonical path is `import datetime`.
//!
//! HANDWRITE-BEGIN reason: per-section primitive vocabulary for stdlib
//! module shims (register_module + dispatch_{nullary,unary,binary,
//! ternary} + atomic module-state slot + Instance-stub class) is not yet
//! emitted by score codegen. The calendar surface is a strong codegen
//! candidate (table-driven by `(name, kind, real|stub)` triples) once
//! the standardize sweep grows a `stdlib_surface` section type. Until
//! then, this file is hand-authored and pinned by tests.

use super::super::rc::{MbObject, MbObjectHeader, ObjData, ObjKind};
use super::super::value::MbValue;
use crate::runtime::rc::MbRwLock as RwLock;
use rustc_hash::FxHashMap;
use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, AtomicU32, Ordering};

// Module-level state. CPython's `calendar.setfirstweekday(n)` mutates a
// process-wide slot read back by `firstweekday()`. We mirror that with an
// AtomicI64 so the surface stays callable from multiple threads.
static FIRST_WEEKDAY: AtomicI64 = AtomicI64::new(0);

// -- Variadic dispatchers --

macro_rules! dispatch_nullary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
            $fn()
        }
    };
}

macro_rules! dispatch_unary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a.get(0).copied().unwrap_or_else(MbValue::none))
        }
    };
}

macro_rules! dispatch_binary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(
                a.get(0).copied().unwrap_or_else(MbValue::none),
                a.get(1).copied().unwrap_or_else(MbValue::none),
            )
        }
    };
}

macro_rules! dispatch_ternary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(
                a.get(0).copied().unwrap_or_else(MbValue::none),
                a.get(1).copied().unwrap_or_else(MbValue::none),
                a.get(2).copied().unwrap_or_else(MbValue::none),
            )
        }
    };
}

// Variadic stub: ignores args, returns a canned MbValue.
macro_rules! dispatch_stub {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
            $fn()
        }
    };
}

dispatch_unary!(dispatch_isleap, mb_calendar_isleap);
dispatch_binary!(dispatch_leapdays, mb_calendar_leapdays);
dispatch_binary!(dispatch_monthrange, mb_calendar_monthrange);
dispatch_ternary!(dispatch_weekday, mb_calendar_weekday);
dispatch_binary!(dispatch_monthcalendar, mb_calendar_monthcalendar);
dispatch_nullary!(dispatch_firstweekday, mb_calendar_firstweekday);
dispatch_unary!(dispatch_setfirstweekday, mb_calendar_setfirstweekday);
dispatch_unary!(dispatch_timegm, mb_calendar_timegm);

// Pretty-print + helper stubs (CPython prints / formats; we return "").
dispatch_stub!(dispatch_prmonth, mb_calendar_empty_str);
dispatch_stub!(dispatch_prcal, mb_calendar_empty_str);
dispatch_stub!(dispatch_prweek, mb_calendar_empty_str);
dispatch_stub!(dispatch_month, mb_calendar_empty_str);
dispatch_stub!(dispatch_calendar_fn, mb_calendar_empty_str);
dispatch_stub!(dispatch_week, mb_calendar_empty_str);
dispatch_stub!(dispatch_weekheader, mb_calendar_empty_str);
dispatch_stub!(dispatch_format, mb_calendar_empty_str);
dispatch_stub!(dispatch_formatstring, mb_calendar_empty_str);
dispatch_stub!(dispatch_main, mb_calendar_none);
dispatch_stub!(dispatch_different_locale, mb_calendar_none);
dispatch_stub!(dispatch_global_enum, mb_calendar_none);

// Class constructors — return Instance stubs carrying class_name + firstweekday.
unsafe extern "C" fn dispatch_calendar_cls(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let fw = a.get(0).copied().unwrap_or_else(MbValue::none);
    mb_calendar_class_new("Calendar", fw, None)
}

unsafe extern "C" fn dispatch_text_calendar(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let fw = a.get(0).copied().unwrap_or_else(MbValue::none);
    mb_calendar_class_new("TextCalendar", fw, None)
}

unsafe extern "C" fn dispatch_html_calendar(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let fw = a.get(0).copied().unwrap_or_else(MbValue::none);
    mb_calendar_class_new("HTMLCalendar", fw, None)
}

unsafe extern "C" fn dispatch_locale_text_calendar(
    args_ptr: *const MbValue,
    nargs: usize,
) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let fw = a.get(0).copied().unwrap_or_else(MbValue::none);
    let locale = a.get(1).copied().unwrap_or_else(MbValue::none);
    mb_calendar_class_new("LocaleTextCalendar", fw, Some(locale))
}

unsafe extern "C" fn dispatch_locale_html_calendar(
    args_ptr: *const MbValue,
    nargs: usize,
) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let fw = a.get(0).copied().unwrap_or_else(MbValue::none);
    let locale = a.get(1).copied().unwrap_or_else(MbValue::none);
    mb_calendar_class_new("LocaleHTMLCalendar", fw, Some(locale))
}

// -- Registration --

pub fn register() {
    let mut attrs = HashMap::new();

    let dispatchers: Vec<(&str, usize)> = vec![
        ("isleap", dispatch_isleap as usize),
        ("leapdays", dispatch_leapdays as usize),
        ("monthrange", dispatch_monthrange as usize),
        ("weekday", dispatch_weekday as usize),
        ("monthcalendar", dispatch_monthcalendar as usize),
        ("firstweekday", dispatch_firstweekday as usize),
        ("setfirstweekday", dispatch_setfirstweekday as usize),
        ("timegm", dispatch_timegm as usize),
        ("prmonth", dispatch_prmonth as usize),
        ("prcal", dispatch_prcal as usize),
        ("prweek", dispatch_prweek as usize),
        ("month", dispatch_month as usize),
        ("calendar", dispatch_calendar_fn as usize),
        ("week", dispatch_week as usize),
        ("weekheader", dispatch_weekheader as usize),
        ("format", dispatch_format as usize),
        ("formatstring", dispatch_formatstring as usize),
        ("main", dispatch_main as usize),
        ("different_locale", dispatch_different_locale as usize),
        ("global_enum", dispatch_global_enum as usize),
        ("Calendar", dispatch_calendar_cls as usize),
        ("TextCalendar", dispatch_text_calendar as usize),
        ("HTMLCalendar", dispatch_html_calendar as usize),
        ("LocaleTextCalendar", dispatch_locale_text_calendar as usize),
        ("LocaleHTMLCalendar", dispatch_locale_html_calendar as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    // Data attributes — sequences eagerly built at register-time so
    // `callable(calendar.month_name) == False` parity holds.
    attrs.insert("month_name".to_string(), mb_calendar_month_name());
    attrs.insert("month_abbr".to_string(), mb_calendar_month_abbr());
    attrs.insert("day_name".to_string(), mb_calendar_day_name());
    attrs.insert("day_abbr".to_string(), mb_calendar_day_abbr());
    attrs.insert("mdays".to_string(), mb_calendar_mdays());

    // Weekday integer constants (0..6).
    for (i, name) in [
        "MONDAY",
        "TUESDAY",
        "WEDNESDAY",
        "THURSDAY",
        "FRIDAY",
        "SATURDAY",
        "SUNDAY",
    ]
    .iter()
    .enumerate()
    {
        attrs.insert(name.to_string(), MbValue::from_int(i as i64));
    }

    // Month integer constants (1..12).
    for (i, name) in [
        "JANUARY",
        "FEBRUARY",
        "MARCH",
        "APRIL",
        "MAY",
        "JUNE",
        "JULY",
        "AUGUST",
        "SEPTEMBER",
        "OCTOBER",
        "NOVEMBER",
        "DECEMBER",
    ]
    .iter()
    .enumerate()
    {
        attrs.insert(name.to_string(), MbValue::from_int((i + 1) as i64));
    }

    // CPython's EPOCH constant — year 1970 (Unix epoch).
    attrs.insert("EPOCH".to_string(), MbValue::from_int(1970));

    // Exception sentinels — Instance stubs.
    attrs.insert("error".to_string(), make_error_class("error"));
    attrs.insert(
        "IllegalMonthError".to_string(),
        make_error_class("IllegalMonthError"),
    );
    attrs.insert(
        "IllegalWeekdayError".to_string(),
        make_error_class("IllegalWeekdayError"),
    );

    // IntEnum re-exports (`Day`, `Month`, `IntEnum` itself) — passive
    // sentinels with class_name set so identity checks work.
    attrs.insert("Day".to_string(), make_enum_class("Day"));
    attrs.insert("Month".to_string(), make_enum_class("Month"));
    attrs.insert("IntEnum".to_string(), make_enum_class("IntEnum"));

    // Module re-exports — exposed as None placeholders. User code that
    // reaches through `calendar.datetime` / `calendar.sys` / etc. is rare;
    // the canonical paths are the top-level imports.
    for sub in ["c", "datetime", "sys", "warnings", "repeat"] {
        attrs.insert(sub.to_string(), MbValue::none());
    }

    super::register_module("calendar", attrs);
}

// -- Instance-stub helpers --

fn make_error_class(name: &str) -> MbValue {
    let mut fields = FxHashMap::default();
    fields.insert(
        "__name__".to_string(),
        MbValue::from_ptr(MbObject::new_str(name.to_string())),
    );
    fields.insert(
        "__module__".to_string(),
        MbValue::from_ptr(MbObject::new_str("calendar".to_string())),
    );
    let obj = Box::new(MbObject {
        header: MbObjectHeader {
            rc: AtomicU32::new(1),
            kind: ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: name.to_string(),
            fields: RwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

fn make_enum_class(name: &str) -> MbValue {
    let mut fields = FxHashMap::default();
    fields.insert(
        "__name__".to_string(),
        MbValue::from_ptr(MbObject::new_str(name.to_string())),
    );
    fields.insert(
        "__module__".to_string(),
        MbValue::from_ptr(MbObject::new_str("calendar".to_string())),
    );
    let obj = Box::new(MbObject {
        header: MbObjectHeader {
            rc: AtomicU32::new(1),
            kind: ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: name.to_string(),
            fields: RwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

fn mb_calendar_class_new(name: &str, firstweekday: MbValue, locale: Option<MbValue>) -> MbValue {
    let fw = firstweekday
        .as_int()
        .unwrap_or(FIRST_WEEKDAY.load(Ordering::Relaxed));
    let mut fields = FxHashMap::default();
    fields.insert("firstweekday".to_string(), MbValue::from_int(fw));
    if let Some(loc) = locale {
        fields.insert("locale".to_string(), loc);
    }
    let obj = Box::new(MbObject {
        header: MbObjectHeader {
            rc: AtomicU32::new(1),
            kind: ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: name.to_string(),
            fields: RwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

// -- Stub bodies --

fn mb_calendar_empty_str() -> MbValue {
    MbValue::from_ptr(MbObject::new_str(String::new()))
}

fn mb_calendar_none() -> MbValue {
    MbValue::none()
}

// -- Real implementations --

#[inline]
fn is_leap(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

#[inline]
fn days_in_month(y: i64, m: i64) -> i64 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap(y) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

/// Zeller-based weekday (0=Mon..6=Sun) for proleptic Gregorian (y, m, d).
fn zeller_weekday(y: i64, m: i64, d: i64) -> i64 {
    let (ay, am) = if m < 3 { (y - 1, m + 12) } else { (y, m) };
    let k = ay.rem_euclid(100);
    let j = ay.div_euclid(100);
    let h = (d + (13 * (am + 1)) / 5 + k + k / 4 + j / 4 + 5 * j).rem_euclid(7);
    (h + 5).rem_euclid(7)
}

pub fn mb_calendar_isleap(year: MbValue) -> MbValue {
    MbValue::from_bool(is_leap(year.as_int().unwrap_or(0)))
}

pub fn mb_calendar_leapdays(y1: MbValue, y2: MbValue) -> MbValue {
    let a = y1.as_int().unwrap_or(0);
    let b = y2.as_int().unwrap_or(0);
    let cl = |y: i64| y / 4 - y / 100 + y / 400;
    // CPython counts leap years in [y1, y2): subtract one from each
    // bound so the boundary year matches the open-interval convention.
    MbValue::from_int(cl(b - 1) - cl(a - 1))
}

pub fn mb_calendar_monthrange(year: MbValue, month: MbValue) -> MbValue {
    let y = year.as_int().unwrap_or(2000);
    let m = month.as_int().unwrap_or(1);
    let days = days_in_month(y, m);
    let wd = zeller_weekday(y, m, 1);
    MbValue::from_ptr(MbObject::new_tuple(vec![
        MbValue::from_int(wd),
        MbValue::from_int(days),
    ]))
}

pub fn mb_calendar_weekday(year: MbValue, month: MbValue, day: MbValue) -> MbValue {
    let y = year.as_int().unwrap_or(2000);
    let m = month.as_int().unwrap_or(1);
    let d = day.as_int().unwrap_or(1);
    MbValue::from_int(zeller_weekday(y, m, d))
}

/// calendar.monthcalendar(year, month) -> list of weeks; each week is a
/// 7-element list with 0 for days outside the month. Mirrors CPython's
/// Calendar(firstweekday=0).monthdayscalendar(...).
pub fn mb_calendar_monthcalendar(year: MbValue, month: MbValue) -> MbValue {
    let y = year.as_int().unwrap_or(2000);
    let m = month.as_int().unwrap_or(1);
    let first_wd = zeller_weekday(y, m, 1);
    let dim = days_in_month(y, m);
    let fw = FIRST_WEEKDAY.load(Ordering::Relaxed).rem_euclid(7);
    let lead = (first_wd - fw).rem_euclid(7) as usize;

    let mut days: Vec<i64> = Vec::with_capacity(lead + dim as usize);
    for _ in 0..lead {
        days.push(0);
    }
    for d in 1..=dim {
        days.push(d);
    }
    while days.len() % 7 != 0 {
        days.push(0);
    }

    let mut weeks: Vec<MbValue> = Vec::new();
    for chunk in days.chunks(7) {
        let row: Vec<MbValue> = chunk.iter().map(|d| MbValue::from_int(*d)).collect();
        weeks.push(MbValue::from_ptr(MbObject::new_list(row)));
    }
    MbValue::from_ptr(MbObject::new_list(weeks))
}

pub fn mb_calendar_firstweekday() -> MbValue {
    MbValue::from_int(FIRST_WEEKDAY.load(Ordering::Relaxed))
}

pub fn mb_calendar_setfirstweekday(v: MbValue) -> MbValue {
    let n = v.as_int().unwrap_or(0).rem_euclid(7);
    FIRST_WEEKDAY.store(n, Ordering::Relaxed);
    MbValue::none()
}

/// calendar.timegm((y, m, d, h, mi, s, ...)) -> int seconds since epoch (UTC).
///
/// Accepts a tuple/list whose first six entries are
/// (year, month, day, hour, minute, second). Mirrors CPython's
/// `calendar.timegm` for tuples produced by `time.gmtime`.
pub fn mb_calendar_timegm(tup: MbValue) -> MbValue {
    let items: Vec<MbValue> = tup
        .as_ptr()
        .map(|ptr| unsafe {
            match &(*ptr).data {
                ObjData::Tuple(items) => items.clone(),
                ObjData::List(ref lock) => lock.read().unwrap().to_vec(),
                _ => Vec::new(),
            }
        })
        .unwrap_or_default();
    let g = |i: usize, default: i64| items.get(i).and_then(|v| v.as_int()).unwrap_or(default);
    let year = g(0, 1970);
    let month = g(1, 1).max(1).min(12);
    let day = g(2, 1);
    let hour = g(3, 0);
    let minute = g(4, 0);
    let second = g(5, 0);

    // Days from 1970-01-01 to (year, month, day): sum year-day and month-day deltas.
    let mut days: i64 = 0;
    if year >= 1970 {
        for y in 1970..year {
            days += if is_leap(y) { 366 } else { 365 };
        }
    } else {
        for y in year..1970 {
            days -= if is_leap(y) { 366 } else { 365 };
        }
    }
    for m in 1..month {
        days += days_in_month(year, m);
    }
    days += day - 1;

    let secs = days * 86_400 + hour * 3600 + minute * 60 + second;
    MbValue::from_int(secs)
}

// -- Data attribute constructors --

pub fn mb_calendar_month_name() -> MbValue {
    let names = [
        "",
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    let vals: Vec<MbValue> = names
        .iter()
        .map(|n| MbValue::from_ptr(MbObject::new_str(n.to_string())))
        .collect();
    MbValue::from_ptr(MbObject::new_list(vals))
}

pub fn mb_calendar_month_abbr() -> MbValue {
    let names = [
        "", "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let vals: Vec<MbValue> = names
        .iter()
        .map(|n| MbValue::from_ptr(MbObject::new_str(n.to_string())))
        .collect();
    MbValue::from_ptr(MbObject::new_list(vals))
}

pub fn mb_calendar_day_name() -> MbValue {
    let names = [
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
        "Sunday",
    ];
    let vals: Vec<MbValue> = names
        .iter()
        .map(|n| MbValue::from_ptr(MbObject::new_str(n.to_string())))
        .collect();
    MbValue::from_ptr(MbObject::new_list(vals))
}

pub fn mb_calendar_day_abbr() -> MbValue {
    let names = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    let vals: Vec<MbValue> = names
        .iter()
        .map(|n| MbValue::from_ptr(MbObject::new_str(n.to_string())))
        .collect();
    MbValue::from_ptr(MbObject::new_list(vals))
}

/// calendar.mdays — list of days-in-month indexed by month
/// (entry 0 is a placeholder so `mdays[1]` is January).
pub fn mb_calendar_mdays() -> MbValue {
    let dim = [0, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let vals: Vec<MbValue> = dim.iter().map(|n| MbValue::from_int(*n)).collect();
    MbValue::from_ptr(MbObject::new_list(vals))
}

// HANDWRITE-END

#[cfg(test)]
mod tests {
    use super::super::super::rc::ObjData;
    use super::*;

    fn tuple_int_at(val: MbValue, idx: usize) -> Option<i64> {
        val.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Tuple(ref items) = (*ptr).data {
                items.get(idx).and_then(|v| v.as_int())
            } else {
                None
            }
        })
    }

    fn list_len(val: MbValue) -> usize {
        val.as_ptr()
            .map(|ptr| unsafe {
                if let ObjData::List(ref lock) = (*ptr).data {
                    lock.read().unwrap().len()
                } else {
                    0
                }
            })
            .unwrap_or(0)
    }

    fn list_str_at(val: MbValue, idx: usize) -> Option<String> {
        val.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                lock.read().unwrap().get(idx).copied().and_then(|v| {
                    v.as_ptr().and_then(|p| {
                        if let ObjData::Str(ref s) = (*p).data {
                            Some(s.clone())
                        } else {
                            None
                        }
                    })
                })
            } else {
                None
            }
        })
    }

    fn list_int_at(val: MbValue, idx: usize) -> Option<i64> {
        val.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                lock.read()
                    .unwrap()
                    .get(idx)
                    .copied()
                    .and_then(|v| v.as_int())
            } else {
                None
            }
        })
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

    // -- isleap truth table --

    #[test]
    fn test_isleap_400() {
        assert_eq!(
            mb_calendar_isleap(MbValue::from_int(2000)).as_bool(),
            Some(true)
        );
    }

    #[test]
    fn test_isleap_100() {
        assert_eq!(
            mb_calendar_isleap(MbValue::from_int(1900)).as_bool(),
            Some(false)
        );
    }

    #[test]
    fn test_isleap_4() {
        assert_eq!(
            mb_calendar_isleap(MbValue::from_int(2024)).as_bool(),
            Some(true)
        );
    }

    #[test]
    fn test_isleap_odd() {
        assert_eq!(
            mb_calendar_isleap(MbValue::from_int(2023)).as_bool(),
            Some(false)
        );
    }

    #[test]
    fn test_isleap_2100() {
        // Divisible by 100, not 400 — not a leap year.
        assert_eq!(
            mb_calendar_isleap(MbValue::from_int(2100)).as_bool(),
            Some(false)
        );
    }

    // -- leapdays --

    #[test]
    fn test_leapdays_century() {
        // CPython: calendar.leapdays(1900, 2000) == 24
        let r = mb_calendar_leapdays(MbValue::from_int(1900), MbValue::from_int(2000));
        assert_eq!(r.as_int(), Some(24));
    }

    #[test]
    fn test_leapdays_zero() {
        let r = mb_calendar_leapdays(MbValue::from_int(2000), MbValue::from_int(2000));
        assert_eq!(r.as_int(), Some(0));
    }

    #[test]
    fn test_leapdays_small_range() {
        // [2020, 2025): 2020, 2024 → 2 leap years
        let r = mb_calendar_leapdays(MbValue::from_int(2020), MbValue::from_int(2025));
        assert_eq!(r.as_int(), Some(2));
    }

    // -- monthrange --

    #[test]
    fn test_monthrange_jan() {
        let r = mb_calendar_monthrange(MbValue::from_int(2024), MbValue::from_int(1));
        assert_eq!(tuple_int_at(r, 1), Some(31));
        // 2024-01-01 is Monday → weekday 0
        assert_eq!(tuple_int_at(r, 0), Some(0));
    }

    #[test]
    fn test_monthrange_apr() {
        let r = mb_calendar_monthrange(MbValue::from_int(2024), MbValue::from_int(4));
        assert_eq!(tuple_int_at(r, 1), Some(30));
    }

    #[test]
    fn test_monthrange_feb_leap() {
        let r = mb_calendar_monthrange(MbValue::from_int(2024), MbValue::from_int(2));
        assert_eq!(tuple_int_at(r, 1), Some(29));
    }

    #[test]
    fn test_monthrange_feb_normal() {
        let r = mb_calendar_monthrange(MbValue::from_int(2023), MbValue::from_int(2));
        assert_eq!(tuple_int_at(r, 1), Some(28));
    }

    // -- weekday known dates --

    #[test]
    fn test_weekday_2024_01_01_monday() {
        // 2024-01-01 is a Monday → 0
        let r = mb_calendar_weekday(
            MbValue::from_int(2024),
            MbValue::from_int(1),
            MbValue::from_int(1),
        );
        assert_eq!(r.as_int(), Some(0));
    }

    #[test]
    fn test_weekday_2000_01_01_saturday() {
        // 2000-01-01 is a Saturday → 5
        let r = mb_calendar_weekday(
            MbValue::from_int(2000),
            MbValue::from_int(1),
            MbValue::from_int(1),
        );
        assert_eq!(r.as_int(), Some(5));
    }

    #[test]
    fn test_weekday_1970_01_01_thursday() {
        // 1970-01-01 (Unix epoch) is a Thursday → 3
        let r = mb_calendar_weekday(
            MbValue::from_int(1970),
            MbValue::from_int(1),
            MbValue::from_int(1),
        );
        assert_eq!(r.as_int(), Some(3));
    }

    #[test]
    fn test_weekday_2026_05_16_saturday() {
        // 2026-05-16 is a Saturday → 5
        let r = mb_calendar_weekday(
            MbValue::from_int(2026),
            MbValue::from_int(5),
            MbValue::from_int(16),
        );
        assert_eq!(r.as_int(), Some(5));
    }

    // -- monthcalendar --

    #[test]
    fn test_monthcalendar_jan_2024_shape() {
        // 2024-01: starts Monday (firstweekday=0 default), 31 days.
        // → 5 weeks, last week padded with trailing zeros.
        FIRST_WEEKDAY.store(0, Ordering::Relaxed);
        let cal = mb_calendar_monthcalendar(MbValue::from_int(2024), MbValue::from_int(1));
        assert_eq!(list_len(cal), 5);
    }

    #[test]
    fn test_monthcalendar_feb_2024_starts_with_zeros() {
        // 2024-02-01 is Thursday; with firstweekday=0 the first week has
        // three leading zeros [0, 0, 0, 1, 2, 3, 4].
        FIRST_WEEKDAY.store(0, Ordering::Relaxed);
        let cal = mb_calendar_monthcalendar(MbValue::from_int(2024), MbValue::from_int(2));
        let week0 = cal
            .as_ptr()
            .map(|ptr| unsafe {
                if let ObjData::List(ref lock) = (*ptr).data {
                    lock.read().unwrap()[0]
                } else {
                    MbValue::none()
                }
            })
            .unwrap_or_else(MbValue::none);
        assert_eq!(list_int_at(week0, 0), Some(0));
        assert_eq!(list_int_at(week0, 1), Some(0));
        assert_eq!(list_int_at(week0, 2), Some(0));
        assert_eq!(list_int_at(week0, 3), Some(1));
        assert_eq!(list_int_at(week0, 6), Some(4));
    }

    // -- data attributes --

    #[test]
    fn test_month_name_count() {
        let r = mb_calendar_month_name();
        assert_eq!(list_len(r), 13);
        assert_eq!(list_str_at(r, 0).as_deref(), Some(""));
        assert_eq!(list_str_at(r, 1).as_deref(), Some("January"));
        assert_eq!(list_str_at(r, 12).as_deref(), Some("December"));
    }

    #[test]
    fn test_month_abbr_count() {
        let r = mb_calendar_month_abbr();
        assert_eq!(list_len(r), 13);
        assert_eq!(list_str_at(r, 1).as_deref(), Some("Jan"));
        assert_eq!(list_str_at(r, 12).as_deref(), Some("Dec"));
    }

    #[test]
    fn test_day_name_count() {
        let r = mb_calendar_day_name();
        assert_eq!(list_len(r), 7);
        assert_eq!(list_str_at(r, 0).as_deref(), Some("Monday"));
        assert_eq!(list_str_at(r, 6).as_deref(), Some("Sunday"));
    }

    #[test]
    fn test_day_abbr_count() {
        let r = mb_calendar_day_abbr();
        assert_eq!(list_len(r), 7);
        assert_eq!(list_str_at(r, 0).as_deref(), Some("Mon"));
        assert_eq!(list_str_at(r, 6).as_deref(), Some("Sun"));
    }

    #[test]
    fn test_mdays_layout() {
        let r = mb_calendar_mdays();
        assert_eq!(list_len(r), 13);
        assert_eq!(list_int_at(r, 0), Some(0));
        assert_eq!(list_int_at(r, 1), Some(31));
        assert_eq!(list_int_at(r, 2), Some(28));
        assert_eq!(list_int_at(r, 4), Some(30));
        assert_eq!(list_int_at(r, 12), Some(31));
    }

    // -- firstweekday round-trip --

    #[test]
    fn test_setfirstweekday_roundtrip() {
        let _ = mb_calendar_setfirstweekday(MbValue::from_int(3));
        assert_eq!(mb_calendar_firstweekday().as_int(), Some(3));
        let _ = mb_calendar_setfirstweekday(MbValue::from_int(0));
        assert_eq!(mb_calendar_firstweekday().as_int(), Some(0));
    }

    // -- timegm --

    #[test]
    fn test_timegm_epoch() {
        // (1970, 1, 1, 0, 0, 0) → 0
        let tup = MbValue::from_ptr(MbObject::new_tuple(vec![
            MbValue::from_int(1970),
            MbValue::from_int(1),
            MbValue::from_int(1),
            MbValue::from_int(0),
            MbValue::from_int(0),
            MbValue::from_int(0),
        ]));
        assert_eq!(mb_calendar_timegm(tup).as_int(), Some(0));
    }

    #[test]
    fn test_timegm_known_date() {
        // (2024, 1, 1, 0, 0, 0) — CPython: calendar.timegm((2024,1,1,0,0,0))
        // == 1704067200
        let tup = MbValue::from_ptr(MbObject::new_tuple(vec![
            MbValue::from_int(2024),
            MbValue::from_int(1),
            MbValue::from_int(1),
            MbValue::from_int(0),
            MbValue::from_int(0),
            MbValue::from_int(0),
        ]));
        assert_eq!(mb_calendar_timegm(tup).as_int(), Some(1_704_067_200));
    }

    // -- class stubs --

    #[test]
    fn test_text_calendar_stub() {
        let inst = mb_calendar_class_new("TextCalendar", MbValue::from_int(2), None);
        assert_eq!(get_field(inst, "firstweekday").as_int(), Some(2));
    }

    #[test]
    fn test_locale_text_calendar_carries_locale() {
        let loc = MbValue::from_ptr(MbObject::new_str("en_US.UTF-8".to_string()));
        let inst = mb_calendar_class_new("LocaleTextCalendar", MbValue::from_int(0), Some(loc));
        assert_eq!(
            get_str(get_field(inst, "locale")).as_deref(),
            Some("en_US.UTF-8")
        );
    }

    #[test]
    fn test_make_error_class_carries_name() {
        let e = make_error_class("IllegalMonthError");
        assert_eq!(
            get_str(get_field(e, "__name__")).as_deref(),
            Some("IllegalMonthError")
        );
        assert_eq!(
            get_str(get_field(e, "__module__")).as_deref(),
            Some("calendar")
        );
    }

    #[test]
    fn test_make_enum_class_carries_name() {
        let m = make_enum_class("Month");
        assert_eq!(get_str(get_field(m, "__name__")).as_deref(), Some("Month"));
    }
}
