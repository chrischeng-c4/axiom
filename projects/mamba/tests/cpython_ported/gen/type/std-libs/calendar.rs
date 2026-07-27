use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/calendar/Calendar__init__firstweekday_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_Calendar__init__firstweekday_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "Calendar__init__firstweekday_as_int_wrong"
# subject = "calendar.Calendar.__init__(firstweekday: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.Calendar.__init__(firstweekday: int); call it with the wrong type.

typeshed contract: firstweekday is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from calendar import Calendar
try:
    Calendar("not_an_int")  # firstweekday: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/Calendar__itermonthdates__year_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_Calendar__itermonthdates__year_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "Calendar__itermonthdates__year_as_int_wrong"
# subject = "calendar.Calendar.itermonthdates(year: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.Calendar.itermonthdates(year: int); call it with the wrong type.

typeshed contract: year is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from calendar import Calendar
obj = object.__new__(Calendar)
try:
    obj.itermonthdates("not_an_int", 0)  # year: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/Calendar__itermonthdays2__year_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_Calendar__itermonthdays2__year_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "Calendar__itermonthdays2__year_as_int_wrong"
# subject = "calendar.Calendar.itermonthdays2(year: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.Calendar.itermonthdays2(year: int); call it with the wrong type.

typeshed contract: year is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from calendar import Calendar
obj = object.__new__(Calendar)
try:
    obj.itermonthdays2("not_an_int", 0)  # year: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/Calendar__itermonthdays3__year_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_Calendar__itermonthdays3__year_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "Calendar__itermonthdays3__year_as_int_wrong"
# subject = "calendar.Calendar.itermonthdays3(year: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.Calendar.itermonthdays3(year: int); call it with the wrong type.

typeshed contract: year is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from calendar import Calendar
obj = object.__new__(Calendar)
try:
    obj.itermonthdays3("not_an_int", 0)  # year: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/Calendar__itermonthdays4__year_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_Calendar__itermonthdays4__year_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "Calendar__itermonthdays4__year_as_int_wrong"
# subject = "calendar.Calendar.itermonthdays4(year: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.Calendar.itermonthdays4(year: int); call it with the wrong type.

typeshed contract: year is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from calendar import Calendar
obj = object.__new__(Calendar)
try:
    obj.itermonthdays4("not_an_int", 0)  # year: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/Calendar__itermonthdays__year_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_Calendar__itermonthdays__year_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "Calendar__itermonthdays__year_as_int_wrong"
# subject = "calendar.Calendar.itermonthdays(year: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.Calendar.itermonthdays(year: int); call it with the wrong type.

typeshed contract: year is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from calendar import Calendar
obj = object.__new__(Calendar)
try:
    obj.itermonthdays("not_an_int", 0)  # year: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/Calendar__monthdatescalendar__year_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_Calendar__monthdatescalendar__year_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "Calendar__monthdatescalendar__year_as_int_wrong"
# subject = "calendar.Calendar.monthdatescalendar(year: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.Calendar.monthdatescalendar(year: int); call it with the wrong type.

typeshed contract: year is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from calendar import Calendar
obj = object.__new__(Calendar)
try:
    obj.monthdatescalendar("not_an_int", 0)  # year: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/Calendar__monthdays2calendar__year_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_Calendar__monthdays2calendar__year_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "Calendar__monthdays2calendar__year_as_int_wrong"
# subject = "calendar.Calendar.monthdays2calendar(year: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.Calendar.monthdays2calendar(year: int); call it with the wrong type.

typeshed contract: year is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from calendar import Calendar
obj = object.__new__(Calendar)
try:
    obj.monthdays2calendar("not_an_int", 0)  # year: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/Calendar__monthdayscalendar__year_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_Calendar__monthdayscalendar__year_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "Calendar__monthdayscalendar__year_as_int_wrong"
# subject = "calendar.Calendar.monthdayscalendar(year: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.Calendar.monthdayscalendar(year: int); call it with the wrong type.

typeshed contract: year is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from calendar import Calendar
obj = object.__new__(Calendar)
try:
    obj.monthdayscalendar("not_an_int", 0)  # year: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/Calendar__setfirstweekday__firstweekday_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_Calendar__setfirstweekday__firstweekday_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "Calendar__setfirstweekday__firstweekday_as_int_wrong"
# subject = "calendar.Calendar.setfirstweekday(firstweekday: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.Calendar.setfirstweekday(firstweekday: int); call it with the wrong type.

