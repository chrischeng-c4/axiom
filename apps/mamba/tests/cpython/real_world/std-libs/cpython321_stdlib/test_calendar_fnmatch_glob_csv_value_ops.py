# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_calendar_fnmatch_glob_csv_value_ops"
# subject = "cpython321.test_calendar_fnmatch_glob_csv_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_calendar_fnmatch_glob_csv_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_calendar_fnmatch_glob_csv_value_ops: execute CPython 3.12 seed test_calendar_fnmatch_glob_csv_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the value contract of the
# `calendar` / `fnmatch` / `glob` / `csv` / `configparser`
# five-pack pinned to atomic 207: `calendar` (the documented
# full module-level class / helper / constant / sentinel /
# exception identifier hasattr surface — `Calendar` /
# `TextCalendar` / `HTMLCalendar` / `LocaleTextCalendar` /
# `LocaleHTMLCalendar` / `month` / `monthrange` / `isleap` /
# `leapdays` / `weekday` / `weekheader` / `calendar` /
# `month_name` / `month_abbr` / `day_name` / `day_abbr` /
# `MONDAY` / `TUESDAY` / `WEDNESDAY` / `THURSDAY` /
# `FRIDAY` / `SATURDAY` / `SUNDAY` / `JANUARY` / `FEBRUARY`
# / `MARCH` / `APRIL` / `MAY` / `JUNE` / `JULY` / `AUGUST`
# / `SEPTEMBER` / `OCTOBER` / `NOVEMBER` / `DECEMBER` /
# `firstweekday` / `setfirstweekday` / `timegm` /
# `IllegalMonthError` / `IllegalWeekdayError` + the
# documented `isleap(2024) == True` / `isleap(2023) ==
# False` / `leapdays(2020, 2025) == 2` / `weekday(2024, 1,
# 1) == 0` / `MONDAY == 0` / `SUNDAY == 6` /
# `monthrange(2024, 2) == (3, 29)` / `type(monthrange(...))
# .__name__ == "tuple"` value contract), `fnmatch` (the
# documented full module-level helper identifier hasattr
# surface — `fnmatch` / `fnmatchcase` / `filter` /
# `translate` + the documented
# `fnmatch.fnmatch("a.txt", "*.txt") == True` /
# `fnmatch.fnmatch("a.txt", "*.py") == False` /
# `fnmatch.fnmatchcase("A.TXT", "*.txt") == False` /
# `fnmatch.filter(["a.txt", "b.py"], "*.txt") ==
# ["a.txt"]` / `type(fnmatch.translate("*.txt"))
# .__name__ == "str"` glob-match / case-sensitive /
# filter-list / translation-string value contract),
# `glob` (the documented full module-level helper
# identifier hasattr surface — `glob` / `iglob` /
# `escape` / `has_magic` + the documented
# `type(glob.glob(...)).__name__ == "list"` /
# `glob.escape("?test") == "[?]test"` list-return /
# bracket-escape value contract), `csv` (the
# documented partial module-level reader / writer /
# dialect / constant / helper / exception identifier
# hasattr surface — `reader` / `writer` / `DictReader`
# / `DictWriter` / `Dialect` / `excel` / `excel_tab` /
# `unix_dialect` / `QUOTE_ALL` / `QUOTE_MINIMAL` /
# `QUOTE_NONE` / `QUOTE_NONNUMERIC` /
# `field_size_limit` / `get_dialect` / `list_dialects`
# / `register_dialect` / `unregister_dialect` /
# `Error` + the documented `QUOTE_ALL == 1` /
# `QUOTE_MINIMAL == 0` / `QUOTE_NONE == 3` /
# `QUOTE_NONNUMERIC == 2` /
# `type(csv.QUOTE_ALL).__name__ == "int"`
# integer-constant value contract), and
# `configparser` (the documented partial module-level
# class identifier hasattr surface — `ConfigParser`).
#
# Behavioral edges that DIVERGE on mamba
# (hasattr(csv, "Sniffer") False on mamba +
# type(csv.writer(io.StringIO())).__name__ == "writer"
# collapses to "str" on mamba, hasattr(configparser,
# "RawConfigParser") / "BasicInterpolation" /
# "ExtendedInterpolation" / "Interpolation" /
# "InterpolationError" / "InterpolationDepthError" /
# "InterpolationMissingOptionError" /
# "InterpolationSyntaxError" / "NoSectionError" /
# "NoOptionError" / "DuplicateSectionError" /
# "DuplicateOptionError" / "MissingSectionHeaderError"
# / "ParsingError" / "Error" / "DEFAULTSECT" /
# "MAX_INTERPOLATION_DEPTH" all False on mamba +
# configparser.DEFAULTSECT == "DEFAULT" collapses to
# None on mamba + configparser.ConfigParser()
# instance method `sections` not present on mamba)
# are covered in the matching spec fixture
# `lang_csv_configparser_silent`.
import calendar
import fnmatch
import glob
import csv
import configparser


