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

use std::collections::HashMap;
use rustc_hash::FxHashMap;
use crate::runtime::rc::MbRwLock as RwLock;
use std::sync::atomic::{AtomicI64, AtomicU32, Ordering};
use super::super::value::MbValue;
use super::super::rc::{MbObject, MbObjectHeader, ObjData, ObjKind};

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

// Module-level convenience functions. CPython binds these to a module-global
// `TextCalendar()` instance, so they read the module FIRST_WEEKDAY. The `pr*`
// variants print (via calendar_emit, honoring redirect/capture); the rest
// return text. Defined below near the formatting helpers.
dispatch_stub!(dispatch_main, mb_calendar_none);
dispatch_stub!(dispatch_different_locale, mb_calendar_none);
dispatch_stub!(dispatch_global_enum, mb_calendar_none);

// Class constructors — return Instance stubs carrying class_name + firstweekday.
//
// `firstweekday` may arrive positionally (`Calendar(3)`) or as a keyword
// (`Calendar(firstweekday=3)`). Keyword args are lowered as a trailing dict, so
// read positional[0] first, then fall back to the kwargs dict.
unsafe extern "C" fn dispatch_calendar_cls(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let all = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_calendar_class_new("Calendar", ctor_firstweekday(all), None)
}

unsafe extern "C" fn dispatch_text_calendar(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let all = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_calendar_class_new("TextCalendar", ctor_firstweekday(all), None)
}

unsafe extern "C" fn dispatch_html_calendar(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let all = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_calendar_class_new("HTMLCalendar", ctor_firstweekday(all), None)
}

unsafe extern "C" fn dispatch_locale_text_calendar(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let all = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let (pos, _kw) = split_args(all);
    let locale = pos.get(1).copied().unwrap_or_else(MbValue::none);
    mb_calendar_class_new("LocaleTextCalendar", ctor_firstweekday(all), Some(locale))
}

unsafe extern "C" fn dispatch_locale_html_calendar(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let all = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let (pos, _kw) = split_args(all);
    let locale = pos.get(1).copied().unwrap_or_else(MbValue::none);
    mb_calendar_class_new("LocaleHTMLCalendar", ctor_firstweekday(all), Some(locale))
}

/// Resolve the `firstweekday` constructor argument from positional[0] or the
/// trailing kwargs dict; defaults to None (→ module FIRST_WEEKDAY → 0).
fn ctor_firstweekday(all: &[MbValue]) -> MbValue {
    let (pos, kw) = split_args(all);
    if let Some(v) = pos.first().copied() {
        if v.as_int().is_some() {
            return v;
        }
    }
    if let Some(kw) = kw {
        if let Some(ptr) = kw.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    if let Some(v) = lock.read().unwrap()
                        .get(&super::super::dict_ops::DictKey::Str("firstweekday".to_string()))
                        .copied()
                    {
                        return v;
                    }
                }
            }
        }
    }
    MbValue::none()
}

// -- Registration --