typeshed contract: firstweekday is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from calendar import Calendar
obj = object.__new__(Calendar)
try:
    obj.setfirstweekday("not_an_int")  # firstweekday: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/Calendar__yeardatescalendar__year_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_Calendar__yeardatescalendar__year_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "Calendar__yeardatescalendar__year_as_int_wrong"
# subject = "calendar.Calendar.yeardatescalendar(year: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.Calendar.yeardatescalendar(year: int); call it with the wrong type.

typeshed contract: year is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from calendar import Calendar
obj = object.__new__(Calendar)
try:
    obj.yeardatescalendar("not_an_int")  # year: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/Calendar__yeardays2calendar__year_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_Calendar__yeardays2calendar__year_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "Calendar__yeardays2calendar__year_as_int_wrong"
# subject = "calendar.Calendar.yeardays2calendar(year: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.Calendar.yeardays2calendar(year: int); call it with the wrong type.

typeshed contract: year is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from calendar import Calendar
obj = object.__new__(Calendar)
try:
    obj.yeardays2calendar("not_an_int")  # year: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/Calendar__yeardayscalendar__year_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_Calendar__yeardayscalendar__year_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "Calendar__yeardayscalendar__year_as_int_wrong"
# subject = "calendar.Calendar.yeardayscalendar(year: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.Calendar.yeardayscalendar(year: int); call it with the wrong type.

typeshed contract: year is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from calendar import Calendar
obj = object.__new__(Calendar)
try:
    obj.yeardayscalendar("not_an_int")  # year: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/HTMLCalendar__formatday__day_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_HTMLCalendar__formatday__day_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "HTMLCalendar__formatday__day_as_int_wrong"
# subject = "calendar.HTMLCalendar.formatday(day: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.HTMLCalendar.formatday(day: int); call it with the wrong type.

typeshed contract: day is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from calendar import HTMLCalendar
obj = object.__new__(HTMLCalendar)
try:
    obj.formatday("not_an_int", 0)  # day: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/HTMLCalendar__formatmonth__theyear_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_HTMLCalendar__formatmonth__theyear_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "HTMLCalendar__formatmonth__theyear_as_int_wrong"
# subject = "calendar.HTMLCalendar.formatmonth(theyear: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.HTMLCalendar.formatmonth(theyear: int); call it with the wrong type.

typeshed contract: theyear is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from calendar import HTMLCalendar
obj = object.__new__(HTMLCalendar)
try:
    obj.formatmonth("not_an_int", 0)  # theyear: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/HTMLCalendar__formatmonthname__theyear_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_HTMLCalendar__formatmonthname__theyear_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "HTMLCalendar__formatmonthname__theyear_as_int_wrong"
# subject = "calendar.HTMLCalendar.formatmonthname(theyear: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.HTMLCalendar.formatmonthname(theyear: int); call it with the wrong type.

typeshed contract: theyear is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from calendar import HTMLCalendar
obj = object.__new__(HTMLCalendar)
try:
    obj.formatmonthname("not_an_int", 0)  # theyear: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/HTMLCalendar__formatmonthpage__theyear_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_HTMLCalendar__formatmonthpage__theyear_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "HTMLCalendar__formatmonthpage__theyear_as_int_wrong"
# subject = "calendar.HTMLCalendar.formatmonthpage(theyear: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.HTMLCalendar.formatmonthpage(theyear: int); call it with the wrong type.

typeshed contract: theyear is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from calendar import HTMLCalendar
obj = object.__new__(HTMLCalendar)
try:
    obj.formatmonthpage("not_an_int", 0)  # theyear: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/HTMLCalendar__formatweek__theweek_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_HTMLCalendar__formatweek__theweek_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "HTMLCalendar__formatweek__theweek_as_int_wrong"
# subject = "calendar.HTMLCalendar.formatweek(theweek: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.HTMLCalendar.formatweek(theweek: int); call it with the wrong type.

typeshed contract: theweek is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from calendar import HTMLCalendar
obj = object.__new__(HTMLCalendar)
try:
    obj.formatweek("not_an_int")  # theweek: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/HTMLCalendar__formatweekday__day_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_HTMLCalendar__formatweekday__day_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "HTMLCalendar__formatweekday__day_as_int_wrong"
# subject = "calendar.HTMLCalendar.formatweekday(day: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.HTMLCalendar.formatweekday(day: int); call it with the wrong type.

typeshed contract: day is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from calendar import HTMLCalendar
obj = object.__new__(HTMLCalendar)
try:
    obj.formatweekday("not_an_int")  # day: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/HTMLCalendar__formatyear__theyear_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_HTMLCalendar__formatyear__theyear_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "HTMLCalendar__formatyear__theyear_as_int_wrong"