_ledger: list[int] = []

# 1) calendar — full module hasattr surface
assert hasattr(calendar, "Calendar") == True; _ledger.append(1)
assert hasattr(calendar, "TextCalendar") == True; _ledger.append(1)
assert hasattr(calendar, "HTMLCalendar") == True; _ledger.append(1)
assert hasattr(calendar, "LocaleTextCalendar") == True; _ledger.append(1)
assert hasattr(calendar, "LocaleHTMLCalendar") == True; _ledger.append(1)
assert hasattr(calendar, "month") == True; _ledger.append(1)
assert hasattr(calendar, "monthrange") == True; _ledger.append(1)
assert hasattr(calendar, "isleap") == True; _ledger.append(1)
assert hasattr(calendar, "leapdays") == True; _ledger.append(1)
assert hasattr(calendar, "weekday") == True; _ledger.append(1)
assert hasattr(calendar, "weekheader") == True; _ledger.append(1)
assert hasattr(calendar, "calendar") == True; _ledger.append(1)
assert hasattr(calendar, "month_name") == True; _ledger.append(1)
assert hasattr(calendar, "month_abbr") == True; _ledger.append(1)
assert hasattr(calendar, "day_name") == True; _ledger.append(1)
assert hasattr(calendar, "day_abbr") == True; _ledger.append(1)
assert hasattr(calendar, "MONDAY") == True; _ledger.append(1)
assert hasattr(calendar, "TUESDAY") == True; _ledger.append(1)
assert hasattr(calendar, "WEDNESDAY") == True; _ledger.append(1)
assert hasattr(calendar, "THURSDAY") == True; _ledger.append(1)
assert hasattr(calendar, "FRIDAY") == True; _ledger.append(1)
assert hasattr(calendar, "SATURDAY") == True; _ledger.append(1)
assert hasattr(calendar, "SUNDAY") == True; _ledger.append(1)
assert hasattr(calendar, "JANUARY") == True; _ledger.append(1)
assert hasattr(calendar, "FEBRUARY") == True; _ledger.append(1)
assert hasattr(calendar, "MARCH") == True; _ledger.append(1)
assert hasattr(calendar, "APRIL") == True; _ledger.append(1)
assert hasattr(calendar, "MAY") == True; _ledger.append(1)
assert hasattr(calendar, "JUNE") == True; _ledger.append(1)
assert hasattr(calendar, "JULY") == True; _ledger.append(1)
assert hasattr(calendar, "AUGUST") == True; _ledger.append(1)
assert hasattr(calendar, "SEPTEMBER") == True; _ledger.append(1)
assert hasattr(calendar, "OCTOBER") == True; _ledger.append(1)
assert hasattr(calendar, "NOVEMBER") == True; _ledger.append(1)
assert hasattr(calendar, "DECEMBER") == True; _ledger.append(1)
assert hasattr(calendar, "firstweekday") == True; _ledger.append(1)
assert hasattr(calendar, "setfirstweekday") == True; _ledger.append(1)
assert hasattr(calendar, "timegm") == True; _ledger.append(1)
assert hasattr(calendar, "IllegalMonthError") == True; _ledger.append(1)
assert hasattr(calendar, "IllegalWeekdayError") == True; _ledger.append(1)

# 2) calendar — leap-year / weekday / monthrange value contract
assert calendar.isleap(2024) == True; _ledger.append(1)
assert calendar.isleap(2023) == False; _ledger.append(1)
assert calendar.leapdays(2020, 2025) == 2; _ledger.append(1)
assert calendar.weekday(2024, 1, 1) == 0; _ledger.append(1)
assert calendar.MONDAY == 0; _ledger.append(1)
assert calendar.SUNDAY == 6; _ledger.append(1)
_mr = calendar.monthrange(2024, 2)
assert _mr == (3, 29); _ledger.append(1)
assert type(_mr).__name__ == "tuple"; _ledger.append(1)

# 3) fnmatch — full module hasattr surface
assert hasattr(fnmatch, "fnmatch") == True; _ledger.append(1)
assert hasattr(fnmatch, "fnmatchcase") == True; _ledger.append(1)
assert hasattr(fnmatch, "filter") == True; _ledger.append(1)
assert hasattr(fnmatch, "translate") == True; _ledger.append(1)