pub fn register() {
    let mut attrs = HashMap::new();

    let dispatchers: Vec<(&str, usize)> = vec![
        ("isleap",            dispatch_isleap            as usize),
        ("leapdays",          dispatch_leapdays          as usize),
        ("monthrange",        dispatch_monthrange        as usize),
        ("weekday",           dispatch_weekday           as usize),
        ("monthcalendar",     dispatch_monthcalendar     as usize),
        ("firstweekday",      dispatch_firstweekday      as usize),
        ("setfirstweekday",   dispatch_setfirstweekday   as usize),
        ("timegm",            dispatch_timegm            as usize),
        ("prmonth",           dispatch_prmonth           as usize),
        ("prcal",             dispatch_prcal             as usize),
        ("prweek",            dispatch_prweek            as usize),
        ("month",             dispatch_month             as usize),
        ("calendar",          dispatch_calendar_fn       as usize),
        ("week",              dispatch_week              as usize),
        ("weekheader",        dispatch_weekheader        as usize),
        ("format",            dispatch_format            as usize),
        ("formatstring",      dispatch_formatstring      as usize),
        ("main",              dispatch_main              as usize),
        ("different_locale",  dispatch_different_locale  as usize),
        ("global_enum",       dispatch_global_enum       as usize),
        ("Calendar",          dispatch_calendar_cls      as usize),
        ("TextCalendar",      dispatch_text_calendar     as usize),
        ("HTMLCalendar",      dispatch_html_calendar     as usize),
        ("LocaleTextCalendar", dispatch_locale_text_calendar as usize),
        ("LocaleHTMLCalendar", dispatch_locale_html_calendar as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    // HTMLCalendar doubles as a themable base class: map the constructor
    // dispatcher to its class name (NATIVE_TYPE_NAMES) and register the css
    // theming defaults as class attributes, so both
    // `calendar.HTMLCalendar.cssclasses` and user-subclass overrides resolve.
    for (cls, addr) in [
        ("HTMLCalendar", dispatch_html_calendar as usize),
        ("LocaleHTMLCalendar", dispatch_locale_html_calendar as usize),
    ] {
        super::super::module::NATIVE_TYPE_NAMES.with(|m| {
            m.borrow_mut().insert(addr as u64, cls.to_string());
        });
        // mb_class_set_class_attr only writes to registered classes.
        super::super::class::mb_class_register(cls, vec![], std::collections::HashMap::new());
        let str_list = |xs: &[&str]| {
            MbValue::from_ptr(MbObject::new_list(
                xs.iter()
                    .map(|s| MbValue::from_ptr(MbObject::new_str(s.to_string())))
                    .collect(),
            ))
        };
        let set_attr = |name: &str, v: MbValue| {
            super::super::class::mb_class_set_class_attr(
                MbValue::from_ptr(MbObject::new_str(cls.to_string())),
                MbValue::from_ptr(MbObject::new_str(name.to_string())),
                v,
            );
        };
        set_attr("cssclasses", str_list(&HTML_CSS));
        set_attr("cssclasses_weekday_head", str_list(&HTML_CSS_WEEKDAY_HEAD));
        set_attr("cssclass_noday", MbValue::from_ptr(MbObject::new_str("noday".to_string())));
        set_attr("cssclass_month", MbValue::from_ptr(MbObject::new_str(HTML_CSS_MONTH.to_string())));
        set_attr("cssclass_month_head", MbValue::from_ptr(MbObject::new_str(HTML_CSS_MONTH_HEAD.to_string())));
        set_attr("cssclass_year", MbValue::from_ptr(MbObject::new_str(HTML_CSS_YEAR.to_string())));
        set_attr("cssclass_year_head", MbValue::from_ptr(MbObject::new_str(HTML_CSS_YEAR_HEAD.to_string())));
    }

    // Data attributes — sequences eagerly built at register-time so
    // `callable(calendar.month_name) == False` parity holds.
    attrs.insert("month_name".to_string(), mb_calendar_month_name());
    attrs.insert("month_abbr".to_string(), mb_calendar_month_abbr());
    attrs.insert("day_name".to_string(),   mb_calendar_day_name());
    attrs.insert("day_abbr".to_string(),   mb_calendar_day_abbr());
    attrs.insert("mdays".to_string(),      mb_calendar_mdays());

    // Weekday integer constants (0..6).
    for (i, name) in ["MONDAY","TUESDAY","WEDNESDAY","THURSDAY","FRIDAY","SATURDAY","SUNDAY"]
        .iter().enumerate()
    {
        attrs.insert(name.to_string(), MbValue::from_int(i as i64));
    }

    // Month integer constants (1..12).
    for (i, name) in ["JANUARY","FEBRUARY","MARCH","APRIL","MAY","JUNE",
                      "JULY","AUGUST","SEPTEMBER","OCTOBER","NOVEMBER","DECEMBER"]
        .iter().enumerate()
    {
        attrs.insert(name.to_string(), MbValue::from_int((i + 1) as i64));
    }

    // CPython's EPOCH constant — year 1970 (Unix epoch).
    attrs.insert("EPOCH".to_string(), MbValue::from_int(1970));

    // Exception classes. Mamba matches `except calendar.Err` by the type-name
    // string carried on both the registered marker and the raised instance
    // (see exception::mb_exception_matches). Exposing each as a `Str` of its
    // own name makes specific catches discriminate, and the names are listed
    // in exception.rs under ValueError so `except ValueError` / `except
    // Exception` also catch them (CPython: both subclass ValueError).
    attrs.insert("error".to_string(),
        make_error_class("error"));
    attrs.insert("IllegalMonthError".to_string(),
        MbValue::from_ptr(MbObject::new_str("IllegalMonthError".to_string())));
    attrs.insert("IllegalWeekdayError".to_string(),
        MbValue::from_ptr(MbObject::new_str("IllegalWeekdayError".to_string())));

    // IntEnum re-exports (`Day`, `Month`, `IntEnum` itself) — passive
    // sentinels with class_name set so identity checks work.
    attrs.insert("Day".to_string(),     make_enum_class("Day"));
    attrs.insert("Month".to_string(),   make_enum_class("Month"));
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
    fields.insert("__name__".to_string(),
        MbValue::from_ptr(MbObject::new_str(name.to_string())));
    fields.insert("__module__".to_string(),
        MbValue::from_ptr(MbObject::new_str("calendar".to_string())));
    let obj = Box::new(MbObject {
        header: MbObjectHeader { rc: AtomicU32::new(1), kind: ObjKind::Instance },
        data: ObjData::Instance {
            class_name: name.to_string(),
            fields: RwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

fn make_enum_class(name: &str) -> MbValue {
    let mut fields = FxHashMap::default();
    fields.insert("__name__".to_string(),
        MbValue::from_ptr(MbObject::new_str(name.to_string())));
    fields.insert("__module__".to_string(),
        MbValue::from_ptr(MbObject::new_str("calendar".to_string())));
    let obj = Box::new(MbObject {
        header: MbObjectHeader { rc: AtomicU32::new(1), kind: ObjKind::Instance },
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
        .unwrap_or_else(|| FIRST_WEEKDAY.load(Ordering::Relaxed))
        .rem_euclid(7);
    let mut fields = FxHashMap::default();
    fields.insert("firstweekday".to_string(), MbValue::from_int(fw));
    if let Some(loc) = locale {
        fields.insert("locale".to_string(), loc);
    }
    attach_calendar_methods(&mut fields, name, fw as usize);
    let obj = Box::new(MbObject {
        header: MbObjectHeader { rc: AtomicU32::new(1), kind: ObjKind::Instance },
        data: ObjData::Instance {
            class_name: name.to_string(),
            fields: RwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

/// Register a native function pointer as an instance method field. The
/// generic instance fall-through in `mb_call_method` (core class.rs) calls a
/// callable instance field with NO implicit self, so each method is a plain
/// flat-args dispatcher that receives only the user-supplied arguments. Where
/// a method needs `self.firstweekday`, we attach the monomorphized variant for
/// the instance's firstweekday at construction time.
fn put_method(fields: &mut FxHashMap<String, MbValue>, name: &str, addr: usize) {
    fields.insert(name.to_string(), MbValue::from_func(addr));
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(addr as u64);
    });
}

/// Wire the calendar instance method table. `fw` is the resolved firstweekday
/// (0..6) used to pick the monomorphized iteration/format variant.
fn attach_calendar_methods(fields: &mut FxHashMap<String, MbValue>, name: &str, fw: usize) {
    let fw = fw % 7;
    // Calendar base methods (every Calendar subclass inherits these).
    put_method(fields, "iterweekdays", ITERWEEKDAYS[fw] as usize);
    put_method(fields, "itermonthdates", ITERMONTHDATES[fw] as usize);
    put_method(fields, "itermonthdays", ITERMONTHDAYS[fw] as usize);
    put_method(fields, "itermonthdays2", ITERMONTHDAYS2[fw] as usize);
    put_method(fields, "itermonthdays3", ITERMONTHDAYS3[fw] as usize);
    put_method(fields, "itermonthdays4", ITERMONTHDAYS4[fw] as usize);
    put_method(fields, "monthdatescalendar", MONTHDATESCALENDAR[fw] as usize);
    put_method(fields, "monthdays2calendar", MONTHDAYS2CALENDAR[fw] as usize);
    put_method(fields, "monthdayscalendar", MONTHDAYSCALENDAR[fw] as usize);
    put_method(fields, "yeardatescalendar", YEARDATESCALENDAR[fw] as usize);
    put_method(fields, "yeardays2calendar", YEARDAYS2CALENDAR[fw] as usize);
    put_method(fields, "yeardayscalendar", YEARDAYSCALENDAR[fw] as usize);

    match name {
        "TextCalendar" | "LocaleTextCalendar" => {
            put_method(fields, "formatday", text_formatday as usize);
            put_method(fields, "formatweek", text_formatweek as usize);
            put_method(fields, "formatweekday", text_formatweekday as usize);
            put_method(fields, "formatweekheader", text_formatweekheader as usize);
            put_method(fields, "formatmonthname", TEXT_FORMATMONTHNAME[fw] as usize);
            put_method(fields, "formatmonth", TEXT_FORMATMONTH[fw] as usize);
            put_method(fields, "formatyear", TEXT_FORMATYEAR[fw] as usize);
            put_method(fields, "prmonth", TEXT_PRMONTH[fw] as usize);
            put_method(fields, "prweek", text_prweek as usize);
            put_method(fields, "pryear", TEXT_PRYEAR[fw] as usize);
        }
        "HTMLCalendar" | "LocaleHTMLCalendar" => {
            put_method(fields, "formatday", html_formatday as usize);
            put_method(fields, "formatweek", html_formatweek as usize);
            put_method(fields, "formatweekday", html_formatweekday as usize);
            put_method(fields, "formatweekheader", html_formatweekheader as usize);
            put_method(fields, "formatmonthname", html_formatmonthname as usize);
            put_method(fields, "formatmonth", HTML_FORMATMONTH[fw] as usize);
            put_method(fields, "formatyear", HTML_FORMATYEAR[fw] as usize);
            put_method(fields, "formatyearpage", HTML_FORMATYEARPAGE[fw] as usize);
            // HTMLCalendar class-level theming attributes (read by the format
            // methods and overridable by user subclasses).
            html_css_defaults(fields);
        }
        _ => {}
    }
}

// -- Stub bodies --

/// Emit `s` to stdout, honoring the active contextlib.redirect_stdout target
/// stack and any conformance capture buffer (mirrors the `mb_out!` macro in
/// builtins.rs). Calendar's `pr*` methods print via this so that
/// `redirect_stdout`/`captured_stdout` see their output.
fn calendar_emit(s: &str) {
    if !super::super::output::write_captured(s) {
        print!("{s}");
    }
}

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
        1|3|5|7|8|10|12 => 31,
        4|6|9|11 => 30,
        2 => if is_leap(y) { 29 } else { 28 },
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

/// Raise `type_name(msg)` into the runtime exception slot and return None.
fn calendar_raise(type_name: &str, msg: String) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str(type_name.to_string())),
        MbValue::from_ptr(MbObject::new_str(msg)),
    );
    MbValue::none()
}

pub fn mb_calendar_monthrange(year: MbValue, month: MbValue) -> MbValue {
    let y = year.as_int().unwrap_or(2000);
    let m = month.as_int().unwrap_or(1);
    // CPython: monthrange raises IllegalMonthError for month ∉ 1..=12.
    if !(1..=12).contains(&m) {
        return calendar_raise("IllegalMonthError", format!("bad month number {m}; must be 1-12"));
    }
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
    for _ in 0..lead { days.push(0); }
    for d in 1..=dim { days.push(d); }
    while days.len() % 7 != 0 { days.push(0); }

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
    // CPython's guard is `if not MONDAY <= firstweekday <= SUNDAY`, so a
    // non-int argument fails inside the comparison with TypeError.
    let n = match v.as_int_pyint() {
        Some(n) => n,
        None => {
            let tn = if v.is_float() {
                "float"
            } else if v.is_none() {
                "NoneType"
            } else if let Some(ptr) = v.as_ptr() {
                unsafe {
                    match (*ptr).data {
                        ObjData::Str(_) => "str",
                        ObjData::Bytes(_) => "bytes",
                        ObjData::List(_) => "list",
                        _ => "object",
                    }
                }
            } else {
                "object"
            };
            return calendar_raise(
                "TypeError",
                format!("'<=' not supported between instances of 'int' and '{tn}'"),
            );
        }
    };
    // CPython: setfirstweekday raises IllegalWeekdayError for weekday ∉ 0..=6.
    if !(0..=6).contains(&n) {
        return calendar_raise(
            "IllegalWeekdayError",
            format!("bad weekday number {n}; must be 0 (Monday) to 6 (Sunday)"),
        );
    }
    FIRST_WEEKDAY.store(n, Ordering::Relaxed);
    MbValue::none()
}

/// calendar.timegm((y, m, d, h, mi, s, ...)) -> int seconds since epoch (UTC).
///
/// Accepts a tuple/list whose first six entries are
/// (year, month, day, hour, minute, second), or a `time.struct_time` Instance
/// (read via its `tm_year` / `tm_mon` / ... named fields). Mirrors CPython's
/// `calendar.timegm`.
pub fn mb_calendar_timegm(tup: MbValue) -> MbValue {
    // struct_time Instance path: read named fields directly.
    let st_fields: Option<[i64; 6]> = tup.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { ref class_name, ref fields } = (*ptr).data {
            if class_name == "struct_time" {
                let f = fields.read().unwrap();
                let rd = |k: &str, d: i64| f.get(k).and_then(|v| v.as_int()).unwrap_or(d);
                return Some([
                    rd("tm_year", 1970),
                    rd("tm_mon", 1),
                    rd("tm_mday", 1),
                    rd("tm_hour", 0),
                    rd("tm_min", 0),
                    rd("tm_sec", 0),
                ]);
            }
        }
        None
    });
    let items: Vec<MbValue> = tup.as_ptr().map(|ptr| unsafe {
        match &(*ptr).data {
            ObjData::Tuple(items) => items.clone(),
            ObjData::List(ref lock) => lock.read().unwrap().to_vec(),
            _ => Vec::new(),
        }
    }).unwrap_or_default();
    let g = |i: usize, default: i64| items.get(i).and_then(|v| v.as_int()).unwrap_or(default);
    let (year, month, day, hour, minute, second) = if let Some(s) = st_fields {
        (s[0], s[1].max(1).min(12), s[2], s[3], s[4], s[5])
    } else {
        (g(0, 1970), g(1, 1).max(1).min(12), g(2, 1), g(3, 0), g(4, 0), g(5, 0))
    };

    // Days from 1970-01-01 to (year, month, day): sum year-day and month-day deltas.
    let mut days: i64 = 0;
    if year >= 1970 {
        for y in 1970..year { days += if is_leap(y) { 366 } else { 365 }; }
    } else {
        for y in year..1970 { days -= if is_leap(y) { 366 } else { 365 }; }
    }
    for m in 1..month { days += days_in_month(year, m); }
    days += day - 1;

    let secs = days * 86_400 + hour * 3600 + minute * 60 + second;
    MbValue::from_int(secs)
}

// -- Data attribute constructors --

pub fn mb_calendar_month_name() -> MbValue {
    let names = ["","January","February","March","April","May","June",
                 "July","August","September","October","November","December"];
    let vals: Vec<MbValue> = names.iter()
        .map(|n| MbValue::from_ptr(MbObject::new_str(n.to_string()))).collect();
    MbValue::from_ptr(MbObject::new_list(vals))
}

pub fn mb_calendar_month_abbr() -> MbValue {
    let names = ["","Jan","Feb","Mar","Apr","May","Jun",
                 "Jul","Aug","Sep","Oct","Nov","Dec"];
    let vals: Vec<MbValue> = names.iter()
        .map(|n| MbValue::from_ptr(MbObject::new_str(n.to_string()))).collect();
    MbValue::from_ptr(MbObject::new_list(vals))
}

pub fn mb_calendar_day_name() -> MbValue {
    let names = ["Monday","Tuesday","Wednesday","Thursday","Friday","Saturday","Sunday"];
    let vals: Vec<MbValue> = names.iter()
        .map(|n| MbValue::from_ptr(MbObject::new_str(n.to_string()))).collect();
    MbValue::from_ptr(MbObject::new_list(vals))
}

pub fn mb_calendar_day_abbr() -> MbValue {
    let names = ["Mon","Tue","Wed","Thu","Fri","Sat","Sun"];
    let vals: Vec<MbValue> = names.iter()
        .map(|n| MbValue::from_ptr(MbObject::new_str(n.to_string()))).collect();
    MbValue::from_ptr(MbObject::new_list(vals))
}

/// calendar.mdays — list of days-in-month indexed by month
/// (entry 0 is a placeholder so `mdays[1]` is January).
pub fn mb_calendar_mdays() -> MbValue {
    let dim = [0, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let vals: Vec<MbValue> = dim.iter().map(|n| MbValue::from_int(*n)).collect();
    MbValue::from_ptr(MbObject::new_list(vals))
}

// -- Calendar instance-method engine (CPython 3.12 `calendar.py` faithful) --
//
// Method functions are flat-args dispatchers invoked via the generic instance
// field-callable fall-through in core `mb_call_method` (NO implicit self).
// firstweekday-dependent methods are monomorphized into 7 variants (fw=0..6)
// and the right variant is bound to the instance at construction time.

/// Flat-args native dispatcher type for calendar instance methods. Function
/// pointers can live in `const` arrays as this type; cast to `usize` only at
/// the runtime registration site (`as usize`), never in const context.
type CalFn = unsafe extern "C" fn(*const MbValue, usize) -> MbValue;

const MONTH_FULL: [&str; 13] = [
    "", "January", "February", "March", "April", "May", "June",
    "July", "August", "September", "October", "November", "December",
];
const DAY_ABBR3: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
const DAY_FULL: [&str; 7] = [
    "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday",
];

/// Days since the proleptic-Gregorian epoch (0001-01-01 == ordinal 1, like
/// `datetime.date.toordinal`: proleptic Gregorian ordinal where
/// 0001-01-01 == 1. Used to walk consecutive dates across month and year
/// boundaries for itermonthdays3/4 and monthdatescalendar.
fn date_to_ordinal(y: i64, m: i64, d: i64) -> i64 {
    // Days before year `y`, plus days before month `m` in `y`, plus `d`.
    // Use floor division (div_euclid) so the leap-year-count terms match
    // CPython for non-positive years (e.g. year 0, where yprev == -1).
    let yprev = y - 1;
    let days_before_year =
        yprev * 365 + yprev.div_euclid(4) - yprev.div_euclid(100) + yprev.div_euclid(400);
    let mut days_before_month = 0i64;
    for mm in 1..m {
        days_before_month += days_in_month(y, mm);
    }
    days_before_year + days_before_month + d
}

fn ordinal_to_date(ord: i64) -> (i64, i64, i64) {
    // Invert date_to_ordinal. Estimate the year, then correct.
    // Days in 400/100/4-year cycles for the proleptic Gregorian calendar.
    const DI400Y: i64 = 146_097;
    const DI100Y: i64 = 36_524;
    const DI4Y: i64 = 1_461;
    let n = ord - 1; // 0-based day count from 0001-01-01.
    let n400 = n / DI400Y;
    let mut rem = n % DI400Y;
    let n100 = rem / DI100Y;
    rem %= DI100Y;
    let n4 = rem / DI4Y;
    rem %= DI4Y;
    let n1 = rem / 365;
    rem %= 365;
    let year = n400 * 400 + n100 * 100 + n4 * 4 + n1 + 1;
    if n1 == 4 || n100 == 4 {
        // Last day of a leap year (Dec 31 of year-1).
        return (year - 1, 12, 31);
    }
    // `rem` is the 0-based day within `year`; find month/day.
    let mut m = 1i64;
    loop {
        let dim = days_in_month(year, m);
        if rem < dim {
            break;
        }
        rem -= dim;
        m += 1;
    }
    (year, m, rem + 1)
}

/// weekday (0=Mon..6=Sun) of a proleptic-Gregorian date. ordinal 1 ==
/// 0001-01-01 == Monday(0).
fn weekday_of(y: i64, m: i64, d: i64) -> i64 {
    (date_to_ordinal(y, m, d) - 1).rem_euclid(7)
}

/// CPython monthrange: (weekday_of_first, days_in_month).
fn monthrange(y: i64, m: i64) -> (i64, i64) {
    (weekday_of(y, m, 1), days_in_month(y, m))
}

/// CPython's date-yielding methods (itermonthdates / monthdatescalendar /
/// yeardatescalendar) emit `datetime.date` objects. Mamba's `datetime.date`
/// constructor does not yet produce a value that satisfies `isinstance(x,
/// datetime.date)` (it returns a datetime, identity-distinct from the date
/// type), so those fixtures are blocked on the core datetime model. We yield a
/// (year, month, day) tuple here, which keeps the day-number / structure
/// fixtures (yeardayscalendar, itermonthdays3, etc.) correct.
fn make_date(y: i64, m: i64, d: i64) -> MbValue {
    // Real datetime.date instances (CPython: itermonthdates and the
    // *datescalendar grids yield date objects, not (y, m, d) tuples).
    let args = MbValue::from_ptr(MbObject::new_list(vec![
        MbValue::from_int(y),
        MbValue::from_int(m),
        MbValue::from_int(d),
    ]));
    super::datetime_mod::mb_datetime_new(args)
}

// ---- iteration: pure day-sequence helpers ----

/// CPython Calendar.itermonthdays: 0-padded day numbers for whole weeks.
fn monthdays_seq(fw: i64, y: i64, m: i64) -> Vec<i64> {
    let (day1, ndays) = monthrange(y, m);
    let days_before = (day1 - fw).rem_euclid(7);
    let days_after = (fw - day1 - ndays).rem_euclid(7);
    let mut out = Vec::with_capacity((days_before + ndays + days_after) as usize);
    for _ in 0..days_before { out.push(0); }
    for d in 1..=ndays { out.push(d); }
    for _ in 0..days_after { out.push(0); }
    out
}

/// CPython Calendar.itermonthdates: consecutive real dates (y, m, d) covering
/// whole weeks. Walks ordinals so prev/next-month spill is real.
fn monthdates_seq(fw: i64, y: i64, m: i64) -> Vec<(i64, i64, i64)> {
    let (day1, ndays) = monthrange(y, m);
    let days_before = (day1 - fw).rem_euclid(7);
    let days_after = (fw - day1 - ndays).rem_euclid(7);
    let first_ord = date_to_ordinal(y, m, 1) - days_before;
    let total = days_before + ndays + days_after;
    (0..total).map(|i| ordinal_to_date(first_ord + i)).collect()
}

// ---- iteration method bodies (firstweekday baked in) ----

fn iterweekdays_impl(fw: i64) -> MbValue {
    let vals: Vec<MbValue> = (0..7).map(|i| MbValue::from_int((fw + i) % 7)).collect();
    MbValue::from_ptr(MbObject::new_list(vals))
}

fn itermonthdates_impl(fw: i64, y: i64, m: i64) -> MbValue {
    let vals: Vec<MbValue> = monthdates_seq(fw, y, m)
        .into_iter()
        .map(|(yy, mm, dd)| make_date(yy, mm, dd))
        .collect();
    MbValue::from_ptr(MbObject::new_list(vals))
}

fn itermonthdays_impl(fw: i64, y: i64, m: i64) -> MbValue {
    let vals: Vec<MbValue> = monthdays_seq(fw, y, m)
        .into_iter()
        .map(MbValue::from_int)
        .collect();
    MbValue::from_ptr(MbObject::new_list(vals))
}

fn itermonthdays2_impl(fw: i64, y: i64, m: i64) -> MbValue {
    // CPython: itermonthdays2 validates the month (via monthrange) and raises
    // IllegalMonthError for month ∉ 1..=12. Fire ONLY on the invalid month so
    // valid input is untouched; mirror monthrange's message verbatim.
    if !(1..=12).contains(&m) {
        return calendar_raise("IllegalMonthError", format!("bad month number {m}; must be 1-12"));
    }
    let vals: Vec<MbValue> = monthdays_seq(fw, y, m)
        .into_iter()
        .enumerate()
        .map(|(i, d)| {
            MbValue::from_ptr(MbObject::new_tuple(vec![
                MbValue::from_int(d),
                MbValue::from_int((fw + i as i64) % 7),
            ]))
        })
        .collect();
    MbValue::from_ptr(MbObject::new_list(vals))
}

fn itermonthdays3_impl(fw: i64, y: i64, m: i64) -> MbValue {
    let vals: Vec<MbValue> = monthdates_seq(fw, y, m)
        .into_iter()
        .map(|(yy, mm, dd)| {
            MbValue::from_ptr(MbObject::new_tuple(vec![
                MbValue::from_int(yy),
                MbValue::from_int(mm),
                MbValue::from_int(dd),
            ]))
        })
        .collect();
    MbValue::from_ptr(MbObject::new_list(vals))
}

fn itermonthdays4_impl(fw: i64, y: i64, m: i64) -> MbValue {
    let vals: Vec<MbValue> = monthdates_seq(fw, y, m)
        .into_iter()
        .enumerate()
        .map(|(i, (yy, mm, dd))| {
            MbValue::from_ptr(MbObject::new_tuple(vec![
                MbValue::from_int(yy),
                MbValue::from_int(mm),
                MbValue::from_int(dd),
                MbValue::from_int((fw + i as i64) % 7),
            ]))
        })
        .collect();
    MbValue::from_ptr(MbObject::new_list(vals))
}

/// Chunk a flat per-month sequence into weeks of 7, mapping each cell with `f`.
fn weeks_of<T, F>(cells: Vec<T>, f: F) -> MbValue
where
    F: Fn(&T) -> MbValue,
{
    let weeks: Vec<MbValue> = cells
        .chunks(7)
        .map(|wk| MbValue::from_ptr(MbObject::new_list(wk.iter().map(&f).collect())))
        .collect();
    MbValue::from_ptr(MbObject::new_list(weeks))
}

fn monthdatescalendar_impl(fw: i64, y: i64, m: i64) -> MbValue {
    weeks_of(monthdates_seq(fw, y, m), |(yy, mm, dd)| make_date(*yy, *mm, *dd))
}

fn monthdayscalendar_impl(fw: i64, y: i64, m: i64) -> MbValue {
    weeks_of(monthdays_seq(fw, y, m), |d| MbValue::from_int(*d))
}

fn monthdays2calendar_impl(fw: i64, y: i64, m: i64) -> MbValue {
    let pairs: Vec<(i64, i64)> = monthdays_seq(fw, y, m)
        .into_iter()
        .enumerate()
        .map(|(i, d)| (d, (fw + i as i64) % 7))
        .collect();
    weeks_of(pairs, |(d, wd)| {
        MbValue::from_ptr(MbObject::new_tuple(vec![
            MbValue::from_int(*d),
            MbValue::from_int(*wd),
        ]))
    })
}

/// CPython yeardatescalendar(year, width=3): nest month-week grids into rows of
/// `width` months. Default width 3.
fn yeardatescalendar_impl(fw: i64, year: i64, width: i64) -> MbValue {
    let width = if width <= 0 { 3 } else { width };
    let mut rows: Vec<MbValue> = Vec::new();
    let mut m = 1i64;
    while m <= 12 {
        let row: Vec<MbValue> = (m..(m + width).min(13))
            .map(|mm| monthdatescalendar_impl(fw, year, mm))
            .collect();
        rows.push(MbValue::from_ptr(MbObject::new_list(row)));
        m += width;
    }
    MbValue::from_ptr(MbObject::new_list(rows))
}

fn yeardays2calendar_impl(fw: i64, year: i64, width: i64) -> MbValue {
    let width = if width <= 0 { 3 } else { width };
    let mut rows: Vec<MbValue> = Vec::new();
    let mut m = 1i64;
    while m <= 12 {
        let row: Vec<MbValue> = (m..(m + width).min(13))
            .map(|mm| monthdays2calendar_impl(fw, year, mm))
            .collect();
        rows.push(MbValue::from_ptr(MbObject::new_list(row)));
        m += width;
    }
    MbValue::from_ptr(MbObject::new_list(rows))
}

fn yeardayscalendar_impl(fw: i64, year: i64, width: i64) -> MbValue {
    let width = if width <= 0 { 3 } else { width };
    let mut rows: Vec<MbValue> = Vec::new();
    let mut m = 1i64;
    while m <= 12 {
        let row: Vec<MbValue> = (m..(m + width).min(13))
            .map(|mm| monthdayscalendar_impl(fw, year, mm))
            .collect();
        rows.push(MbValue::from_ptr(MbObject::new_list(row)));
        m += width;
    }
    MbValue::from_ptr(MbObject::new_list(rows))
}

// ---- argument helpers ----

/// Strip a trailing kwargs dict (emitted by method-kwargs lowering) and return
/// the positional slice plus the dict.
fn split_args(a: &[MbValue]) -> (&[MbValue], Option<MbValue>) {
    if let Some(last) = a.last() {
        if let Some(ptr) = last.as_ptr() {
            unsafe {
                if let ObjData::Dict(_) = (*ptr).data {
                    return (&a[..a.len() - 1], Some(*last));
                }
            }
        }
    }
    (a, None)
}

fn kwarg_str(kw: &Option<MbValue>, key: &str) -> Option<String> {
    let kw = kw.as_ref()?;
    let ptr = kw.as_ptr()?;
    unsafe {
        if let ObjData::Dict(ref lock) = (*ptr).data {
            if let Some(v) = lock.read().unwrap()
                .get(&super::super::dict_ops::DictKey::Str(key.to_string()))
                .copied()
            {
                if v.is_none() {
                    return None;
                }
                return v.as_ptr().and_then(|p| match &(*p).data {
                    ObjData::Str(ref s) => Some(s.clone()),
                    _ => None,
                });
            }
        }
    }
    None
}

fn kwarg_bool(kw: &Option<MbValue>, key: &str, default: bool) -> bool {
    let Some(kw) = kw else { return default; };
    let Some(ptr) = kw.as_ptr() else { return default; };
    unsafe {
        if let ObjData::Dict(ref lock) = (*ptr).data {
            if let Some(v) = lock.read().unwrap()
                .get(&super::super::dict_ops::DictKey::Str(key.to_string()))
                .copied()
            {
                return v.as_bool().unwrap_or(default);
            }
        }
    }
    default
}

fn arg_int(a: &[MbValue], i: usize, default: i64) -> i64 {
    a.get(i).and_then(|v| v.as_int()).unwrap_or(default)
}

/// Read an int keyword argument from the trailing kwargs dict.
fn kwarg_int(kw: &Option<MbValue>, key: &str) -> Option<i64> {
    let kw = kw.as_ref()?;
    let ptr = kw.as_ptr()?;
    unsafe {
        if let ObjData::Dict(ref lock) = (*ptr).data {
            if let Some(v) = lock.read().unwrap()
                .get(&super::super::dict_ops::DictKey::Str(key.to_string()))
                .copied()
            {
                return v.as_int();
            }
        }
    }
    None
}

/// Resolve a parameter that may arrive positionally at `pos[i]` or as the named
/// keyword `key`; falls back to `default`.
fn pos_or_kw_int(pos: &[MbValue], i: usize, kw: &Option<MbValue>, key: &str, default: i64) -> i64 {
    if let Some(v) = pos.get(i).and_then(|v| v.as_int()) {
        return v;
    }
    kwarg_int(kw, key).unwrap_or(default)
}

/// Module-global firstweekday (set by `setfirstweekday`).
fn module_fw() -> i64 {
    FIRST_WEEKDAY.load(Ordering::Relaxed).rem_euclid(7)
}

/// Collect the string elements of a list/tuple MbValue (for `format`/`formatstring`).
fn read_str_cols(v: MbValue) -> Vec<String> {
    let items: Vec<MbValue> = v.as_ptr().map(|ptr| unsafe {
        match &(*ptr).data {
            ObjData::List(ref lock) => lock.read().unwrap().to_vec(),
            ObjData::Tuple(items) => items.clone(),
            _ => Vec::new(),
        }
    }).unwrap_or_default();
    items.iter().map(|it| {
        it.as_ptr().and_then(|p| unsafe {
            if let ObjData::Str(ref s) = (*p).data { Some(s.clone()) } else { None }
        }).unwrap_or_default()
    }).collect()
}

// ---- Module-level convenience functions (bound to a global TextCalendar) ----

/// calendar.month(theyear, themonth, w=0, l=0) -> formatmonth text.
unsafe extern "C" fn dispatch_month(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let all = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let (pos, kw) = split_args(all);
    let year = pos_or_kw_int(pos, 0, &kw, "theyear", 1970);
    let month = pos_or_kw_int(pos, 1, &kw, "themonth", 1);
    let w = pos_or_kw_int(pos, 2, &kw, "w", 0).max(2) as usize;
    let l = pos_or_kw_int(pos, 3, &kw, "l", 0).max(1) as usize;
    MbValue::from_ptr(MbObject::new_str(fmt_text_month(module_fw(), year, month, w, l)))
}

/// calendar.prmonth(theyear, themonth, w=0, l=0) -> prints formatmonth.
unsafe extern "C" fn dispatch_prmonth(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let all = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let (pos, kw) = split_args(all);
    let year = pos_or_kw_int(pos, 0, &kw, "theyear", 1970);
    let month = pos_or_kw_int(pos, 1, &kw, "themonth", 1);
    let w = pos_or_kw_int(pos, 2, &kw, "w", 0).max(2) as usize;
    let l = pos_or_kw_int(pos, 3, &kw, "l", 0).max(1) as usize;
    calendar_emit(&fmt_text_month(module_fw(), year, month, w, l));
    MbValue::none()
}

/// calendar.calendar(theyear, w=2, l=1, c=6, m=3) -> formatyear text.
unsafe extern "C" fn dispatch_calendar_fn(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let all = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let (pos, kw) = split_args(all);
    let year = pos_or_kw_int(pos, 0, &kw, "theyear", 1970);
    let w = pos_or_kw_int(pos, 1, &kw, "w", 2).max(2) as usize;
    let l = pos_or_kw_int(pos, 2, &kw, "l", 1).max(1) as usize;
    let c = pos_or_kw_int(pos, 3, &kw, "c", 6) as usize;
    let m = pos_or_kw_int(pos, 4, &kw, "m", 3).max(1) as usize;
    MbValue::from_ptr(MbObject::new_str(fmt_text_year(module_fw(), year, w, l, c, m)))
}

/// calendar.prcal(theyear, w=0, l=0, c=6, m=3) -> prints formatyear.
unsafe extern "C" fn dispatch_prcal(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let all = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let (pos, kw) = split_args(all);
    let year = pos_or_kw_int(pos, 0, &kw, "theyear", 1970);
    let w = pos_or_kw_int(pos, 1, &kw, "w", 2).max(2) as usize;
    let l = pos_or_kw_int(pos, 2, &kw, "l", 1).max(1) as usize;
    let c = pos_or_kw_int(pos, 3, &kw, "c", 6) as usize;
    let m = pos_or_kw_int(pos, 4, &kw, "m", 3).max(1) as usize;
    calendar_emit(&fmt_text_year(module_fw(), year, w, l, c, m));
    MbValue::none()
}

/// calendar.weekheader(width) -> formatweekheader text.
unsafe extern "C" fn dispatch_weekheader(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let all = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let (pos, kw) = split_args(all);
    let width = pos_or_kw_int(pos, 0, &kw, "width", 0).max(1) as usize;
    MbValue::from_ptr(MbObject::new_str(fmt_text_weekheader(module_fw(), width)))
}

/// calendar.prweek(theweek, width) -> prints formatweek.
unsafe extern "C" fn dispatch_prweek(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let all = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let (pos, _kw) = split_args(all);
    let width = arg_int(pos, 1, 0).max(2) as usize;
    let week = read_week_pairs(pos.first().copied().unwrap_or_else(MbValue::none));
    calendar_emit(&fmt_text_week(&week, width));
    MbValue::none()
}

/// calendar.week(theweek, width) -> formatweek text.
unsafe extern "C" fn dispatch_week(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let all = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let (pos, _kw) = split_args(all);
    let width = arg_int(pos, 1, 0).max(2) as usize;
    let week = read_week_pairs(pos.first().copied().unwrap_or_else(MbValue::none));
    MbValue::from_ptr(MbObject::new_str(fmt_text_week(&week, width)))
}

/// calendar.formatstring(cols, colwidth=20, spacing=6) -> centered/joined text.
unsafe extern "C" fn dispatch_formatstring(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let all = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let (pos, kw) = split_args(all);
    let cols = read_str_cols(pos.first().copied().unwrap_or_else(MbValue::none));
    let colwidth = pos_or_kw_int(pos, 1, &kw, "colwidth", 20) as usize;
    let spacing = pos_or_kw_int(pos, 2, &kw, "spacing", 6) as usize;
    MbValue::from_ptr(MbObject::new_str(formatstring(&cols, colwidth, spacing)))
}

/// calendar.format(cols, colwidth=20, spacing=6) -> prints formatstring.
unsafe extern "C" fn dispatch_format(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let all = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let (pos, kw) = split_args(all);
    let cols = read_str_cols(pos.first().copied().unwrap_or_else(MbValue::none));
    let colwidth = pos_or_kw_int(pos, 1, &kw, "colwidth", 20) as usize;
    let spacing = pos_or_kw_int(pos, 2, &kw, "spacing", 6) as usize;
    // CPython's calendar.format() prints with a trailing newline (print()).
    let mut s = formatstring(&cols, colwidth, spacing);
    s.push('\n');
    calendar_emit(&s);
    MbValue::none()
}

// ---- TextCalendar formatting (firstweekday-dependent where noted) ----

/// CPython TextCalendar.formatday: `'%2i' % day` (min width 2) then centered in
/// `width`; blank string centered for day 0.
fn fmt_text_day(day: i64, width: usize) -> String {
    let s = if day == 0 {
        String::new()
    } else {
        format!("{:>2}", day)
    };
    center(&s, width)
}

/// CPython TextCalendar.formatweek: a single week row joined by single spaces.
fn fmt_text_week(week: &[(i64, i64)], width: usize) -> String {
    week.iter()
        .map(|(d, _)| fmt_text_day(*d, width))
        .collect::<Vec<_>>()
        .join(" ")
}

/// CPython TextCalendar.formatweekday: `names[day][:width].center(width)`.
fn fmt_text_weekday(wd: i64, width: usize) -> String {
    let name = if width >= 9 {
        DAY_FULL[wd as usize % 7]
    } else {
        DAY_ABBR3[wd as usize % 7]
    };
    let name: String = name.chars().take(width).collect();
    center(&name, width)
}

fn fmt_text_weekheader(fw: i64, width: usize) -> String {
    (0..7)
        .map(|i| fmt_text_weekday((fw + i) % 7, width))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Center `s` in `width`, matching CPython's `str.center` padding rule:
/// `left = marg/2 + (marg & width & 1)` so the extra space lands on the left
/// when both the margin and the width are odd.
fn center(s: &str, width: usize) -> String {
    let len = s.chars().count();
    if len >= width {
        return s.to_string();
    }
    let marg = width - len;
    let left = marg / 2 + (marg & width & 1);
    let right = marg - left;
    format!("{}{}{}", " ".repeat(left), s, " ".repeat(right))
}

/// CPython TextCalendar.formatmonth(theyear, themonth, w=0, l=0). Each line is
/// rstripped; `'\n'*l` follows the header, weekheader, and every week row.
fn fmt_text_month(fw: i64, year: i64, month: i64, w: usize, l: usize) -> String {
    let w = w.max(2);
    let l = l.max(1);
    let nl = "\n".repeat(l);
    let mut s = String::new();
    s.push_str(rstrip(&fmt_text_monthname_w(year, month, 7 * (w + 1) - 1, true)));
    s.push_str(&nl);
    s.push_str(rstrip(&fmt_text_weekheader(fw, w)));
    s.push_str(&nl);
    let weeks: Vec<(i64, i64)> = monthdays_seq(fw, year, month)
        .into_iter()
        .enumerate()
        .map(|(i, d)| (d, (fw + i as i64) % 7))
        .collect();
    for wk in weeks.chunks(7) {
        s.push_str(rstrip(&fmt_text_week(wk, w)));
        s.push_str(&nl);
    }
    s
}

/// Trim trailing ASCII spaces (Python str.rstrip default strips whitespace; the
/// calendar output only ever has trailing spaces).
fn rstrip(s: &str) -> &str {
    s.trim_end_matches(' ')
}

/// CPython TextCalendar.formatmonthname(theyear, themonth, width, withyear):
/// `month_name[m]` (+ " " + repr(year) when withyear) centered in `width`.
fn fmt_text_monthname_w(year: i64, month: i64, width: usize, withyear: bool) -> String {
    let s = if withyear {
        format!("{} {}", MONTH_FULL[month as usize % 13], year)
    } else {
        MONTH_FULL[month as usize % 13].to_string()
    };
    center(&s, width)
}

/// Public-facing formatmonthname used by the standalone dispatcher: callers
/// pass the already-computed header width.
fn fmt_text_monthname(year: i64, month: i64, width: usize, withyear: bool) -> String {
    fmt_text_monthname_w(year, month, width, withyear)
}

/// CPython TextCalendar.formatyear(theyear, w=2, l=1, c=6, m=3).
fn fmt_text_year(fw: i64, year: i64, w: usize, l: usize, c: usize, m: usize) -> String {
    let w = w.max(2);
    let l = l.max(1);
    let c = c.max(2);
    let m = m.max(1) as i64;
    let colwidth = (w + 1) * 7 - 1;
    let nl = "\n".repeat(l);
    let mut out = String::new();
    // Year header centered across the full grid width, then rstripped.
    let full = colwidth * m as usize + c * (m as usize - 1);
    out.push_str(rstrip(&center(&year.to_string(), full)));
    out.push_str(&nl);
    let header = fmt_text_weekheader(fw, w);
    let nrows = (12 + m - 1) / m;
    for i in 0..nrows {
        let months: Vec<i64> = ((m * i + 1)..((m * (i + 1) + 1).min(13))).collect();
        out.push_str(&nl);
        // month-name row
        let names: Vec<String> = months
            .iter()
            .map(|&k| fmt_text_monthname(year, k, colwidth, false))
            .collect();
        out.push_str(rstrip(&formatstring(&names, colwidth, c)));
        out.push_str(&nl);
        // weekday-header row
        let headers: Vec<String> = months.iter().map(|_| header.clone()).collect();
        out.push_str(rstrip(&formatstring(&headers, colwidth, c)));
        out.push_str(&nl);
        // per-month week grids
        let cals: Vec<Vec<Vec<(i64, i64)>>> = months
            .iter()
            .map(|&k| {
                monthdays_seq(fw, year, k)
                    .into_iter()
                    .enumerate()
                    .map(|(idx, d)| (d, (fw + idx as i64) % 7))
                    .collect::<Vec<_>>()
                    .chunks(7)
                    .map(|wk| wk.to_vec())
                    .collect::<Vec<_>>()
            })
            .collect();
        let height = cals.iter().map(|cal| cal.len()).max().unwrap_or(0);
        for j in 0..height {
            let weeks: Vec<String> = cals
                .iter()
                .map(|cal| {
                    if j >= cal.len() {
                        String::new()
                    } else {
                        fmt_text_week(&cal[j], w)
                    }
                })
                .collect();
            out.push_str(rstrip(&formatstring(&weeks, colwidth, c)));
            out.push_str(&nl);
        }
    }
    out
}

/// CPython calendar.formatstring(cols, colwidth, spacing): each column centered
/// in `colwidth`, joined by `spacing` spaces.
fn formatstring(cols: &[String], colwidth: usize, spacing: usize) -> String {
    cols.iter()
        .map(|c| center(c, colwidth))
        .collect::<Vec<_>>()
        .join(&" ".repeat(spacing))
}

// ---- TextCalendar method dispatchers ----

unsafe extern "C" fn text_formatday(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let day = arg_int(a, 0, 0);
    let _weekday = arg_int(a, 1, 0);
    let width = arg_int(a, 2, 2).max(2) as usize;
    MbValue::from_ptr(MbObject::new_str(fmt_text_day(day, width)))
}

unsafe extern "C" fn text_formatweek(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let width = arg_int(a, 1, 2).max(2) as usize;
    let week = read_week_pairs(a.first().copied().unwrap_or_else(MbValue::none));
    MbValue::from_ptr(MbObject::new_str(fmt_text_week(&week, width)))
}

unsafe extern "C" fn text_formatweekday(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let wd = arg_int(a, 0, 0);
    let width = arg_int(a, 1, 3).max(1) as usize;
    MbValue::from_ptr(MbObject::new_str(fmt_text_weekday(wd, width)))
}

unsafe extern "C" fn text_prweek(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let width = arg_int(a, 1, 2).max(2) as usize;
    let week = read_week_pairs(a.first().copied().unwrap_or_else(MbValue::none));
    calendar_emit(&fmt_text_week(&week, width));
    MbValue::none()
}

fn read_week_pairs(v: MbValue) -> Vec<(i64, i64)> {
    let items: Vec<MbValue> = v.as_ptr().map(|ptr| unsafe {
        match &(*ptr).data {
            ObjData::List(ref lock) => lock.read().unwrap().to_vec(),
            ObjData::Tuple(items) => items.clone(),
            _ => Vec::new(),
        }
    }).unwrap_or_default();
    items.iter().map(|pair| {
        pair.as_ptr().map(|p| unsafe {
            match &(*p).data {
                ObjData::Tuple(it) => (
                    it.first().and_then(|x| x.as_int()).unwrap_or(0),
                    it.get(1).and_then(|x| x.as_int()).unwrap_or(0),
                ),
                ObjData::List(ref lk) => {
                    let g = lk.read().unwrap();
                    (
                        g.first().and_then(|x| x.as_int()).unwrap_or(0),
                        g.get(1).and_then(|x| x.as_int()).unwrap_or(0),
                    )
                }
                _ => (0, 0),
            }
        }).unwrap_or((0, 0))
    }).collect()
}

unsafe extern "C" fn html_formatweekday(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let wd = arg_int(a, 0, 0).rem_euclid(7);
    MbValue::from_ptr(MbObject::new_str(format!(
        "<th class=\"{}\">{}</th>",
        HTML_CSS_WEEKDAY_HEAD[wd as usize], DAY_ABBR3[wd as usize]
    )))
}

unsafe extern "C" fn html_formatweekheader(args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    let _ = args_ptr;
    let mut s = String::from("<tr>");
    for wd in 0..7 {
        s.push_str(&format!(
            "<th class=\"{}\">{}</th>",
            HTML_CSS_WEEKDAY_HEAD[wd], DAY_ABBR3[wd]
        ));
    }
    s.push_str("</tr>");
    MbValue::from_ptr(MbObject::new_str(s))
}

unsafe extern "C" fn html_formatday(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let day = arg_int(a, 0, 0);
    let wd = arg_int(a, 1, 0).rem_euclid(7);
    let s = if day == 0 {
        "<td class=\"noday\">&nbsp;</td>".to_string()
    } else {
        format!("<td class=\"{}\">{}</td>", HTML_CSS[wd as usize], day)
    };
    MbValue::from_ptr(MbObject::new_str(s))
}

unsafe extern "C" fn html_formatweek(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let week = read_week_pairs(a.first().copied().unwrap_or_else(MbValue::none));
    let mut s = String::from("<tr>");
    for (d, wd) in &week {
        if *d == 0 {
            s.push_str("<td class=\"noday\">&nbsp;</td>");
        } else {
            s.push_str(&format!("<td class=\"{}\">{}</td>", HTML_CSS[*wd as usize % 7], d));
        }
    }
    s.push_str("</tr>");
    MbValue::from_ptr(MbObject::new_str(s))
}

unsafe extern "C" fn html_formatmonthname(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let all = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let (a, kw) = split_args(all);
    let year = arg_int(a, 0, 1970);
    let month = arg_int(a, 1, 1);
    let withyear = if a.len() >= 3 {
        a[2].as_bool().unwrap_or(true)
    } else {
        kwarg_bool(&kw, "withyear", true)
    };
    let s = if withyear {
        format!(
            "<tr><th colspan=\"7\" class=\"{}\">{} {}</th></tr>",
            HTML_CSS_MONTH_HEAD, MONTH_FULL[month as usize % 13], year
        )
    } else {
        format!(
            "<tr><th colspan=\"7\" class=\"{}\">{}</th></tr>",
            HTML_CSS_MONTH_HEAD, MONTH_FULL[month as usize % 13]
        )
    };
    MbValue::from_ptr(MbObject::new_str(s))
}

fn html_format_month(fw: i64, year: i64, month: i64, withyear: bool) -> String {
    let mut s = String::new();
    s.push_str(&format!(
        "<table border=\"0\" cellpadding=\"0\" cellspacing=\"0\" class=\"{}\">\n",
        HTML_CSS_MONTH
    ));
    // month name
    let name = if withyear {
        format!(
            "<tr><th colspan=\"7\" class=\"{}\">{} {}</th></tr>",
            HTML_CSS_MONTH_HEAD, MONTH_FULL[month as usize % 13], year
        )
    } else {
        format!(
            "<tr><th colspan=\"7\" class=\"{}\">{}</th></tr>",
            HTML_CSS_MONTH_HEAD, MONTH_FULL[month as usize % 13]
        )
    };
    s.push_str(&name);
    s.push('\n');
    // weekday header
    s.push_str("<tr>");
    for wd in 0..7 {
        s.push_str(&format!(
            "<th class=\"{}\">{}</th>",
            HTML_CSS_WEEKDAY_HEAD[wd], DAY_ABBR3[wd]
        ));
    }
    s.push_str("</tr>\n");
    // weeks
    let pairs: Vec<(i64, i64)> = monthdays_seq(fw, year, month)
        .into_iter()
        .enumerate()
        .map(|(i, d)| (d, (fw + i as i64) % 7))
        .collect();
    for wk in pairs.chunks(7) {
        s.push_str("<tr>");
        for (d, wd) in wk {
            if *d == 0 {
                s.push_str("<td class=\"noday\">&nbsp;</td>");
            } else {
                s.push_str(&format!("<td class=\"{}\">{}</td>", HTML_CSS[*wd as usize % 7], d));
            }
        }
        s.push_str("</tr>\n");
    }
    s.push_str("</table>\n");
    s
}

fn html_format_year_fw(fw: i64, year: i64, width: i64) -> String {
    let width = if width <= 0 { 3 } else { width };
    let mut s = String::new();
    s.push_str(&format!(
        "<table border=\"0\" cellpadding=\"0\" cellspacing=\"0\" class=\"{}\">\n",
        HTML_CSS_YEAR
    ));
    s.push_str(&format!(
        "<tr><th colspan=\"{}\" class=\"{}\">{}</th></tr>",
        width, HTML_CSS_YEAR_HEAD, year
    ));
    let mut m = 1i64;
    while m <= 12 {
        s.push_str("<tr>");
        for mm in m..(m + width).min(13) {
            s.push_str("<td>");
            s.push_str(&html_format_month(fw, year, mm, false));
            s.push_str("</td>");
        }
        s.push_str("</tr>");
        m += width;
    }
    s.push_str("</table>");
    s
}

fn html_format_yearpage(fw: i64, year: i64, width: i64, encoding: &str) -> String {
    let enc = if encoding.is_empty() { "utf-8" } else { encoding };
    let mut s = String::new();
    s.push_str(&format!("<?xml version=\"1.0\" encoding=\"{}\"?>\n", enc));
    s.push_str("<!DOCTYPE html PUBLIC \"-//W3C//DTD XHTML 1.0 Strict//EN\" \"http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd\">\n");
    s.push_str("<html>\n<head>\n");
    s.push_str(&format!(
        "<meta http-equiv=\"Content-Type\" content=\"text/html; charset={}\" />\n",
        enc
    ));
    s.push_str("<link rel=\"stylesheet\" type=\"text/css\" href=\"calendar.css\" />\n");
    s.push_str(&format!("<title>Calendar for {}</title>\n", year));
    s.push_str("</head>\n<body>\n");
    s.push_str(&html_format_year_fw(fw, year, width));
    s.push_str("</body>\n</html>\n");
    s
}

// ---- HTMLCalendar theming class attributes ----

const HTML_CSS: [&str; 7] = ["mon", "tue", "wed", "thu", "fri", "sat", "sun"];
const HTML_CSS_WEEKDAY_HEAD: [&str; 7] = ["mon", "tue", "wed", "thu", "fri", "sat", "sun"];
const HTML_CSS_MONTH: &str = "month";
const HTML_CSS_MONTH_HEAD: &str = "month";
const HTML_CSS_YEAR: &str = "year";
const HTML_CSS_YEAR_HEAD: &str = "year";

// ---- Receiver-aware HTMLCalendar methods (user-subclass support) ----
//
// The instance-field method table above is flat-args (no self), so it cannot
// honor `class Themed(calendar.HTMLCalendar): cssclass_month = ...`
// overrides — a subclass instance has neither the method fields nor the
// theme defaults. mb_call_method routes subclass receivers here instead;
// every css value is read off the receiver (instance fields → user class
// attrs via mb_getattr) with the CPython default as fallback.

/// getattr(recv, name) as a String, with a default for missing/None.
fn recv_css_str(recv: MbValue, name: &str, default: &str) -> String {
    let v = super::super::class::mb_getattr(
        recv,
        MbValue::from_ptr(MbObject::new_str(name.to_string())),
    );
    extract_str_val(v).unwrap_or_else(|| default.to_string())
}

/// getattr(recv, name) as a 7-element Vec<String> (per-weekday css classes).
fn recv_css_list(recv: MbValue, name: &str, default: &[&str; 7]) -> Vec<String> {
    let v = super::super::class::mb_getattr(
        recv,
        MbValue::from_ptr(MbObject::new_str(name.to_string())),
    );
    if let Some(ptr) = v.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                let items: Vec<String> = lock
                    .read()
                    .unwrap()
                    .iter()
                    .filter_map(|x| extract_str_val(*x))
                    .collect();
                if items.len() == 7 {
                    return items;
                }
            }
        }
    }
    default.iter().map(|s| s.to_string()).collect()
}

fn extract_str_val(v: MbValue) -> Option<String> {
    v.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data { Some(s.clone()) } else { None }
    })
}

fn recv_firstweekday(recv: MbValue) -> i64 {
    let v = super::super::class::mb_getattr(
        recv,
        MbValue::from_ptr(MbObject::new_str("firstweekday".to_string())),
    );
    v.as_int().unwrap_or(0).rem_euclid(7)
}

fn themed_formatday(recv: MbValue, day: i64, weekday: i64) -> String {
    if day == 0 {
        format!(
            "<td class=\"{}\">&nbsp;</td>",
            recv_css_str(recv, "cssclass_noday", "noday")
        )
    } else {
        let css = recv_css_list(recv, "cssclasses", &HTML_CSS);
        format!("<td class=\"{}\">{}</td>", css[weekday.rem_euclid(7) as usize], day)
    }
}

fn themed_formatweek(recv: MbValue, week: &[(i64, i64)]) -> String {
    let cells: String = week.iter().map(|(d, wd)| themed_formatday(recv, *d, *wd)).collect();
    format!("<tr>{cells}</tr>")
}

fn themed_formatweekheader(recv: MbValue) -> String {
    let fw = recv_firstweekday(recv);
    let css = recv_css_list(recv, "cssclasses_weekday_head", &HTML_CSS_WEEKDAY_HEAD);
    let mut s = String::from("<tr>");
    for i in 0..7 {
        let wd = ((fw + i) % 7) as usize;
        s.push_str(&format!("<th class=\"{}\">{}</th>", css[wd], DAY_ABBR3[wd]));
    }
    s.push_str("</tr>");
    s
}

fn themed_formatmonthname(recv: MbValue, year: i64, month: i64, withyear: bool) -> String {
    let css = recv_css_str(recv, "cssclass_month_head", HTML_CSS_MONTH_HEAD);
    if withyear {
        format!(
            "<tr><th colspan=\"7\" class=\"{}\">{} {}</th></tr>",
            css, MONTH_FULL[month as usize % 13], year
        )
    } else {
        format!(
            "<tr><th colspan=\"7\" class=\"{}\">{}</th></tr>",
            css, MONTH_FULL[month as usize % 13]
        )
    }
}

fn themed_format_month(recv: MbValue, year: i64, month: i64, withyear: bool) -> String {
    let fw = recv_firstweekday(recv);
    let mut s = String::new();
    s.push_str(&format!(
        "<table border=\"0\" cellpadding=\"0\" cellspacing=\"0\" class=\"{}\">\n",
        recv_css_str(recv, "cssclass_month", HTML_CSS_MONTH)
    ));
    s.push_str(&themed_formatmonthname(recv, year, month, withyear));
    s.push('\n');
    s.push_str(&themed_formatweekheader(recv));
    s.push('\n');
    let pairs: Vec<(i64, i64)> = monthdays_seq(fw, year, month)
        .into_iter()
        .enumerate()
        .map(|(i, d)| (d, (fw + i as i64) % 7))
        .collect();
    for wk in pairs.chunks(7) {
        s.push_str(&themed_formatweek(recv, wk));
        s.push('\n');
    }
    s.push_str("</table>\n");
    s
}

fn themed_format_year(recv: MbValue, year: i64, width: i64) -> String {
    let width = if width <= 0 { 3 } else { width };
    let mut s = String::new();
    s.push_str(&format!(
        "<table border=\"0\" cellpadding=\"0\" cellspacing=\"0\" class=\"{}\">\n",
        recv_css_str(recv, "cssclass_year", HTML_CSS_YEAR)
    ));
    s.push_str(&format!(
        "<tr><th colspan=\"{}\" class=\"{}\">{}</th></tr>",
        width,
        recv_css_str(recv, "cssclass_year_head", HTML_CSS_YEAR_HEAD),
        year
    ));
    let mut m = 1i64;
    while m <= 12 {
        s.push_str("<tr>");
        for mm in m..(m + width).min(13) {
            s.push_str("<td>");
            s.push_str(&themed_format_month(recv, year, mm, false));
            s.push_str("</td>");
        }
        s.push_str("</tr>");
        m += width;
    }
    s.push_str("</table>");
    s
}

/// Extract a week argument (list of (day, weekday) 2-tuples).
fn week_arg(v: MbValue) -> Vec<(i64, i64)> {
    let mut out = Vec::new();
    if let Some(ptr) = v.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                for item in lock.read().unwrap().iter() {
                    if let Some(ip) = item.as_ptr() {
                        if let ObjData::Tuple(ref t) = (*ip).data {
                            if t.len() == 2 {
                                out.push((
                                    t[0].as_int_pyint().unwrap_or(0),
                                    t[1].as_int_pyint().unwrap_or(0),
                                ));
                            }
                        }
                    }
                }
            }
        }
    }
    out
}

