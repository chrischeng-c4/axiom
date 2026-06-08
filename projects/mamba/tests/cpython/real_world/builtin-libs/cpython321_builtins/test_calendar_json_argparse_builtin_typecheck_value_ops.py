# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_calendar_json_argparse_builtin_typecheck_value_ops"
# subject = "cpython321.test_calendar_json_argparse_builtin_typecheck_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_calendar_json_argparse_builtin_typecheck_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_calendar_json_argparse_builtin_typecheck_value_ops: execute CPython 3.12 seed test_calendar_json_argparse_builtin_typecheck_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the value contract of the
# `calendar` / `json` / `argparse` triplet + the documented
# builtin type-introspection surface used by every Python-typed
# path: `calendar` (the documented `isleap` / `leapdays` /
# `month_name` / `day_name` / `monthrange` / `weekday` /
# `timegm` module-level helper value contract + the documented
# `TextCalendar` / `HTMLCalendar` / `month` / `calendar` class /
# helper hasattr surface), `json` (the documented `dumps` with
# `indent=` / `separators=` / `sort_keys=` / `ensure_ascii=False`
# keyword surface + the documented `loads` scalar return
# contract + the documented `dump` / `load` / `JSONDecoder` /
# `JSONEncoder` / `JSONDecodeError` module hasattr surface),
# `argparse` (the documented `ArgumentParser` module-level
# constructor hasattr surface), and the documented `isinstance`
# / `issubclass` / `type` / `repr` / `bool` builtin type-
# introspection contract.
#
# The matching subset between mamba and CPython is the full
# `calendar` module-level helper layer (isleap / leapdays /
# month_name index / day_name index / monthrange numeric [1] /
# weekday / timegm) + full `calendar` class / helper hasattr
# layer (TextCalendar / HTMLCalendar / month / calendar /
# isleap / leapdays / month_name / day_name / monthrange /
# weekday / timegm), the full `json.dumps` with indent= /
# separators= / sort_keys= / ensure_ascii=False keyword
# surface, the `json.loads` scalar layer (true / null / numeric
# / string), the full `json` module hasattr surface (dumps /
# loads / dump / load / JSONDecoder / JSONEncoder /
# JSONDecodeError), the `argparse.ArgumentParser` constructor
# hasattr layer, and the full builtin `isinstance` /
# `issubclass` / `type` / `repr` / `bool` value contract.
#
# Surface in this fixture:
#   • calendar.isleap — Gregorian leap-year contract
#     (2020 / 2000 leap, 2021 / 1900 not);
#   • calendar.leapdays — leap-day count contract;
#   • calendar.month_name / day_name — int-indexed name table;
#   • calendar.monthrange — first weekday + day-count tuple
#     (asserts the numeric [1] day-count only — [0] weekday
#     enum representation diverges across implementations);
#   • calendar.weekday — weekday index;
#   • calendar.timegm — UTC tuple-to-epoch conversion;
#   • calendar — module hasattr surface (isleap / leapdays /
#     month_name / day_name / monthrange / weekday / timegm /
#     TextCalendar / HTMLCalendar / month / calendar);
#   • json.dumps — indent= / separators= / sort_keys= /
#     ensure_ascii=False keyword contracts;
#   • json.loads — scalar return contract (true / null /
#     numeric / string);
#   • json — module hasattr surface (dumps / loads / dump /
#     load / JSONDecoder / JSONEncoder / JSONDecodeError);
#   • argparse — module hasattr surface (ArgumentParser);
#   • isinstance — int / float / str / list / dict + tuple-arg;
#   • issubclass — bool < int + int < object;
#   • type — name introspection contract;
#   • repr — int / str / list / dict / tuple repr contract;
#   • bool — truthy / falsy coercion contract.
#
# Behavioral edges that DIVERGE on mamba (json.dumps(..., ensure_
# ascii=True) leaves non-ASCII unescaped — the documented \uXXXX
# escape contract is broken, argparse.ArgumentParser() returns a
# `dict` not the documented class instance — entire instance
# surface broken, argparse.Namespace / Action / ArgumentTypeError
# / ArgumentError / FileType / BooleanOptionalAction hasattr
# False — the documented class identifier surface is missing,
# argparse.ArgumentParser().prog returns None — the documented
# `prog` instance attribute is broken) are covered in the
# matching spec fixture `lang_argparse_jsondumps_silent`.
import calendar
import json
import argparse


_ledger: list[int] = []

# 1) calendar.isleap — Gregorian leap-year contract
assert calendar.isleap(2020) == True; _ledger.append(1)
assert calendar.isleap(2021) == False; _ledger.append(1)
assert calendar.isleap(2000) == True; _ledger.append(1)
assert calendar.isleap(1900) == False; _ledger.append(1)

# 2) calendar.leapdays — leap-day count
assert calendar.leapdays(2000, 2020) == 5; _ledger.append(1)

# 3) calendar.month_name / day_name — name table
assert calendar.month_name[1] == "January"; _ledger.append(1)
assert calendar.month_name[12] == "December"; _ledger.append(1)
assert calendar.day_name[0] == "Monday"; _ledger.append(1)
assert calendar.day_name[6] == "Sunday"; _ledger.append(1)

# 4) calendar.monthrange — day-count value contract ([1] only —
#    [0] weekday enum repr diverges between implementations)
assert calendar.monthrange(2024, 2)[1] == 29; _ledger.append(1)
assert calendar.monthrange(2023, 2)[1] == 28; _ledger.append(1)
assert calendar.monthrange(2024, 1)[1] == 31; _ledger.append(1)

