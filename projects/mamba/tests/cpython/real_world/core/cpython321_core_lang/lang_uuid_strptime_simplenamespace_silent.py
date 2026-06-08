# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_uuid_strptime_simplenamespace_silent"
# subject = "cpython321.lang_uuid_strptime_simplenamespace_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_uuid_strptime_simplenamespace_silent.py"
# status = "filled"
# ///
"""cpython321.lang_uuid_strptime_simplenamespace_silent: execute CPython 3.12 seed lang_uuid_strptime_simplenamespace_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass seed for SILENT divergences across the
# uuid / time / types quartet pinned by atomic 146: `uuid` (the
# UUID bare class identity, the documented `type(uuid.uuid4())
# is UUID` / `type(uuid.uuid1()) is UUID` instance-class
# contract, and the documented IETF-RFC-4122 string form of the
# NAMESPACE_DNS / NAMESPACE_URL / NAMESPACE_OID / NAMESPACE_X500
# UUID constants — mamba lowers these to int handles), `time`
# (the strptime `%Y` year-parsing contract and the struct_time
# bare class identity), and `types` (the SimpleNamespace bare
# class identity and the documented kwargs-constructor + dot-
# accessor surface).
#
# The matching subset (uuid.UUID hex-string round-trip + .hex /
# .int / .bytes accessors, uuid.uuid5 deterministic-namespace
# output, calendar MONDAY..SUNDAY integer sentinels + isleap +
# month_name / day_name / month_abbr / day_abbr + weekday +
# timegm, time.strftime / gmtime / time / monotonic /
# perf_counter / process_time / mktime / CLOCK_MONOTONIC,
# types.ModuleType / FunctionType / LambdaType / MethodType /
# BuiltinFunctionType / GeneratorType / NoneType class identity
# + isinstance(None, NoneType)) is covered by
# `test_uuid_calendar_time_types_value_ops`; this fixture pins
# the CPython-only contracts that mamba currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • uuid.UUID.__name__ == "UUID" — bare class identity (mamba:
#     returns None);
#   • type(uuid.uuid4()).__name__ == "UUID" — v4 instance-class
#     contract (mamba: returns "int" — mamba lowers UUID to an
#     integer handle);
#   • type(uuid.uuid1()).__name__ == "UUID" — v1 instance-class
#     contract (mamba: returns "int");
#   • str(uuid.NAMESPACE_DNS) ==
#     "6ba7b810-9dad-11d1-80b4-00c04fd430c8" — RFC-4122 DNS
#     namespace (mamba: returns "2199023255552" — the integer
#     handle, not the UUID string);
#   • str(uuid.NAMESPACE_URL) ==
#     "6ba7b811-9dad-11d1-80b4-00c04fd430c8" (mamba:
#     "2199023255553");
#   • str(uuid.NAMESPACE_OID) ==
#     "6ba7b812-9dad-11d1-80b4-00c04fd430c8" (mamba:
#     "2199023255554");
#   • str(uuid.NAMESPACE_X500) ==
#     "6ba7b814-9dad-11d1-80b4-00c04fd430c8" (mamba:
#     "2199023255555");
#   • time.strptime("2024-01-15", "%Y-%m-%d").tm_year == 2024 —
#     parses the year token (mamba: returns 1900 — strptime
#     silently falls back to the epoch year);
#   • time.struct_time.__name__ == "struct_time" — bare class
#     identity (mamba: returns None);
#   • types.SimpleNamespace.__name__ == "SimpleNamespace" — bare
#     class identity (mamba: returns None);
#   • types.SimpleNamespace(x=1, y=2).x == 1 — kwargs constructor
#     + dot accessor (mamba: AttributeError, types is a `dict`
#     that has no SimpleNamespace attribute).
import uuid as _uuid_mod
import time as _time_mod
import types as _types_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# class identifiers / instance constructors / format-string
# helpers that mamba's bundled type stubs do not surface
# accurately.
uuid: Any = _uuid_mod
time: Any = _time_mod
types: Any = _types_mod

_ledger: list[int] = []

# 1) uuid.UUID — bare class identity
assert uuid.UUID.__name__ == "UUID"; _ledger.append(1)

# 2) uuid.uuid4 / uuid.uuid1 — instance-class contract
assert type(uuid.uuid4()).__name__ == "UUID"; _ledger.append(1)
assert type(uuid.uuid1()).__name__ == "UUID"; _ledger.append(1)

# 3) uuid.NAMESPACE_* — RFC-4122 namespace UUID strings
assert str(uuid.NAMESPACE_DNS) == "6ba7b810-9dad-11d1-80b4-00c04fd430c8"; _ledger.append(1)
assert str(uuid.NAMESPACE_URL) == "6ba7b811-9dad-11d1-80b4-00c04fd430c8"; _ledger.append(1)
assert str(uuid.NAMESPACE_OID) == "6ba7b812-9dad-11d1-80b4-00c04fd430c8"; _ledger.append(1)
assert str(uuid.NAMESPACE_X500) == "6ba7b814-9dad-11d1-80b4-00c04fd430c8"; _ledger.append(1)

# 4) time.strptime — parses `%Y` year token
_st = time.strptime("2024-01-15", "%Y-%m-%d")
assert _st.tm_year == 2024; _ledger.append(1)

# 5) time.struct_time — bare class identity
assert time.struct_time.__name__ == "struct_time"; _ledger.append(1)

# 6) types.SimpleNamespace — bare class identity
assert types.SimpleNamespace.__name__ == "SimpleNamespace"; _ledger.append(1)

# 7) types.SimpleNamespace — kwargs constructor + dot accessor
_ns = types.SimpleNamespace(x=1, y=2)
assert _ns.x == 1; _ledger.append(1)
assert _ns.y == 2; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_uuid_strptime_simplenamespace_silent {sum(_ledger)} asserts")