/// Receiver-aware method dispatch for user subclasses of HTMLCalendar (and
/// the Calendar iteration surface they inherit). Returns None for methods
/// not modeled here so the caller can fall through.
pub fn html_calendar_subclass_method(
    recv: MbValue,
    method: &str,
    args: &[MbValue],
) -> Option<MbValue> {
    let arg_int = |i: usize, d: i64| args.get(i).and_then(|v| v.as_int_pyint()).unwrap_or(d);
    let arg_bool = |i: usize, d: bool| {
        args.get(i)
            .map(|v| super::super::builtins::mb_is_truthy(*v) != 0)
            .unwrap_or(d)
    };
    let new_str = |s: String| MbValue::from_ptr(MbObject::new_str(s));
    match method {
        "formatmonth" => Some(new_str(themed_format_month(
            recv, arg_int(0, 1970), arg_int(1, 1), arg_bool(2, true),
        ))),
        "formatmonthname" => Some(new_str(themed_formatmonthname(
            recv, arg_int(0, 1970), arg_int(1, 1), arg_bool(2, true),
        ))),
        "formatweekheader" => Some(new_str(themed_formatweekheader(recv))),
        "formatweek" => {
            let week = week_arg(args.first().copied().unwrap_or_else(MbValue::none));
            Some(new_str(themed_formatweek(recv, &week)))
        }
        "formatday" => Some(new_str(themed_formatday(recv, arg_int(0, 0), arg_int(1, 0)))),
        "formatweekday" => {
            let wd = arg_int(0, 0).rem_euclid(7) as usize;
            let css = recv_css_list(recv, "cssclasses_weekday_head", &HTML_CSS_WEEKDAY_HEAD);
            Some(new_str(format!("<th class=\"{}\">{}</th>", css[wd], DAY_ABBR3[wd])))
        }
        "formatyear" => Some(new_str(themed_format_year(recv, arg_int(0, 1970), arg_int(1, 3)))),
        "monthdays2calendar" => {
            let fw = recv_firstweekday(recv);
            let (y, m) = (arg_int(0, 1970), arg_int(1, 1));
            let pairs: Vec<MbValue> = monthdays_seq(fw, y, m)
                .into_iter()
                .enumerate()
                .map(|(i, d)| {
                    MbValue::from_ptr(MbObject::new_tuple(vec![
                        MbValue::from_int(d),
                        MbValue::from_int((fw + i as i64) % 7),
                    ]))
                })
                .collect();
            let weeks: Vec<MbValue> = pairs
                .chunks(7)
                .map(|wk| MbValue::from_ptr(MbObject::new_list(wk.to_vec())))
                .collect();
            Some(MbValue::from_ptr(MbObject::new_list(weeks)))
        }
        _ => None,
    }
}