# subject = "calendar.HTMLCalendar.formatyear(theyear: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.HTMLCalendar.formatyear(theyear: int); call it with the wrong type.

typeshed contract: theyear is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from calendar import HTMLCalendar
obj = object.__new__(HTMLCalendar)
try:
    obj.formatyear("not_an_int")  # theyear: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/HTMLCalendar__formatyearpage__theyear_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_HTMLCalendar__formatyearpage__theyear_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "HTMLCalendar__formatyearpage__theyear_as_int_wrong"
# subject = "calendar.HTMLCalendar.formatyearpage(theyear: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.HTMLCalendar.formatyearpage(theyear: int); call it with the wrong type.

typeshed contract: theyear is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from calendar import HTMLCalendar
obj = object.__new__(HTMLCalendar)
try:
    obj.formatyearpage("not_an_int")  # theyear: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/IllegalMonthError__init__month_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_IllegalMonthError__init__month_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "IllegalMonthError__init__month_as_int_wrong"
# subject = "calendar.IllegalMonthError.__init__(month: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.IllegalMonthError.__init__(month: int); call it with the wrong type.

typeshed contract: month is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from calendar import IllegalMonthError
try:
    IllegalMonthError("not_an_int")  # month: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/IllegalWeekdayError__init__weekday_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_IllegalWeekdayError__init__weekday_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "IllegalWeekdayError__init__weekday_as_int_wrong"
# subject = "calendar.IllegalWeekdayError.__init__(weekday: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.IllegalWeekdayError.__init__(weekday: int); call it with the wrong type.

typeshed contract: weekday is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from calendar import IllegalWeekdayError
try:
    IllegalWeekdayError("not_an_int")  # weekday: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/LocaleHTMLCalendar__formatmonthname__theyear_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_LocaleHTMLCalendar__formatmonthname__theyear_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "LocaleHTMLCalendar__formatmonthname__theyear_as_int_wrong"
# subject = "calendar.LocaleHTMLCalendar.formatmonthname(theyear: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.LocaleHTMLCalendar.formatmonthname(theyear: int); call it with the wrong type.

typeshed contract: theyear is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from calendar import LocaleHTMLCalendar
obj = object.__new__(LocaleHTMLCalendar)
try:
    obj.formatmonthname("not_an_int", 0)  # theyear: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/LocaleHTMLCalendar__formatweekday__day_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_LocaleHTMLCalendar__formatweekday__day_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "LocaleHTMLCalendar__formatweekday__day_as_int_wrong"
# subject = "calendar.LocaleHTMLCalendar.formatweekday(day: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.LocaleHTMLCalendar.formatweekday(day: int); call it with the wrong type.

typeshed contract: day is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from calendar import LocaleHTMLCalendar
obj = object.__new__(LocaleHTMLCalendar)
try:
    obj.formatweekday("not_an_int")  # day: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/LocaleHTMLCalendar__init__firstweekday_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_LocaleHTMLCalendar__init__firstweekday_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "LocaleHTMLCalendar__init__firstweekday_as_int_wrong"
# subject = "calendar.LocaleHTMLCalendar.__init__(firstweekday: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.LocaleHTMLCalendar.__init__(firstweekday: int); call it with the wrong type.

typeshed contract: firstweekday is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from calendar import LocaleHTMLCalendar
try:
    LocaleHTMLCalendar("not_an_int")  # firstweekday: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/LocaleTextCalendar__init__firstweekday_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_LocaleTextCalendar__init__firstweekday_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "LocaleTextCalendar__init__firstweekday_as_int_wrong"
# subject = "calendar.LocaleTextCalendar.__init__(firstweekday: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.LocaleTextCalendar.__init__(firstweekday: int); call it with the wrong type.

typeshed contract: firstweekday is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from calendar import LocaleTextCalendar
try:
    LocaleTextCalendar("not_an_int")  # firstweekday: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/TextCalendar__formatday__day_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_TextCalendar__formatday__day_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "TextCalendar__formatday__day_as_int_wrong"
# subject = "calendar.TextCalendar.formatday(day: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.TextCalendar.formatday(day: int); call it with the wrong type.

typeshed contract: day is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from calendar import TextCalendar
obj = object.__new__(TextCalendar)
try:
    obj.formatday("not_an_int", 0, 0)  # day: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/TextCalendar__formatmonth__theyear_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_TextCalendar__formatmonth__theyear_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "TextCalendar__formatmonth__theyear_as_int_wrong"
# subject = "calendar.TextCalendar.formatmonth(theyear: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.TextCalendar.formatmonth(theyear: int); call it with the wrong type.