# 4) fnmatch — glob-match / case-sensitive / filter-list / translation value contract
assert fnmatch.fnmatch("a.txt", "*.txt") == True; _ledger.append(1)
assert fnmatch.fnmatch("a.txt", "*.py") == False; _ledger.append(1)
assert fnmatch.fnmatchcase("A.TXT", "*.txt") == False; _ledger.append(1)
assert fnmatch.filter(["a.txt", "b.py"], "*.txt") == ["a.txt"]; _ledger.append(1)
assert type(fnmatch.translate("*.txt")).__name__ == "str"; _ledger.append(1)

# 5) glob — full module hasattr surface
assert hasattr(glob, "glob") == True; _ledger.append(1)
assert hasattr(glob, "iglob") == True; _ledger.append(1)
assert hasattr(glob, "escape") == True; _ledger.append(1)
assert hasattr(glob, "has_magic") == True; _ledger.append(1)

# 6) glob — list-return / bracket-escape value contract
assert type(glob.glob("/etc/h*sts")).__name__ == "list"; _ledger.append(1)
assert glob.escape("?test") == "[?]test"; _ledger.append(1)

# 7) csv — partial module hasattr surface
#    (Sniffer DIVERGES on mamba — moved to spec)
assert hasattr(csv, "reader") == True; _ledger.append(1)
assert hasattr(csv, "writer") == True; _ledger.append(1)
assert hasattr(csv, "DictReader") == True; _ledger.append(1)
assert hasattr(csv, "DictWriter") == True; _ledger.append(1)
assert hasattr(csv, "Dialect") == True; _ledger.append(1)
assert hasattr(csv, "excel") == True; _ledger.append(1)
assert hasattr(csv, "excel_tab") == True; _ledger.append(1)
assert hasattr(csv, "unix_dialect") == True; _ledger.append(1)
assert hasattr(csv, "QUOTE_ALL") == True; _ledger.append(1)
assert hasattr(csv, "QUOTE_MINIMAL") == True; _ledger.append(1)
assert hasattr(csv, "QUOTE_NONE") == True; _ledger.append(1)
assert hasattr(csv, "QUOTE_NONNUMERIC") == True; _ledger.append(1)
assert hasattr(csv, "field_size_limit") == True; _ledger.append(1)
assert hasattr(csv, "get_dialect") == True; _ledger.append(1)
assert hasattr(csv, "list_dialects") == True; _ledger.append(1)
assert hasattr(csv, "register_dialect") == True; _ledger.append(1)
assert hasattr(csv, "unregister_dialect") == True; _ledger.append(1)
assert hasattr(csv, "Error") == True; _ledger.append(1)

# 8) csv — integer-constant value contract
assert csv.QUOTE_ALL == 1; _ledger.append(1)
assert csv.QUOTE_MINIMAL == 0; _ledger.append(1)
assert csv.QUOTE_NONE == 3; _ledger.append(1)
assert csv.QUOTE_NONNUMERIC == 2; _ledger.append(1)
assert type(csv.QUOTE_ALL).__name__ == "int"; _ledger.append(1)

# 9) configparser — partial module hasattr surface
#    (RawConfigParser / BasicInterpolation /
#    ExtendedInterpolation / Interpolation /
#    InterpolationError / InterpolationDepthError /
#    InterpolationMissingOptionError /
#    InterpolationSyntaxError / NoSectionError /
#    NoOptionError / DuplicateSectionError /
#    DuplicateOptionError / MissingSectionHeaderError /
#    ParsingError / Error / DEFAULTSECT /
#    MAX_INTERPOLATION_DEPTH all DIVERGE on mamba —
#    moved to spec)
assert hasattr(configparser, "ConfigParser") == True; _ledger.append(1)

# NB: hasattr(csv, "Sniffer") False on mamba +
# type(csv.writer(io.StringIO())).__name__ == "writer"
# collapses to "str" on mamba, hasattr(configparser,
# "RawConfigParser") / "BasicInterpolation" /
# "ExtendedInterpolation" / "Interpolation" /
# "InterpolationError" / "InterpolationDepthError" /
# "InterpolationMissingOptionError" /
# "InterpolationSyntaxError" / "NoSectionError" /
# "NoOptionError" / "DuplicateSectionError" /
# "DuplicateOptionError" / "MissingSectionHeaderError"
# / "ParsingError" / "Error" / "DEFAULTSECT" /
# "MAX_INTERPOLATION_DEPTH" all False on mamba +
# configparser.DEFAULTSECT == "DEFAULT" collapses to
# None on mamba + configparser.ConfigParser()
# instance method `sections` not present on mamba —
# all DIVERGE on mamba — moved to the divergence-spec
# fixture.

print(f"MAMBA_ASSERTION_PASS: test_calendar_fnmatch_glob_csv_value_ops {sum(_ledger)} asserts")