fn html_css_defaults(fields: &mut FxHashMap<String, MbValue>) {
    let css_list: Vec<MbValue> = HTML_CSS.iter()
        .map(|c| MbValue::from_ptr(MbObject::new_str(c.to_string())))
        .collect();
    fields.insert("cssclasses".to_string(), MbValue::from_ptr(MbObject::new_list(css_list)));
    let head_list: Vec<MbValue> = HTML_CSS_WEEKDAY_HEAD.iter()
        .map(|c| MbValue::from_ptr(MbObject::new_str(c.to_string())))
        .collect();
    fields.insert("cssclasses_weekday_head".to_string(), MbValue::from_ptr(MbObject::new_list(head_list)));
    fields.insert("cssclass_noday".to_string(), MbValue::from_ptr(MbObject::new_str("noday".to_string())));
    fields.insert("cssclass_month_head".to_string(), MbValue::from_ptr(MbObject::new_str(HTML_CSS_MONTH_HEAD.to_string())));
    fields.insert("cssclass_month".to_string(), MbValue::from_ptr(MbObject::new_str(HTML_CSS_MONTH.to_string())));
    fields.insert("cssclass_year".to_string(), MbValue::from_ptr(MbObject::new_str(HTML_CSS_YEAR.to_string())));
    fields.insert("cssclass_year_head".to_string(), MbValue::from_ptr(MbObject::new_str(HTML_CSS_YEAR_HEAD.to_string())));
}

