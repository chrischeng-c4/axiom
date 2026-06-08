# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_decimal_fractions_numbers_queue_sched_silent"
# subject = "cpython321.lang_decimal_fractions_numbers_queue_sched_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_decimal_fractions_numbers_queue_sched_silent.py"
# status = "filled"
# ///
"""cpython321.lang_decimal_fractions_numbers_queue_sched_silent: execute CPython 3.12 seed lang_decimal_fractions_numbers_queue_sched_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the
# silent value-contract divergence of the `decimal` /
# `fractions` / `numbers` / `queue` / `sched` five-pack
# pinned to atomic 213: `decimal` (the documented
# `hasattr(decimal, "Context") / "getcontext" / "setcontext"
# / "localcontext" / "ROUND_HALF_UP" / "ROUND_HALF_EVEN" /
# "DivisionByZero" / "InvalidOperation" / "MAX_PREC" /
# "DefaultContext" == True` extended hasattr surface + the
# documented `type(decimal.Decimal("1.5")).__name__ ==
# "Decimal"` constructor-identity value contract + the
# documented `str(decimal.Decimal("1.5")) == "1.5"` /
# `decimal.Decimal("1.5") + decimal.Decimal("2.5") ==
# decimal.Decimal("4.0")` arithmetic value contract),
# `fractions` (the documented
# `type(fractions.Fraction(1, 3)).__name__ == "Fraction"`
# constructor-identity value contract + the documented
# `str(fractions.Fraction(1, 3)) == "1/3"` /
# `str(fractions.Fraction(1, 3) + fractions.Fraction(1, 6))
# == "1/2"` /
# `str(fractions.Fraction(355, 113).limit_denominator(100))
# == "311/99"` /
# `str(fractions.Fraction(0.5)) == "1/2"` Fraction
# repr / arithmetic value contract), `numbers` (the
# documented `isinstance(5, numbers.Integral) == True` /
# `isinstance(5.0, numbers.Real) == True` /
# `isinstance(5, numbers.Number) == True` /
# `issubclass(int, numbers.Integral) == True` /
# `issubclass(float, numbers.Real) == True` numbers-
# tower isinstance/issubclass value contract), `queue`
# (the documented `type(queue.Queue()).__name__ ==
# "Queue"` Queue constructor-identity value contract),
# and `sched` (the documented
# `hasattr(sched, "scheduler") / "Event" == True`
# module-level helper / class identifier hasattr surface).
#
# Behavioral edges that CONFORM on mamba
# (decimal `Decimal` hasattr, fractions `Fraction` hasattr
# + `Fraction(1, 3).numerator == 1` / `.denominator == 3`
# integer-component value contract, numbers `Number` /
# `Complex` / `Real` / `Rational` / `Integral` hasattr
# surface, queue `Queue` / `LifoQueue` / `PriorityQueue`
# / `SimpleQueue` / `Empty` / `Full` hasattr surface +
# queue.Queue() FIFO put/get/qsize/empty protocol +
# queue.LifoQueue() LIFO put/get protocol) are covered
# in the matching pass fixture
# `test_decimal_fractions_numbers_queue_value_ops`.
from typing import Any
import decimal as _decimal_mod
import fractions as _fractions_mod
import numbers as _numbers_mod
import queue as _queue_mod
import sched as _sched_mod

decimal: Any = _decimal_mod
fractions: Any = _fractions_mod
numbers: Any = _numbers_mod
queue: Any = _queue_mod
sched: Any = _sched_mod


_ledger: list[int] = []

# 1) decimal — extended module hasattr surface
#    (mamba: Context / getcontext / setcontext / localcontext
#    / ROUND_* / exception types / context-sentinel attributes
#    all False)
assert hasattr(decimal, "Context") == True; _ledger.append(1)
assert hasattr(decimal, "getcontext") == True; _ledger.append(1)
assert hasattr(decimal, "setcontext") == True; _ledger.append(1)
assert hasattr(decimal, "localcontext") == True; _ledger.append(1)
assert hasattr(decimal, "ROUND_HALF_UP") == True; _ledger.append(1)
assert hasattr(decimal, "ROUND_HALF_EVEN") == True; _ledger.append(1)
assert hasattr(decimal, "DivisionByZero") == True; _ledger.append(1)
assert hasattr(decimal, "InvalidOperation") == True; _ledger.append(1)
assert hasattr(decimal, "MAX_PREC") == True; _ledger.append(1)
assert hasattr(decimal, "DefaultContext") == True; _ledger.append(1)

# 2) decimal — Decimal constructor / arithmetic value contract
#    (mamba: type(decimal.Decimal("1.5")).__name__ collapses
#    to "int" + str(Decimal("1.5")) collapses to a huge int +
#    Decimal arithmetic produces unrelated integer results)
_d = decimal.Decimal("1.5")
assert type(_d).__name__ == "Decimal"; _ledger.append(1)
assert str(_d) == "1.5"; _ledger.append(1)
assert str(_d + decimal.Decimal("2.5")) == "4.0"; _ledger.append(1)

# 3) fractions — Fraction repr / arithmetic value contract
#    (mamba: type(fractions.Fraction(1, 3)).__name__ collapses
#    to "int" + str(Fraction(1, 3)) collapses to a huge int +
#    Fraction arithmetic / limit_denominator / float-init all
#    collapse)
_f = fractions.Fraction(1, 3)
assert type(_f).__name__ == "Fraction"; _ledger.append(1)
assert str(_f) == "1/3"; _ledger.append(1)
assert str(_f + fractions.Fraction(1, 6)) == "1/2"; _ledger.append(1)
assert str(fractions.Fraction(355, 113).limit_denominator(100)) == "311/99"; _ledger.append(1)
assert str(fractions.Fraction(0.5)) == "1/2"; _ledger.append(1)

# 4) numbers — numbers-tower isinstance/issubclass value
#    contract
#    (mamba: every isinstance / issubclass check against the
#    numbers ABCs collapses to False)
assert isinstance(5, numbers.Integral) == True; _ledger.append(1)
assert isinstance(5.0, numbers.Real) == True; _ledger.append(1)
assert isinstance(5, numbers.Number) == True; _ledger.append(1)
assert issubclass(int, numbers.Integral) == True; _ledger.append(1)
assert issubclass(float, numbers.Real) == True; _ledger.append(1)

# 5) queue — Queue constructor-identity value contract
#    (mamba: type(queue.Queue()).__name__ collapses to "int"
#    even though the resulting object honors the Queue
#    protocol)
assert type(queue.Queue()).__name__ == "Queue"; _ledger.append(1)

# 6) sched — module-level helper / class identifier hasattr
#    surface
#    (mamba: scheduler / Event both False)
assert hasattr(sched, "scheduler") == True; _ledger.append(1)
assert hasattr(sched, "Event") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_decimal_fractions_numbers_queue_sched_silent {sum(_ledger)} asserts")
