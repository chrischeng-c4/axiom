# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `hasattr(decimal, 'getcontext')`
# (the documented "decimal exposes the active-context accessor" —
# mamba returns False), `hasattr(decimal, 'ROUND_HALF_EVEN')` (the
# documented "decimal exposes the IEEE-754 banker's-rounding mode
# constant" — mamba returns False), `hasattr(decimal,
# 'InvalidOperation')` (the documented "decimal exposes the
# InvalidOperation signal class" — mamba returns False),
# `type(decimal.Decimal('1.5')).__name__ == 'Decimal'` (the
# documented "Decimal() returns a Decimal instance" — mamba returns
# 'int' — handle-typed sentinel), `str(decimal.Decimal('1.5')) ==
# '1.5'` (the documented "str(Decimal) emits the canonical
# numeric-literal form" — mamba returns '70368744177665' — bare
# integer handle), `decimal.Decimal('1.1') == decimal.Decimal('1.1
# ')` (the documented "two Decimals with the same literal compare
# equal" — mamba returns False — handle-id-based equality, two
# separate Decimal() calls produce distinct handles),
# `type(fractions.Fraction(1, 2)).__name__ == 'Fraction'` (the
# documented "Fraction() returns a Fraction instance" — mamba
# returns 'int' — handle-typed sentinel), `str(fractions.Fraction(1
# , 2)) == '1/2'` (the documented "str(Fraction) emits the
# canonical n/d form" — mamba returns '1099511627777' — bare
# integer handle), `statistics.mean([1, 2, 3]) == 2` (the documented
# "mean of [1,2,3] is 2" — mamba returns 4611686018427387904 —
# IEEE-754 bit pattern of 2.0 as int), and `type(collections.
# ChainMap({1: 1}, {2: 2})).__name__ == 'ChainMap'` (the documented
# "ChainMap() returns a ChainMap instance" — mamba returns 'dict' —
# the constructor degrades to a merged dict).
# Ten-pack pinned to atomic 295.
#
# Behavioral edges that CONFORM on mamba (decimal — hasattr Decimal.
# fractions — hasattr Fraction + numerator/denominator. statistics
# — hasattr mean/median/mode/stdev/variance/pstdev/pvariance/median_
# low/median_high/median_grouped/multimode/quantiles/harmonic_mean/
# geometric_mean/fmean/StatisticsError/NormalDist + integer-
# returning median/mode. collections — hasattr deque/Counter/
# defaultdict/OrderedDict/namedtuple/ChainMap/UserDict/UserList/
# UserString + Counter element frequency) are covered in the
# matching pass fixture `test_decimal_fractions_statistics_
# collections_value_ops`.
import decimal
import fractions
import statistics
import collections


_ledger: list[int] = []

# 1) hasattr(decimal, 'getcontext') — active-context accessor
#    (mamba: returns False)
assert hasattr(decimal, "getcontext") == True; _ledger.append(1)

# 2) hasattr(decimal, 'ROUND_HALF_EVEN') — banker's-rounding constant
#    (mamba: returns False)
assert hasattr(decimal, "ROUND_HALF_EVEN") == True; _ledger.append(1)

# 3) hasattr(decimal, 'InvalidOperation') — InvalidOperation signal class
#    (mamba: returns False)
assert hasattr(decimal, "InvalidOperation") == True; _ledger.append(1)

# 4) type(decimal.Decimal('1.5')).__name__ == 'Decimal' — Decimal instance
#    (mamba: returns 'int' — handle-typed sentinel)
assert type(decimal.Decimal("1.5")).__name__ == "Decimal"; _ledger.append(1)

# 5) str(decimal.Decimal('1.5')) == '1.5' — canonical numeric-literal form
#    (mamba: returns '70368744177665' — bare integer handle)
assert str(decimal.Decimal("1.5")) == "1.5"; _ledger.append(1)

# 6) decimal.Decimal('1.1') == decimal.Decimal('1.1') — equal literals compare equal
#    (mamba: returns False — handle-id-based equality across separate ctors)
assert (decimal.Decimal("1.1") == decimal.Decimal("1.1")) == True; _ledger.append(1)

# 7) type(fractions.Fraction(1, 2)).__name__ == 'Fraction' — Fraction instance
#    (mamba: returns 'int' — handle-typed sentinel)
assert type(fractions.Fraction(1, 2)).__name__ == "Fraction"; _ledger.append(1)

# 8) str(fractions.Fraction(1, 2)) == '1/2' — canonical n/d form
#    (mamba: returns '1099511627777' — bare integer handle)
assert str(fractions.Fraction(1, 2)) == "1/2"; _ledger.append(1)

# 9) statistics.mean([1, 2, 3]) == 2 — mean is a Python number == 2
#    (mamba: returns 4611686018427387904 — i64 bit pattern of 2.0)
assert statistics.mean([1, 2, 3]) == 2; _ledger.append(1)

# 10) type(collections.ChainMap({1: 1}, {2: 2})).__name__ == 'ChainMap' — ChainMap instance
#     (mamba: returns 'dict' — constructor degrades to merged dict)
assert type(collections.ChainMap({1: 1}, {2: 2})).__name__ == "ChainMap"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_decimal_fractions_statistics_silent {sum(_ledger)} asserts")