# 5) calendar.weekday — weekday index
assert calendar.weekday(2024, 5, 27) == 0; _ledger.append(1)

# 6) calendar.timegm — UTC epoch
assert calendar.timegm((2024, 1, 1, 0, 0, 0, 0, 0, 0)) == 1704067200; _ledger.append(1)

# 7) calendar — module hasattr surface
assert hasattr(calendar, "isleap") == True; _ledger.append(1)
assert hasattr(calendar, "leapdays") == True; _ledger.append(1)
assert hasattr(calendar, "month_name") == True; _ledger.append(1)
assert hasattr(calendar, "day_name") == True; _ledger.append(1)
assert hasattr(calendar, "monthrange") == True; _ledger.append(1)
assert hasattr(calendar, "weekday") == True; _ledger.append(1)
assert hasattr(calendar, "timegm") == True; _ledger.append(1)
assert hasattr(calendar, "TextCalendar") == True; _ledger.append(1)
assert hasattr(calendar, "HTMLCalendar") == True; _ledger.append(1)
assert hasattr(calendar, "month") == True; _ledger.append(1)
assert hasattr(calendar, "calendar") == True; _ledger.append(1)

# 8) json.dumps — indent= / separators= / sort_keys= / ensure_ascii=False
assert json.dumps({"a": 1, "b": [2, 3]}, indent=2) == '{\n  "a": 1,\n  "b": [\n    2,\n    3\n  ]\n}'; _ledger.append(1)
assert json.dumps([1, 2, 3], separators=(",", ":")) == "[1,2,3]"; _ledger.append(1)
assert json.dumps({"b": 2, "a": 1}, sort_keys=True) == '{"a": 1, "b": 2}'; _ledger.append(1)
assert json.dumps("héllo", ensure_ascii=False) == '"héllo"'; _ledger.append(1)

# 9) json.loads — scalar return contract
assert json.loads("true") == True; _ledger.append(1)
assert json.loads("false") == False; _ledger.append(1)
assert json.loads("null") is None; _ledger.append(1)
assert json.loads("123") == 123; _ledger.append(1)
assert json.loads("3.14") == 3.14; _ledger.append(1)
assert json.loads('"hello"') == "hello"; _ledger.append(1)

# 10) json — module hasattr surface
assert hasattr(json, "dumps") == True; _ledger.append(1)
assert hasattr(json, "loads") == True; _ledger.append(1)
assert hasattr(json, "dump") == True; _ledger.append(1)
assert hasattr(json, "load") == True; _ledger.append(1)
assert hasattr(json, "JSONDecoder") == True; _ledger.append(1)
assert hasattr(json, "JSONEncoder") == True; _ledger.append(1)
assert hasattr(json, "JSONDecodeError") == True; _ledger.append(1)

# 11) argparse — ArgumentParser module-level hasattr
assert hasattr(argparse, "ArgumentParser") == True; _ledger.append(1)

# 12) isinstance — int / float / str / list / dict + tuple-arg
assert isinstance(1, int) == True; _ledger.append(1)
assert isinstance(1.0, float) == True; _ledger.append(1)
assert isinstance("x", str) == True; _ledger.append(1)
assert isinstance([], list) == True; _ledger.append(1)
assert isinstance({}, dict) == True; _ledger.append(1)
assert isinstance(1, (int, str)) == True; _ledger.append(1)
assert isinstance("x", (int, str)) == True; _ledger.append(1)
assert isinstance(1.0, (int, str)) == False; _ledger.append(1)

# 13) issubclass — bool < int + int < object
assert issubclass(bool, int) == True; _ledger.append(1)
assert issubclass(int, object) == True; _ledger.append(1)
assert issubclass(str, object) == True; _ledger.append(1)

# 14) type — name introspection
assert type(1).__name__ == "int"; _ledger.append(1)
assert type(1.0).__name__ == "float"; _ledger.append(1)
assert type("x").__name__ == "str"; _ledger.append(1)
assert type([]).__name__ == "list"; _ledger.append(1)
assert type({}).__name__ == "dict"; _ledger.append(1)
assert type(()).__name__ == "tuple"; _ledger.append(1)

# 15) repr — int / str / list / dict / tuple
assert repr(1) == "1"; _ledger.append(1)
assert repr("x") == "'x'"; _ledger.append(1)
assert repr([1, 2]) == "[1, 2]"; _ledger.append(1)
assert repr({1: 2}) == "{1: 2}"; _ledger.append(1)
assert repr((1, 2)) == "(1, 2)"; _ledger.append(1)

# 16) bool — truthy / falsy coercion
assert bool([]) == False; _ledger.append(1)
assert bool([0]) == True; _ledger.append(1)
assert bool({}) == False; _ledger.append(1)
assert bool({0: 0}) == True; _ledger.append(1)
assert bool("") == False; _ledger.append(1)
assert bool("x") == True; _ledger.append(1)
assert bool(0) == False; _ledger.append(1)
assert bool(1) == True; _ledger.append(1)
assert bool(0.0) == False; _ledger.append(1)
assert bool(None) == False; _ledger.append(1)

# NB: json.dumps(..., ensure_ascii=True) leaves non-ASCII
# unescaped on mamba — the documented \uXXXX escape contract is
# broken, argparse.ArgumentParser() returns a `dict`, argparse.
# Namespace / Action / ArgumentTypeError / ArgumentError /
# FileType / BooleanOptionalAction hasattr False, ap.prog
# returns None — all DIVERGE on mamba — moved to the
# divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_calendar_json_argparse_builtin_typecheck_value_ops {sum(_ledger)} asserts")