typeshed contract: theyear is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from calendar import TextCalendar
obj = object.__new__(TextCalendar)
try:
    obj.formatmonth("not_an_int", 0)  # theyear: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/TextCalendar__formatmonthname__theyear_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_TextCalendar__formatmonthname__theyear_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "TextCalendar__formatmonthname__theyear_as_int_wrong"
# subject = "calendar.TextCalendar.formatmonthname(theyear: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.TextCalendar.formatmonthname(theyear: int); call it with the wrong type.

typeshed contract: theyear is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from calendar import TextCalendar
obj = object.__new__(TextCalendar)
try:
    obj.formatmonthname("not_an_int", 0, 0)  # theyear: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/TextCalendar__formatweek__theweek_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_TextCalendar__formatweek__theweek_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "TextCalendar__formatweek__theweek_as_Iterable_wrong"
# subject = "calendar.TextCalendar.formatweek(theweek: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.TextCalendar.formatweek(theweek: Iterable); call it with the wrong type.

typeshed contract: theweek is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from calendar import TextCalendar
obj = object.__new__(TextCalendar)
try:
    obj.formatweek(_W(), 0)  # theweek: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/TextCalendar__formatweekday__day_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_TextCalendar__formatweekday__day_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "TextCalendar__formatweekday__day_as_int_wrong"
# subject = "calendar.TextCalendar.formatweekday(day: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.TextCalendar.formatweekday(day: int); call it with the wrong type.

typeshed contract: day is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from calendar import TextCalendar
obj = object.__new__(TextCalendar)
try:
    obj.formatweekday("not_an_int", 0)  # day: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/TextCalendar__formatweekheader__width_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_TextCalendar__formatweekheader__width_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "TextCalendar__formatweekheader__width_as_int_wrong"
# subject = "calendar.TextCalendar.formatweekheader(width: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.TextCalendar.formatweekheader(width: int); call it with the wrong type.

typeshed contract: width is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from calendar import TextCalendar
obj = object.__new__(TextCalendar)
try:
    obj.formatweekheader("not_an_int")  # width: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/TextCalendar__formatyear__theyear_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_TextCalendar__formatyear__theyear_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "TextCalendar__formatyear__theyear_as_int_wrong"
# subject = "calendar.TextCalendar.formatyear(theyear: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.TextCalendar.formatyear(theyear: int); call it with the wrong type.

typeshed contract: theyear is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from calendar import TextCalendar
obj = object.__new__(TextCalendar)
try:
    obj.formatyear("not_an_int")  # theyear: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/TextCalendar__prmonth__theyear_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_TextCalendar__prmonth__theyear_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "TextCalendar__prmonth__theyear_as_int_wrong"
# subject = "calendar.TextCalendar.prmonth(theyear: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.TextCalendar.prmonth(theyear: int); call it with the wrong type.

typeshed contract: theyear is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from calendar import TextCalendar
obj = object.__new__(TextCalendar)
try:
    obj.prmonth("not_an_int", 0)  # theyear: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/TextCalendar__prweek__theweek_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_TextCalendar__prweek__theweek_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "TextCalendar__prweek__theweek_as_Iterable_wrong"
# subject = "calendar.TextCalendar.prweek(theweek: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.TextCalendar.prweek(theweek: Iterable); call it with the wrong type.

typeshed contract: theweek is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from calendar import TextCalendar
obj = object.__new__(TextCalendar)
try:
    obj.prweek(_W(), 0)  # theweek: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/TextCalendar__pryear__theyear_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_TextCalendar__pryear__theyear_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "TextCalendar__pryear__theyear_as_int_wrong"
# subject = "calendar.TextCalendar.pryear(theyear: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.TextCalendar.pryear(theyear: int); call it with the wrong type.

typeshed contract: theyear is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from calendar import TextCalendar
obj = object.__new__(TextCalendar)
try:
    obj.pryear("not_an_int")  # theyear: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/calendar__theyear_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_calendar__theyear_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "calendar__theyear_as_int_wrong"
# subject = "calendar.calendar(theyear: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.calendar(theyear: int); call it with the wrong type.

typeshed contract: theyear is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from calendar import calendar
try:
    calendar("not_an_int")  # theyear: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/different_locale__init__locale_as__LocaleType_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_different_locale__init__locale_as__LocaleType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "different_locale__init__locale_as__LocaleType_wrong"
# subject = "calendar.different_locale.__init__(locale: _LocaleType)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.different_locale.__init__(locale: _LocaleType); call it with the wrong type.