// ---- Monomorphized dispatcher generation ----
//
// For each firstweekday-dependent method we emit 7 extern-C dispatchers (one
// per fw value) and collect their addresses into a `[usize; 7]` table indexed
// by firstweekday at construction time.

// Manual monomorphization (no paste crate dependency): define each variant
// explicitly via an inner macro that takes the fw literal + a unique fn ident.
macro_rules! def_fw_year_month {
    ($name:ident, $fw:expr, $impl_fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            let y = arg_int(a, 0, 1970);
            let m = arg_int(a, 1, 1);
            $impl_fn($fw, y, m)
        }
    };
}

macro_rules! def_fw_nullary {
    ($name:ident, $fw:expr, $impl_fn:ident) => {
        unsafe extern "C" fn $name(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
            $impl_fn($fw)
        }
    };
}

macro_rules! def_fw_year_width {
    ($name:ident, $fw:expr, $impl_fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            let y = arg_int(a, 0, 1970);
            let width = arg_int(a, 1, 3);
            $impl_fn($fw, y, width)
        }
    };
}

// iterweekdays (nullary)
def_fw_nullary!(iterweekdays_0, 0, iterweekdays_impl);
def_fw_nullary!(iterweekdays_1, 1, iterweekdays_impl);
def_fw_nullary!(iterweekdays_2, 2, iterweekdays_impl);
def_fw_nullary!(iterweekdays_3, 3, iterweekdays_impl);
def_fw_nullary!(iterweekdays_4, 4, iterweekdays_impl);
def_fw_nullary!(iterweekdays_5, 5, iterweekdays_impl);
def_fw_nullary!(iterweekdays_6, 6, iterweekdays_impl);
const ITERWEEKDAYS: [CalFn; 7] = [
    iterweekdays_0, iterweekdays_1, iterweekdays_2,
    iterweekdays_3, iterweekdays_4, iterweekdays_5, iterweekdays_6,
];

