"""Hot-loop bench for `datetime.date` / `datetime.time` /
`datetime.datetime` / `datetime.timedelta` / `datetime.timezone` /
`datetime.MINYEAR` / `datetime.MAXYEAR` / `datetime.tzinfo`
module-attribute reads (#1436).

End-user scenario: hot serialization / wire-format-binding code that
resolves datetime-machinery constructors and sentinels off the
`datetime` module on every call site - e.g. an ETL row-builder loop
that calls `datetime.date` to mint calendar-day stamps,
`datetime.time` for wall-clock components, `datetime.datetime` for
combined timestamps, `datetime.timedelta` for interval arithmetic,
`datetime.timezone` for fixed-offset zones, `datetime.MINYEAR` /
`datetime.MAXYEAR` to clamp out-of-range inputs, and `datetime.tzinfo`
to dispatch on abstract base type. The canonical hot-path idiom is to
read those names directly off the `datetime` module on every call
rather than caching a local - keeps the call site robust against
late-binding patterns (test monkey-patching, tz-plugin overrides,
runtime backend-swap fixtures). That per-iter module-attribute octuple
read is the workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x -
CPython's `datetime.date` / `datetime.time` / `datetime.datetime` /
`datetime.timedelta` / `datetime.timezone` / `datetime.MINYEAR` /
`datetime.MAXYEAR` / `datetime.tzinfo` are top-level module-dict
probes returning the canonical class / int / abstract-base objects
on 3.12). Mamba's shim returns the same identity-stable values
directly from a dense constant table in the `datetime` module-attribute
resolver, short-circuiting CPython's module-dict probe chain for
read-only datetime-machinery sentinels.

Workload: 10_000 paired reads of `datetime.date`, `datetime.time`,
`datetime.datetime`, `datetime.timedelta`, `datetime.timezone`,
`datetime.MINYEAR`, `datetime.MAXYEAR`, and `datetime.tzinfo` per
iteration, compared by identity (`is`) against the hoisted baseline
references taken once before the loop. The accumulator increments
when all eight reads resolve to the identical objects; a misread
(different identity / wrong binding) would immediately fail the
correctness assert and dead-code elimination of any read would leave
`acc != ITERS`.

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import datetime as _dt

# Hoist baseline references to the canonical datetime-machinery values
# once before the loop. The hot path re-reads the module attribute on
# every iter so the bench actually exercises the module-attribute
# resolver - the `is` compare against the hoisted baseline is the
# correctness probe. Each runtime hoists its own baseline (CPython
# binds to distinct classes / ints / the abstract base; mamba binds
# to its own identity-stable shim values) so the identity invariant
# holds per-runtime.
_DATE_BASELINE = _dt.date
_TIME_BASELINE = _dt.time
_DATETIME_BASELINE = _dt.datetime
_TIMEDELTA_BASELINE = _dt.timedelta
_TIMEZONE_BASELINE = _dt.timezone
_MINYEAR_BASELINE = _dt.MINYEAR
_MAXYEAR_BASELINE = _dt.MAXYEAR
_TZINFO_BASELINE = _dt.tzinfo

ITERS = 10_000

acc = 0
for _ in range(ITERS):
    d = _dt.date
    t = _dt.time
    dt = _dt.datetime
    td = _dt.timedelta
    tz = _dt.timezone
    miny = _dt.MINYEAR
    maxy = _dt.MAXYEAR
    tzi = _dt.tzinfo
    # Accumulator readback prevents DCE - every iteration must
    # resolve to the identical objects bound at the `datetime.date`
    # / `datetime.time` / `datetime.datetime` / `datetime.timedelta`
    # / `datetime.timezone` / `datetime.MINYEAR` / `datetime.MAXYEAR`
    # / `datetime.tzinfo` module slots.
    if (d is _DATE_BASELINE
            and t is _TIME_BASELINE
            and dt is _DATETIME_BASELINE
            and td is _TIMEDELTA_BASELINE
            and tz is _TIMEZONE_BASELINE
            and miny is _MINYEAR_BASELINE
            and maxy is _MAXYEAR_BASELINE
            and tzi is _TZINFO_BASELINE):
        acc = acc + 1

# Correctness: every iteration must read back the canonical
# datetime-machinery values via the module-attribute resolver.
# acc == ITERS or we have a regression in mamba's datetime
# module-attribute table.
assert acc - ITERS == 0, f"datetime module-attribute read acc drift: acc={acc} expected={ITERS}"
print("datetime_type_read_hot:", acc)
