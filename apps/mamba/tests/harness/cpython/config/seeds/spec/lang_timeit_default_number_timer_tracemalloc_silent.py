# Operational AssertionPass seed for SILENT divergences in `timeit`
# (return type and value of every public clock / measurement entry
# point + the documented module-level sentinels) and `tracemalloc`
# (Filter / Snapshot class identity, take_snapshot() return type,
# get_tracemalloc_memory bookkeeping helper). The matching subset
# (tracemalloc lifecycle gate + get_traced_memory tuple shape +
# top-level callables) is covered by
# `test_tracemalloc_start_stop_snapshot_ops`; this fixture pins the
# CPython-only contracts that mamba currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • timeit.default_timer() — returns `float`; mamba returns `int`
#     (looks like a raw nanosecond counter, not a seconds-as-float);
#   • timeit.default_timer() > 0 — True on CPython, False on mamba
#     (the int-typed counter compares against the float-zero literal);
#   • timeit.default_number == 1000000 — module-level sentinel for
#     the default measurement budget; mamba returns an unbound-method
#     handle;
#   • timeit.default_repeat == 5 — module-level sentinel for the
#     default number of measurement runs; mamba returns None;
#   • timeit.template — a documented `str` formatting template used
#     by the auto-template path; mamba returns None;
#   • timeit.timeit('1+1', number=10) — returns a positive float
#     (the measured time); mamba returns int 0;
#   • timeit.repeat('1+1', number=3, repeat=3) — returns a length-3
#     list of floats; mamba returns int 0;
#   • timeit.Timer('1+1') — returns a Timer instance; mamba returns
#     a plain dict;
#   • timeit.Timer.__name__ == "Timer" — class identity;
#   • timeit.Timer.print_exc — Timer class exposes a print_exc method;
#   • tracemalloc.get_tracemalloc_memory() — module-level helper for
#     the tracer's own bookkeeping cost; mamba raises AttributeError;
#   • tracemalloc.Filter — class identity used by snapshot filters;
#   • tracemalloc.Snapshot — class identity used as the return type
#     of take_snapshot();
#   • take_snapshot() returns a Snapshot instance (not a plain dict).
import timeit as _timeit_mod
import tracemalloc as _tm_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright's stub-driven
# attribute narrowing — `timeit.default_number` / `default_repeat` /
# `template` are documented module-level sentinels but Pyright's
# bundled stub does not surface them.
timeit: Any = _timeit_mod
tracemalloc: Any = _tm_mod

_ledger: list[int] = []

# 1) timeit.default_timer — returns float seconds
_t: Any = timeit.default_timer()
assert isinstance(_t, float); _ledger.append(1)
assert _t > 0; _ledger.append(1)
_t2: Any = timeit.default_timer()
assert isinstance(_t2, float); _ledger.append(1)
# Monotonic-ish — _t2 >= _t (both float, same clock source)
assert _t2 >= _t; _ledger.append(1)

# 2) timeit.default_number / default_repeat — module-level sentinels
assert timeit.default_number == 1000000; _ledger.append(1)
assert isinstance(timeit.default_number, int); _ledger.append(1)
assert timeit.default_repeat == 5; _ledger.append(1)
assert isinstance(timeit.default_repeat, int); _ledger.append(1)

# 3) timeit.template — str formatting template
assert isinstance(timeit.template, str); _ledger.append(1)
assert len(timeit.template) > 0; _ledger.append(1)

# 4) timeit.timeit() — returns a positive float
_r: Any = timeit.timeit('1+1', number=10)
assert isinstance(_r, float); _ledger.append(1)
assert _r > 0; _ledger.append(1)
_r_call: Any = timeit.timeit(lambda: 1 + 1, number=10)
assert isinstance(_r_call, float); _ledger.append(1)
assert _r_call > 0; _ledger.append(1)

# 5) timeit.repeat() — returns a list[float] of length `repeat`
_rep: Any = timeit.repeat('1+1', number=3, repeat=3)
assert isinstance(_rep, list); _ledger.append(1)
assert len(_rep) == 3; _ledger.append(1)
for _x in _rep:
    assert isinstance(_x, float); _ledger.append(1)

# 6) timeit.Timer — class with a print_exc method, .timeit instance method
assert timeit.Timer.__name__ == "Timer"; _ledger.append(1)
assert hasattr(timeit.Timer, "print_exc"); _ledger.append(1)
_tm: Any = timeit.Timer('1+1')
assert type(_tm).__name__ == "Timer"; _ledger.append(1)
_tm_r: Any = _tm.timeit(number=10)
assert isinstance(_tm_r, float); _ledger.append(1)
assert _tm_r > 0; _ledger.append(1)

# 7) tracemalloc.get_tracemalloc_memory() — bookkeeping-overhead helper
tracemalloc.start()
_om: Any = tracemalloc.get_tracemalloc_memory()
assert isinstance(_om, int); _ledger.append(1)
assert _om > 0; _ledger.append(1)

# 8) tracemalloc.Filter / Snapshot — class identity
assert tracemalloc.Filter.__name__ == "Filter"; _ledger.append(1)
assert tracemalloc.Snapshot.__name__ == "Snapshot"; _ledger.append(1)

# 9) take_snapshot() returns a Snapshot instance
_snap: Any = tracemalloc.take_snapshot()
assert type(_snap).__name__ == "Snapshot"; _ledger.append(1)
assert isinstance(_snap, tracemalloc.Snapshot); _ledger.append(1)
tracemalloc.stop()

print(f"MAMBA_ASSERTION_PASS: lang_timeit_default_number_timer_tracemalloc_silent {sum(_ledger)} asserts")
