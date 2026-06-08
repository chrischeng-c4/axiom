# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_uuid_calendar_time_types_value_ops"
# subject = "cpython321.test_uuid_calendar_time_types_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_uuid_calendar_time_types_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_uuid_calendar_time_types_value_ops: execute CPython 3.12 seed test_uuid_calendar_time_types_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the value contract of four
# bootstrap stdlib modules that drive every clock / calendar /
# id / reflection path: `uuid` (the UUID hex-string round-trip,
# the str / hex / int / bytes accessor surface, and the uuid5
# deterministic-namespace contract), `calendar` (the documented
# MONDAY / TUESDAY / WEDNESDAY / SUNDAY weekday-name integer
# sentinels + isleap predicate + month_name / day_name /
# month_abbr / day_abbr lookup tables + weekday / timegm helpers),
# `time` (the strftime / gmtime / time / monotonic / perf_counter
# / process_time / mktime / CLOCK_MONOTONIC surface), and `types`
# (the ModuleType / FunctionType / LambdaType / MethodType /
# BuiltinFunctionType / GeneratorType / NoneType bare class
# identity contract).
#
# The matching subset between mamba and CPython is the
# integer-constant + value-contract + class-identity layer:
# uuid.UUID(hex_string).hex / .int / .bytes type accessors,
# uuid.uuid5(NAMESPACE_DNS, "python.org") returns the
# deterministic IETF-RFC-4122 UUID; calendar weekday integers
# match POSIX (MONDAY=0..SUNDAY=6); calendar.isleap returns the
# Gregorian leap-year boolean; calendar.month_name/day_name/etc.
# carry the documented English names; calendar.weekday /
# timegm return the documented values; time.strftime / gmtime /
# time / monotonic / perf_counter / process_time / mktime all
# return the documented types; types.ModuleType / FunctionType /
# LambdaType / MethodType / BuiltinFunctionType / GeneratorType
# / NoneType all carry their documented `__name__` and
# `isinstance(None, NoneType)` is True.
#
# Surface in this fixture:
#   • uuid.UUID("12345678-1234-5678-1234-567812345678").hex ==
#     "12345678123456781234567812345678";
#   • type(UUID().int) is int, type(UUID().bytes) is bytes;
#   • str(uuid.uuid5(uuid.NAMESPACE_DNS, "python.org")) ==
#     "886313e1-3b8a-5372-9b90-0c9aee199e5d" — deterministic;
#   • calendar.MONDAY == 0, TUESDAY == 1, WEDNESDAY == 2,
#     SUNDAY == 6 — POSIX integer sentinels;
#   • calendar.isleap(2024) is True, (2023) is False,
#     (2000) is True, (1900) is False — Gregorian leap rule;
#   • calendar.month_name[1] == "January",
#     month_name[12] == "December";
#   • calendar.day_name[0] == "Monday", day_name[6] == "Sunday";
#   • calendar.month_abbr[1] == "Jan", day_abbr[0] == "Mon";
#   • calendar.weekday(2024, 1, 1) == 0 — 2024-01-01 was Monday;
#   • calendar.timegm((1970, 1, 1, 0, 0, 0, 0, 0, 0)) == 0;
#   • time.strftime("%Y-%m-%d", time.gmtime(0)) == "1970-01-01";
#   • time.gmtime(0).tm_year == 1970, tm_mon == 1;
#   • type(time.time()) is float;
#   • type(time.monotonic()) / perf_counter / process_time are
#     float;
#   • hasattr(time, "CLOCK_MONOTONIC");
#   • types.ModuleType.__name__ == "module",
#     FunctionType.__name__ == "function",
#     LambdaType.__name__ == "function",
#     MethodType.__name__ == "method",
#     BuiltinFunctionType.__name__ ==
#     "builtin_function_or_method",
#     GeneratorType.__name__ == "generator",
#     NoneType.__name__ == "NoneType";
#   • isinstance(None, types.NoneType) is True.
#
# Behavioral edges that DIVERGE on mamba (uuid.UUID class identity,
# type(uuid.uuid4()) is UUID, type(uuid.uuid1()) is UUID,
# str(uuid.NAMESPACE_DNS / NAMESPACE_URL / NAMESPACE_OID /
# NAMESPACE_X500) as IETF-RFC-4122 UUID strings, time.strptime
# parses `%Y`, time.struct_time class identity, types.
# SimpleNamespace class identity + instance constructor) are
# covered in `lang_uuid_strptime_simplenamespace_silent`.
import uuid
import calendar
import time
import types

_ledger: list[int] = []