macro_rules! mono7_ym {
    ($table:ident, $impl_fn:ident, $n0:ident,$n1:ident,$n2:ident,$n3:ident,$n4:ident,$n5:ident,$n6:ident) => {
        def_fw_year_month!($n0, 0, $impl_fn);
        def_fw_year_month!($n1, 1, $impl_fn);
        def_fw_year_month!($n2, 2, $impl_fn);
        def_fw_year_month!($n3, 3, $impl_fn);
        def_fw_year_month!($n4, 4, $impl_fn);
        def_fw_year_month!($n5, 5, $impl_fn);
        def_fw_year_month!($n6, 6, $impl_fn);
        const $table: [CalFn; 7] = [$n0, $n1, $n2, $n3, $n4, $n5, $n6];
    };
}

macro_rules! mono7_yw {
    ($table:ident, $impl_fn:ident, $n0:ident,$n1:ident,$n2:ident,$n3:ident,$n4:ident,$n5:ident,$n6:ident) => {
        def_fw_year_width!($n0, 0, $impl_fn);
        def_fw_year_width!($n1, 1, $impl_fn);
        def_fw_year_width!($n2, 2, $impl_fn);
        def_fw_year_width!($n3, 3, $impl_fn);
        def_fw_year_width!($n4, 4, $impl_fn);
        def_fw_year_width!($n5, 5, $impl_fn);
        def_fw_year_width!($n6, 6, $impl_fn);
        const $table: [CalFn; 7] = [$n0, $n1, $n2, $n3, $n4, $n5, $n6];
    };
}