typeshed contract: locale is _LocaleType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from calendar import different_locale
try:
    different_locale(_W())  # locale: _LocaleType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/format__cols_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_format__cols_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "format__cols_as_int_wrong"
# subject = "calendar.format(cols: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.format(cols: int); call it with the wrong type.

typeshed contract: cols is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from calendar import format
try:
    format("not_an_int")  # cols: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/formatstring__cols_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_formatstring__cols_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "formatstring__cols_as_Iterable_wrong"
# subject = "calendar.formatstring(cols: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.formatstring(cols: Iterable); call it with the wrong type.

typeshed contract: cols is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from calendar import formatstring
try:
    formatstring(_W())  # cols: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/isleap__year_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_isleap__year_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "isleap__year_as_int_wrong"
# subject = "calendar.isleap(year: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.isleap(year: int); call it with the wrong type.

typeshed contract: year is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from calendar import isleap
try:
    isleap("not_an_int")  # year: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/leapdays__y1_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_leapdays__y1_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "leapdays__y1_as_int_wrong"
# subject = "calendar.leapdays(y1: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.leapdays(y1: int); call it with the wrong type.

typeshed contract: y1 is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from calendar import leapdays
try:
    leapdays("not_an_int", 0)  # y1: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/month__theyear_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_month__theyear_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "month__theyear_as_int_wrong"
# subject = "calendar.month(theyear: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.month(theyear: int); call it with the wrong type.

typeshed contract: theyear is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from calendar import month
try:
    month("not_an_int", 0)  # theyear: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/monthcalendar__year_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_monthcalendar__year_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "monthcalendar__year_as_int_wrong"
# subject = "calendar.monthcalendar(year: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.monthcalendar(year: int); call it with the wrong type.

typeshed contract: year is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from calendar import monthcalendar
try:
    monthcalendar("not_an_int", 0)  # year: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/monthrange__year_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_monthrange__year_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "monthrange__year_as_int_wrong"
# subject = "calendar.monthrange(year: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.monthrange(year: int); call it with the wrong type.

typeshed contract: year is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from calendar import monthrange
try:
    monthrange("not_an_int", 0)  # year: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/prcal__theyear_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_prcal__theyear_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "prcal__theyear_as_int_wrong"
# subject = "calendar.prcal(theyear: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.prcal(theyear: int); call it with the wrong type.

typeshed contract: theyear is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from calendar import prcal
try:
    prcal("not_an_int")  # theyear: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/prmonth__theyear_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_prmonth__theyear_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "prmonth__theyear_as_int_wrong"
# subject = "calendar.prmonth(theyear: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.prmonth(theyear: int); call it with the wrong type.

typeshed contract: theyear is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from calendar import prmonth
try:
    prmonth("not_an_int", 0)  # theyear: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/prweek__theweek_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_prweek__theweek_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "prweek__theweek_as_int_wrong"
# subject = "calendar.prweek(theweek: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.prweek(theweek: int); call it with the wrong type.

typeshed contract: theweek is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from calendar import prweek
try:
    prweek("not_an_int", 0)  # theweek: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/setfirstweekday__firstweekday_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_setfirstweekday__firstweekday_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "setfirstweekday__firstweekday_as_int_wrong"
# subject = "calendar.setfirstweekday(firstweekday: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.setfirstweekday(firstweekday: int); call it with the wrong type.

typeshed contract: firstweekday is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from calendar import setfirstweekday
try:
    setfirstweekday("not_an_int")  # firstweekday: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/timegm__tuple_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_timegm__tuple_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "timegm__tuple_as_typed_wrong"
# subject = "calendar.timegm(tuple: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.timegm(tuple: typed); call it with the wrong type.

typeshed contract: tuple is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from calendar import timegm
try:
    timegm(_W())  # tuple: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/week__theweek_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_week__theweek_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "week__theweek_as_int_wrong"
# subject = "calendar.week(theweek: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.week(theweek: int); call it with the wrong type.

typeshed contract: theweek is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from calendar import week
try:
    week("not_an_int", 0)  # theweek: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/weekday__year_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_weekday__year_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "weekday__year_as_int_wrong"
# subject = "calendar.weekday(year: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.weekday(year: int); call it with the wrong type.

typeshed contract: year is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from calendar import weekday
try:
    weekday("not_an_int", 0, 0)  # year: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/calendar/weekheader__width_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_calendar_weekheader__width_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "weekheader__width_as_int_wrong"
# subject = "calendar.weekheader(width: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.weekheader(width: int); call it with the wrong type.

typeshed contract: width is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from calendar import weekheader
try:
    weekheader("not_an_int")  # width: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