# 1) uuid.UUID — hex-string round-trip
_u = uuid.UUID("12345678-1234-5678-1234-567812345678")
assert _u.hex == "12345678123456781234567812345678"; _ledger.append(1)
assert type(_u.int).__name__ == "int"; _ledger.append(1)
assert type(_u.bytes).__name__ == "bytes"; _ledger.append(1)

# 2) uuid.uuid5 — deterministic IETF-RFC-4122 namespace UUID
_u5 = uuid.uuid5(uuid.NAMESPACE_DNS, "python.org")
assert str(_u5) == "886313e1-3b8a-5372-9b90-0c9aee199e5d"; _ledger.append(1)

# 3) calendar — POSIX weekday integer sentinels
assert calendar.MONDAY == 0; _ledger.append(1)
assert calendar.TUESDAY == 1; _ledger.append(1)
assert calendar.WEDNESDAY == 2; _ledger.append(1)
assert calendar.SUNDAY == 6; _ledger.append(1)

# 4) calendar.isleap — Gregorian leap-year rule
assert calendar.isleap(2024) == True; _ledger.append(1)
assert calendar.isleap(2023) == False; _ledger.append(1)
assert calendar.isleap(2000) == True; _ledger.append(1)
assert calendar.isleap(1900) == False; _ledger.append(1)

# 5) calendar.month_name / month_abbr — English month names
assert calendar.month_name[1] == "January"; _ledger.append(1)
assert calendar.month_name[12] == "December"; _ledger.append(1)
assert calendar.month_abbr[1] == "Jan"; _ledger.append(1)

# 6) calendar.day_name / day_abbr — English day names
assert calendar.day_name[0] == "Monday"; _ledger.append(1)
assert calendar.day_name[6] == "Sunday"; _ledger.append(1)
assert calendar.day_abbr[0] == "Mon"; _ledger.append(1)

# 7) calendar.weekday — 2024-01-01 was a Monday
assert calendar.weekday(2024, 1, 1) == 0; _ledger.append(1)

# 8) calendar.timegm — UTC epoch zero
assert calendar.timegm((1970, 1, 1, 0, 0, 0, 0, 0, 0)) == 0; _ledger.append(1)

# 9) time.strftime / gmtime — UNIX epoch zero
assert time.strftime("%Y-%m-%d", time.gmtime(0)) == "1970-01-01"; _ledger.append(1)

# 10) time.gmtime — struct-time accessor surface
_gm0 = time.gmtime(0)
assert _gm0.tm_year == 1970; _ledger.append(1)
assert _gm0.tm_mon == 1; _ledger.append(1)

# 11) time.* clock helpers return float
assert type(time.time()).__name__ == "float"; _ledger.append(1)
assert type(time.monotonic()).__name__ == "float"; _ledger.append(1)
assert type(time.perf_counter()).__name__ == "float"; _ledger.append(1)
assert type(time.process_time()).__name__ == "float"; _ledger.append(1)

# 12) time.CLOCK_MONOTONIC — POSIX clock constant
assert hasattr(time, "CLOCK_MONOTONIC"); _ledger.append(1)

# 13) types — class identity surface
assert types.ModuleType.__name__ == "module"; _ledger.append(1)
assert types.FunctionType.__name__ == "function"; _ledger.append(1)
assert types.LambdaType.__name__ == "function"; _ledger.append(1)
assert types.MethodType.__name__ == "method"; _ledger.append(1)
assert types.BuiltinFunctionType.__name__ == "builtin_function_or_method"; _ledger.append(1)
assert types.GeneratorType.__name__ == "generator"; _ledger.append(1)
assert types.NoneType.__name__ == "NoneType"; _ledger.append(1)

# 14) types.NoneType — instance predicate
assert isinstance(None, types.NoneType) == True; _ledger.append(1)

# 15) hasattr surface — module-level helpers
assert hasattr(uuid, "UUID"); _ledger.append(1)
assert hasattr(uuid, "uuid4"); _ledger.append(1)
assert hasattr(uuid, "uuid5"); _ledger.append(1)
assert hasattr(calendar, "MONDAY"); _ledger.append(1)
assert hasattr(calendar, "isleap"); _ledger.append(1)
assert hasattr(time, "strftime"); _ledger.append(1)
assert hasattr(time, "gmtime"); _ledger.append(1)
assert hasattr(types, "ModuleType"); _ledger.append(1)

# NB: uuid.UUID class identity, type(uuid.uuid4()) is UUID,
# type(uuid.uuid1()) is UUID, str(uuid.NAMESPACE_*) as
# IETF-RFC-4122 UUID strings, time.strptime parses `%Y`,
# time.struct_time class identity, types.SimpleNamespace class
# identity + instance constructor all DIVERGE on mamba — moved
# to the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_uuid_calendar_time_types_value_ops {sum(_ledger)} asserts")
