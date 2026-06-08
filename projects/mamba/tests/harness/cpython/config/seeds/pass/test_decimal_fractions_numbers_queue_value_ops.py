# Operational AssertionPass seed for the value contract of the
# `decimal` / `fractions` / `numbers` / `queue` four-pack
# pinned to atomic 213: `decimal` (the documented partial
# module-level helper identifier hasattr surface — `Decimal`),
# `fractions` (the documented partial module-level class
# identifier hasattr surface — `Fraction` + the
# documented `fractions.Fraction(1, 3).numerator == 1` /
# `fractions.Fraction(1, 3).denominator == 3` /
# `type(fractions.Fraction(1, 3).numerator).__name__ == "int"`
# / `type(fractions.Fraction(1, 3).denominator).__name__ ==
# "int"` numerator/denominator value contract), `numbers`
# (the documented full module-level class identifier hasattr
# surface — `Number` / `Complex` / `Real` / `Rational` /
# `Integral`), and `queue` (the documented full module-level
# helper / class / exception identifier hasattr surface —
# `Queue` / `LifoQueue` / `PriorityQueue` / `SimpleQueue` /
# `Empty` / `Full` + the documented
# `queue.Queue().put(...).put(...).put(...).qsize() == 3` /
# `queue.Queue().get() == 1` / `queue.Queue().empty()
# == False/True` FIFO queue-protocol value contract +
# the documented `queue.LifoQueue().put(1).put(2).put(3)
# .get() == 3` LIFO queue-protocol value contract).
#
# Behavioral edges that DIVERGE on mamba
# (hasattr(decimal, "Context") / "getcontext" / "setcontext"
# / "localcontext" / "ROUND_HALF_UP" / "ROUND_HALF_EVEN" /
# "ROUND_DOWN" / "ROUND_UP" / "ROUND_HALF_DOWN" /
# "ROUND_CEILING" / "ROUND_FLOOR" / "DivisionByZero" /
# "InvalidOperation" / "Inexact" / "Overflow" / "Underflow"
# / "Subnormal" / "Rounded" / "Clamped" / "MAX_PREC" /
# "MAX_EMAX" / "MIN_EMIN" / "MIN_ETINY" / "HAVE_THREADS" /
# "BasicContext" / "ExtendedContext" / "DefaultContext"
# all False on mamba +
# type(decimal.Decimal("1.5")).__name__ == "Decimal"
# collapses to "int" on mamba +
# str(decimal.Decimal("1.5")) == "1.5" collapses to
# "70368744177664" on mamba, type(fractions.Fraction
# (1, 3)).__name__ == "Fraction" collapses to "int" on
# mamba + str(fractions.Fraction(1, 3)) == "1/3"
# collapses to huge-int on mamba, isinstance(5, numbers.
# Integral) / (5.0, numbers.Real) / (5, numbers.Number)
# / issubclass(int, numbers.Integral) / (float, numbers.
# Real) all False on mamba, type(queue.Queue()).__name__
# == "Queue" collapses to "int" on mamba, hasattr(sched,
# "scheduler") / "Event" all False on mamba) are covered
# in the matching spec fixture
# `lang_decimal_fractions_numbers_queue_sched_silent`.
import decimal
import fractions
import numbers
import queue


_ledger: list[int] = []

# 1) decimal — partial module hasattr surface
#    (Context / getcontext / setcontext / localcontext /
#    ROUND_* / exception types / context-sentinel attributes
#    all DIVERGE on mamba — moved to spec)
assert hasattr(decimal, "Decimal") == True; _ledger.append(1)

# 2) fractions — partial module hasattr surface
#    (Fraction value contracts including type identity,
#    str, arithmetic DIVERGE on mamba — moved to spec.
#    `gcd` was removed from `fractions` in CPython 3.9
#    so it is NOT part of the documented hasattr surface;
#    use `math.gcd` instead.)
assert hasattr(fractions, "Fraction") == True; _ledger.append(1)