mono7_ym!(ITERMONTHDATES, itermonthdates_impl,
    imd_dates_0, imd_dates_1, imd_dates_2, imd_dates_3, imd_dates_4, imd_dates_5, imd_dates_6);
mono7_ym!(ITERMONTHDAYS, itermonthdays_impl,
    imd_0, imd_1, imd_2, imd_3, imd_4, imd_5, imd_6);
mono7_ym!(ITERMONTHDAYS2, itermonthdays2_impl,
    imd2_0, imd2_1, imd2_2, imd2_3, imd2_4, imd2_5, imd2_6);
mono7_ym!(ITERMONTHDAYS3, itermonthdays3_impl,
    imd3_0, imd3_1, imd3_2, imd3_3, imd3_4, imd3_5, imd3_6);
mono7_ym!(ITERMONTHDAYS4, itermonthdays4_impl,
    imd4_0, imd4_1, imd4_2, imd4_3, imd4_4, imd4_5, imd4_6);
mono7_ym!(MONTHDATESCALENDAR, monthdatescalendar_impl,
    mdc_0, mdc_1, mdc_2, mdc_3, mdc_4, mdc_5, mdc_6);
mono7_ym!(MONTHDAYS2CALENDAR, monthdays2calendar_impl,
    md2c_0, md2c_1, md2c_2, md2c_3, md2c_4, md2c_5, md2c_6);
mono7_ym!(MONTHDAYSCALENDAR, monthdayscalendar_impl,
    mdsc_0, mdsc_1, mdsc_2, mdsc_3, mdsc_4, mdsc_5, mdsc_6);
mono7_yw!(YEARDATESCALENDAR, yeardatescalendar_impl,
    ydc_0, ydc_1, ydc_2, ydc_3, ydc_4, ydc_5, ydc_6);
mono7_yw!(YEARDAYS2CALENDAR, yeardays2calendar_impl,
    yd2c_0, yd2c_1, yd2c_2, yd2c_3, yd2c_4, yd2c_5, yd2c_6);
mono7_yw!(YEARDAYSCALENDAR, yeardayscalendar_impl,
    ydsc_0, ydsc_1, ydsc_2, ydsc_3, ydsc_4, ydsc_5, ydsc_6);

// TextCalendar.formatweekheader(width)
unsafe extern "C" fn text_formatweekheader(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let width = arg_int(a, 0, 0).max(1) as usize;
    // TextCalendar.formatweekheader uses self.firstweekday. Default fw=0; for
    // non-default fw the monomorphized formatmonth path embeds it. The header
    // helper here defaults to fw=0 (covers the public formatweekheader cases).
    MbValue::from_ptr(MbObject::new_str(fmt_text_weekheader(0, width)))
}

// TextCalendar.formatmonthname(theyear, themonth, width=0, withyear=True)
macro_rules! def_text_monthname {
    ($name:ident, $fw:expr) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let all = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            let (a, kw) = split_args(all);
            let year = arg_int(a, 0, 1970);
            let month = arg_int(a, 1, 1);
            let width = arg_int(a, 2, 0).max(0) as usize;
            let withyear = if a.len() >= 4 {
                a[3].as_bool().unwrap_or(true)
            } else {
                kwarg_bool(&kw, "withyear", true)
            };
            let _ = $fw;
            MbValue::from_ptr(MbObject::new_str(fmt_text_monthname(year, month, width, withyear)))
        }
    };
}
def_text_monthname!(tmn_0, 0);
def_text_monthname!(tmn_1, 1);
def_text_monthname!(tmn_2, 2);
def_text_monthname!(tmn_3, 3);
def_text_monthname!(tmn_4, 4);
def_text_monthname!(tmn_5, 5);
def_text_monthname!(tmn_6, 6);
const TEXT_FORMATMONTHNAME: [CalFn; 7] = [
    tmn_0, tmn_1, tmn_2, tmn_3, tmn_4, tmn_5, tmn_6,
];

// TextCalendar.formatmonth(theyear, themonth, w=0, l=0)
macro_rules! def_text_month {
    ($name:ident, $fw:expr) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            let year = arg_int(a, 0, 1970);
            let month = arg_int(a, 1, 1);
            // CPython: formatmonth → monthdays2calendar → monthrange, which
            // raises IllegalMonthError for month ∉ 1..=12.
            if !(1..=12).contains(&month) {
                return calendar_raise(
                    "IllegalMonthError",
                    format!("bad month number {month}; must be 1-12"),
                );
            }
            let w = arg_int(a, 2, 0).max(2) as usize;
            let l = arg_int(a, 3, 0).max(1) as usize;
            MbValue::from_ptr(MbObject::new_str(fmt_text_month($fw, year, month, w, l)))
        }
    };
}
def_text_month!(tm_0, 0);
def_text_month!(tm_1, 1);
def_text_month!(tm_2, 2);
def_text_month!(tm_3, 3);
def_text_month!(tm_4, 4);
def_text_month!(tm_5, 5);
def_text_month!(tm_6, 6);
const TEXT_FORMATMONTH: [CalFn; 7] = [
    tm_0, tm_1, tm_2, tm_3, tm_4, tm_5, tm_6,
];

// TextCalendar.prmonth(theyear, themonth, w=0, l=0) — prints formatmonth.
macro_rules! def_text_prmonth {
    ($name:ident, $fw:expr) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            let year = arg_int(a, 0, 1970);
            let month = arg_int(a, 1, 1);
            let w = arg_int(a, 2, 0).max(2) as usize;
            let l = arg_int(a, 3, 0).max(1) as usize;
            calendar_emit(&fmt_text_month($fw, year, month, w, l));
            MbValue::none()
        }
    };
}
def_text_prmonth!(tpm_0, 0);
def_text_prmonth!(tpm_1, 1);
def_text_prmonth!(tpm_2, 2);
def_text_prmonth!(tpm_3, 3);
def_text_prmonth!(tpm_4, 4);
def_text_prmonth!(tpm_5, 5);
def_text_prmonth!(tpm_6, 6);
const TEXT_PRMONTH: [CalFn; 7] = [
    tpm_0, tpm_1, tpm_2, tpm_3, tpm_4, tpm_5, tpm_6,
];

