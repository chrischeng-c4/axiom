# Operational AssertionPass seed for SILENT divergences across the
# arbitrary-precision / rational-arithmetic / modal-summary trio
# pinned by atomic 163: `decimal` (the documented `Decimal`
# constructor + arithmetic / str / repr / equality / quantize
# contracts + `getcontext` / `ROUND_HALF_UP` / `ROUND_DOWN`
# attribute surface), `fractions` (the documented `Fraction`
# arithmetic + string-constructor + float-constructor +
# `limit_denominator` contracts), and `statistics` (the
# documented `mode` modal-element contract on string bags).
#
# The matching subset (statistics float-arithmetic mean / median /
# variance / stdev / pstdev / fmean / median_low / median_high
# + integer-bag mode, statistics module hasattr surface,
# decimal.Decimal class hasattr only, fractions.Fraction class
# hasattr + numerator / denominator attribute, hashlib full
# digest surface — sha256 / md5 / sha1 / sha512 / new /
# algorithms_guaranteed + hexdigest / digest_size / name +
# incremental update) is covered by
# `test_statistics_hashlib_value_ops`; this fixture pins the
# CPython-only contracts that mamba currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • Decimal("1.1") + Decimal("2.2") == Decimal("3.3") —
#     arbitrary-precision addition (mamba: returns
#     -140737488355327 — int-handle pattern overflow);
#   • str(Decimal("1.1")) == "1.1" — documented str contract
#     (mamba: returns "70368744177664" — int-handle pattern
#     id leaks through str());
#   • repr(Decimal("1.1")) == "Decimal('1.1')" — documented
#     repr contract (mamba: returns "70368744177664");
#   • Decimal("1.1") == Decimal("1.1") is True — equality
#     contract (mamba: returns False, int-handles diverge per
#     construction);
#   • hasattr(decimal, "getcontext") is True — documented
#     context accessor (mamba: False);
#   • hasattr(decimal, "ROUND_HALF_UP") is True — documented
#     rounding-mode constant (mamba: False);
#   • hasattr(decimal, "ROUND_DOWN") is True (mamba: False);
#   • Fraction(1,2) + Fraction(1,3) == Fraction(5,6) —
#     rational addition (mamba: returns 2199023255553 int-handle
#     garbage);
#   • str(Fraction(1,2)) == "1/2" — documented str contract
#     (mamba: returns "1099511627776" — int-handle id leaks);
#   • Fraction("3/4") == Fraction(3, 4) — string constructor
#     contract (mamba: returns 1099511627778 int-handle);
#   • Fraction(0.5) == Fraction(1, 2) — float constructor
#     contract (mamba: returns 1099511627779 int-handle);
#   • Fraction(0.1).limit_denominator(10) == Fraction(1, 10)
#     — denominator-bounded approximation (mamba: returns
#     1099511627780 int-handle);
#   • statistics.mode(["a","b","a","c"]) == "a" — modal-
#     element contract on string bag (mamba: returns None).
import decimal as _decimal_mod
import fractions as _fractions_mod
import statistics as _statistics_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# class identifiers / module-level helpers / instance methods
# that mamba's bundled type stubs do not surface accurately.
decimal: Any = _decimal_mod
fractions: Any = _fractions_mod
statistics: Any = _statistics_mod


_ledger: list[int] = []

# 1) Decimal — arithmetic / str / repr / equality
_d1 = decimal.Decimal("1.1")
_d2 = decimal.Decimal("2.2")
assert _d1 + _d2 == decimal.Decimal("3.3"); _ledger.append(1)
assert str(_d1) == "1.1"; _ledger.append(1)
assert repr(_d1) == "Decimal('1.1')"; _ledger.append(1)
assert (_d1 == decimal.Decimal("1.1")) == True; _ledger.append(1)

# 2) decimal — documented module attribute surface
assert hasattr(decimal, "getcontext") == True; _ledger.append(1)
assert hasattr(decimal, "ROUND_HALF_UP") == True; _ledger.append(1)
assert hasattr(decimal, "ROUND_DOWN") == True; _ledger.append(1)

# 3) Fraction — arithmetic + str + canonical equality
_f1 = fractions.Fraction(1, 2)
_f2 = fractions.Fraction(1, 3)
assert _f1 + _f2 == fractions.Fraction(5, 6); _ledger.append(1)
assert str(_f1) == "1/2"; _ledger.append(1)

# 4) Fraction — string + float constructors
assert fractions.Fraction("3/4") == fractions.Fraction(3, 4); _ledger.append(1)
assert fractions.Fraction(0.5) == fractions.Fraction(1, 2); _ledger.append(1)

# 5) Fraction — limit_denominator approximation
assert fractions.Fraction(0.1).limit_denominator(10) == fractions.Fraction(1, 10); _ledger.append(1)

# 6) statistics.mode — modal element on string bag
assert statistics.mode(["a", "b", "a", "c"]) == "a"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_decimal_fraction_mode_silent {sum(_ledger)} asserts")