# 3) fractions — numerator/denominator value contract
_f = fractions.Fraction(1, 3)
assert _f.numerator == 1; _ledger.append(1)
assert _f.denominator == 3; _ledger.append(1)
assert type(_f.numerator).__name__ == "int"; _ledger.append(1)
assert type(_f.denominator).__name__ == "int"; _ledger.append(1)

# 4) numbers — full module hasattr surface
#    (isinstance / issubclass against the numbers tower
#    DIVERGE on mamba — moved to spec)
assert hasattr(numbers, "Number") == True; _ledger.append(1)
assert hasattr(numbers, "Complex") == True; _ledger.append(1)
assert hasattr(numbers, "Real") == True; _ledger.append(1)
assert hasattr(numbers, "Rational") == True; _ledger.append(1)
assert hasattr(numbers, "Integral") == True; _ledger.append(1)

# 5) queue — full module hasattr surface
#    (type(queue.Queue()).__name__ == "Queue" type-identity
#    DIVERGE on mamba — moved to spec)
assert hasattr(queue, "Queue") == True; _ledger.append(1)
assert hasattr(queue, "LifoQueue") == True; _ledger.append(1)
assert hasattr(queue, "PriorityQueue") == True; _ledger.append(1)
assert hasattr(queue, "SimpleQueue") == True; _ledger.append(1)
assert hasattr(queue, "Empty") == True; _ledger.append(1)
assert hasattr(queue, "Full") == True; _ledger.append(1)

# 6) queue — FIFO queue-protocol value contract
_q = queue.Queue()
_q.put(1)
_q.put(2)
_q.put(3)
assert _q.qsize() == 3; _ledger.append(1)
assert _q.get() == 1; _ledger.append(1)
assert _q.get() == 2; _ledger.append(1)
assert _q.empty() == False; _ledger.append(1)
assert _q.qsize() == 1; _ledger.append(1)
assert _q.get() == 3; _ledger.append(1)
assert _q.empty() == True; _ledger.append(1)

# 7) queue — LIFO queue-protocol value contract
_lq = queue.LifoQueue()
_lq.put(1)
_lq.put(2)
_lq.put(3)
assert _lq.get() == 3; _ledger.append(1)
assert _lq.get() == 2; _ledger.append(1)
assert _lq.get() == 1; _ledger.append(1)

# NB: hasattr(decimal, "Context") / "getcontext" / "setcontext"
# / "localcontext" / "ROUND_HALF_UP" / "ROUND_HALF_EVEN" /
# "ROUND_DOWN" / "ROUND_UP" / "ROUND_HALF_DOWN" /
# "ROUND_CEILING" / "ROUND_FLOOR" / "DivisionByZero" /
# "InvalidOperation" / "Inexact" / "Overflow" / "Underflow"
# / "Subnormal" / "Rounded" / "Clamped" / "MAX_PREC" /
# "MAX_EMAX" / "MIN_EMIN" / "MIN_ETINY" / "HAVE_THREADS" /
# "BasicContext" / "ExtendedContext" / "DefaultContext"
# all False on mamba +
# type(decimal.Decimal("1.5")).__name__ == "Decimal"
# collapses to "int" on mamba + Decimal arithmetic /
# str / equality value contracts all collapse on mamba,
# type(fractions.Fraction(1, 3)).__name__ == "Fraction"
# collapses to "int" on mamba + str(fractions.Fraction
# (1, 3)) == "1/3" collapses to huge-int on mamba +
# Fraction arithmetic / limit_denominator / float-init
# all collapse on mamba, isinstance(5, numbers.Integral)
# / (5.0, numbers.Real) / (5, numbers.Number) /
# issubclass(int, numbers.Integral) / (float, numbers.
# Real) all False on mamba, type(queue.Queue()).__name__
# == "Queue" collapses to "int" on mamba, hasattr(sched,
# "scheduler") / "Event" all False on mamba — all
# DIVERGE on mamba — moved to the divergence-spec
# fixture.

print(f"MAMBA_ASSERTION_PASS: test_decimal_fractions_numbers_queue_value_ops {sum(_ledger)} asserts")