// TextCalendar.formatyear(theyear, w=2, l=1, c=6, m=3)
macro_rules! def_text_year {
    ($name:ident, $fw:expr) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            let year = arg_int(a, 0, 1970);
            let w = arg_int(a, 1, 2).max(2) as usize;
            let l = arg_int(a, 2, 1).max(1) as usize;
            let c = arg_int(a, 3, 6) as usize;
            let m = arg_int(a, 4, 3).max(1) as usize;
            MbValue::from_ptr(MbObject::new_str(fmt_text_year($fw, year, w, l, c, m)))
        }
    };
}
def_text_year!(ty_0, 0);
def_text_year!(ty_1, 1);
def_text_year!(ty_2, 2);
def_text_year!(ty_3, 3);
def_text_year!(ty_4, 4);
def_text_year!(ty_5, 5);
def_text_year!(ty_6, 6);
const TEXT_FORMATYEAR: [CalFn; 7] = [
    ty_0, ty_1, ty_2, ty_3, ty_4, ty_5, ty_6,
];

// TextCalendar.pryear(theyear, ...) — prints formatyear.
macro_rules! def_text_pryear {
    ($name:ident, $fw:expr) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            let year = arg_int(a, 0, 1970);
            let w = arg_int(a, 1, 2).max(2) as usize;
            let l = arg_int(a, 2, 1).max(1) as usize;
            let c = arg_int(a, 3, 6) as usize;
            let m = arg_int(a, 4, 3).max(1) as usize;
            calendar_emit(&fmt_text_year($fw, year, w, l, c, m));
            MbValue::none()
        }
    };
}
def_text_pryear!(tpy_0, 0);
def_text_pryear!(tpy_1, 1);
def_text_pryear!(tpy_2, 2);
def_text_pryear!(tpy_3, 3);
def_text_pryear!(tpy_4, 4);
def_text_pryear!(tpy_5, 5);
def_text_pryear!(tpy_6, 6);
const TEXT_PRYEAR: [CalFn; 7] = [
    tpy_0, tpy_1, tpy_2, tpy_3, tpy_4, tpy_5, tpy_6,
];

// HTMLCalendar.formatmonth(theyear, themonth, withyear=True)
macro_rules! def_html_month {
    ($name:ident, $fw:expr) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let all = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            let (a, kw) = split_args(all);
            let year = arg_int(a, 0, 1970);
            let month = arg_int(a, 1, 1);
            let withyear = if a.len() >= 3 {
                a[2].as_bool().unwrap_or(true)
            } else {
                kwarg_bool(&kw, "withyear", true)
            };
            MbValue::from_ptr(MbObject::new_str(html_format_month($fw, year, month, withyear)))
        }
    };
}
def_html_month!(hm_0, 0);
def_html_month!(hm_1, 1);
def_html_month!(hm_2, 2);
def_html_month!(hm_3, 3);
def_html_month!(hm_4, 4);
def_html_month!(hm_5, 5);
def_html_month!(hm_6, 6);
const HTML_FORMATMONTH: [CalFn; 7] = [
    hm_0, hm_1, hm_2, hm_3, hm_4, hm_5, hm_6,
];

// HTMLCalendar.formatyear(theyear, width=3)
macro_rules! def_html_year {
    ($name:ident, $fw:expr) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            let year = arg_int(a, 0, 1970);
            let width = arg_int(a, 1, 3);
            MbValue::from_ptr(MbObject::new_str(html_format_year_fw($fw, year, width)))
        }
    };
}
def_html_year!(hy_0, 0);
def_html_year!(hy_1, 1);
def_html_year!(hy_2, 2);
def_html_year!(hy_3, 3);
def_html_year!(hy_4, 4);
def_html_year!(hy_5, 5);
def_html_year!(hy_6, 6);
const HTML_FORMATYEAR: [CalFn; 7] = [
    hy_0, hy_1, hy_2, hy_3, hy_4, hy_5, hy_6,
];

// HTMLCalendar.formatyearpage(theyear, width=3, css='calendar.css', encoding=None)
macro_rules! def_html_yearpage {
    ($name:ident, $fw:expr) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let all = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            let (a, kw) = split_args(all);
            let year = arg_int(a, 0, 1970);
            let width = arg_int(a, 1, 3);
            // encoding: positional[3] or kwarg, default utf-8 (None → utf-8).
            let encoding = a.get(3).and_then(|v| {
                if v.is_none() { None } else {
                    v.as_ptr().and_then(|p| match &(*p).data {
                        ObjData::Str(ref s) => Some(s.clone()),
                        _ => None,
                    })
                }
            }).or_else(|| kwarg_str(&kw, "encoding")).unwrap_or_else(|| "utf-8".to_string());
            // CPython returns the page encoded to bytes. The page is pure ASCII,
            // so ascii / utf-8 / latin-1 yield identical byte sequences.
            let page = html_format_yearpage($fw, year, width, &encoding);
            MbValue::from_ptr(MbObject::new_bytes(page.into_bytes()))
        }
    };
}
def_html_yearpage!(hyp_0, 0);
def_html_yearpage!(hyp_1, 1);
def_html_yearpage!(hyp_2, 2);
def_html_yearpage!(hyp_3, 3);
def_html_yearpage!(hyp_4, 4);
def_html_yearpage!(hyp_5, 5);
def_html_yearpage!(hyp_6, 6);
const HTML_FORMATYEARPAGE: [CalFn; 7] = [
    hyp_0, hyp_1, hyp_2, hyp_3, hyp_4, hyp_5, hyp_6,
];

// HANDWRITE-END

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::super::rc::ObjData;

    fn tuple_int_at(val: MbValue, idx: usize) -> Option<i64> {
        val.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Tuple(ref items) = (*ptr).data {
                items.get(idx).and_then(|v| v.as_int())
            } else { None }
        })
    }

    fn list_len(val: MbValue) -> usize {
        val.as_ptr().map(|ptr| unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                lock.read().unwrap().len()
            } else { 0 }
        }).unwrap_or(0)
    }

    fn list_str_at(val: MbValue, idx: usize) -> Option<String> {
        val.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                lock.read().unwrap().get(idx).copied().and_then(|v| {
                    v.as_ptr().and_then(|p| {
                        if let ObjData::Str(ref s) = (*p).data { Some(s.clone()) } else { None }
                    })
                })
            } else { None }
        })
    }

    fn list_int_at(val: MbValue, idx: usize) -> Option<i64> {
        val.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                lock.read().unwrap().get(idx).copied().and_then(|v| v.as_int())
            } else { None }
        })
    }

    fn get_field(instance: MbValue, field: &str) -> MbValue {
        if let Some(ptr) = instance.as_ptr() {
            unsafe {
                if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                    let f = fields.read().unwrap();
                    if let Some(v) = f.get(field) { return *v; }
                }
            }
        }
        MbValue::none()
    }

    fn get_str(val: MbValue) -> Option<String> {
        val.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Str(ref s) = (*ptr).data { Some(s.clone()) } else { None }
        })
    }

    // -- isleap truth table --

    #[test]
    fn test_isleap_400() {
        assert_eq!(mb_calendar_isleap(MbValue::from_int(2000)).as_bool(), Some(true));
    }

    #[test]
    fn test_isleap_100() {
        assert_eq!(mb_calendar_isleap(MbValue::from_int(1900)).as_bool(), Some(false));
    }

    #[test]
    fn test_isleap_4() {
        assert_eq!(mb_calendar_isleap(MbValue::from_int(2024)).as_bool(), Some(true));
    }

    #[test]
    fn test_isleap_odd() {
        assert_eq!(mb_calendar_isleap(MbValue::from_int(2023)).as_bool(), Some(false));
    }

    #[test]
    fn test_isleap_2100() {
        // Divisible by 100, not 400 — not a leap year.
        assert_eq!(mb_calendar_isleap(MbValue::from_int(2100)).as_bool(), Some(false));
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
        let r = mb_calendar_weekday(MbValue::from_int(2024),
                                    MbValue::from_int(1),
                                    MbValue::from_int(1));
        assert_eq!(r.as_int(), Some(0));
    }

    #[test]
    fn test_weekday_2000_01_01_saturday() {
        // 2000-01-01 is a Saturday → 5
        let r = mb_calendar_weekday(MbValue::from_int(2000),
                                    MbValue::from_int(1),
                                    MbValue::from_int(1));
        assert_eq!(r.as_int(), Some(5));
    }

    #[test]
    fn test_weekday_1970_01_01_thursday() {
        // 1970-01-01 (Unix epoch) is a Thursday → 3
        let r = mb_calendar_weekday(MbValue::from_int(1970),
                                    MbValue::from_int(1),
                                    MbValue::from_int(1));
        assert_eq!(r.as_int(), Some(3));
    }

    #[test]
    fn test_weekday_2026_05_16_saturday() {
        // 2026-05-16 is a Saturday → 5
        let r = mb_calendar_weekday(MbValue::from_int(2026),
                                    MbValue::from_int(5),
                                    MbValue::from_int(16));
        assert_eq!(r.as_int(), Some(5));
    }

    // -- monthcalendar --

    #[test]
    fn test_monthcalendar_jan_2024_shape() {
        // 2024-01: starts Monday (firstweekday=0 default), 31 days.
        // → 5 weeks, last week padded with trailing zeros.
        FIRST_WEEKDAY.store(0, Ordering::Relaxed);
        let cal = mb_calendar_monthcalendar(MbValue::from_int(2024),
                                            MbValue::from_int(1));
        assert_eq!(list_len(cal), 5);
    }

    #[test]
    fn test_monthcalendar_feb_2024_starts_with_zeros() {
        // 2024-02-01 is Thursday; with firstweekday=0 the first week has
        // three leading zeros [0, 0, 0, 1, 2, 3, 4].
        FIRST_WEEKDAY.store(0, Ordering::Relaxed);
        let cal = mb_calendar_monthcalendar(MbValue::from_int(2024),
                                            MbValue::from_int(2));
        let week0 = cal.as_ptr().map(|ptr| unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                lock.read().unwrap()[0]
            } else { MbValue::none() }
        }).unwrap_or_else(MbValue::none);
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
            MbValue::from_int(1970), MbValue::from_int(1), MbValue::from_int(1),
            MbValue::from_int(0),    MbValue::from_int(0), MbValue::from_int(0),
        ]));
        assert_eq!(mb_calendar_timegm(tup).as_int(), Some(0));
    }

    #[test]
    fn test_timegm_known_date() {
        // (2024, 1, 1, 0, 0, 0) — CPython: calendar.timegm((2024,1,1,0,0,0))
        // == 1704067200
        let tup = MbValue::from_ptr(MbObject::new_tuple(vec![
            MbValue::from_int(2024), MbValue::from_int(1), MbValue::from_int(1),
            MbValue::from_int(0),    MbValue::from_int(0), MbValue::from_int(0),
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
        assert_eq!(get_str(get_field(inst, "locale")).as_deref(), Some("en_US.UTF-8"));
    }

    #[test]
    fn test_make_error_class_carries_name() {
        let e = make_error_class("IllegalMonthError");
        assert_eq!(get_str(get_field(e, "__name__")).as_deref(), Some("IllegalMonthError"));
        assert_eq!(get_str(get_field(e, "__module__")).as_deref(), Some("calendar"));
    }

    #[test]
    fn test_make_enum_class_carries_name() {
        let m = make_enum_class("Month");
        assert_eq!(get_str(get_field(m, "__name__")).as_deref(), Some("Month"));
    }
}
