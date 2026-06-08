# Operational AssertionPass seed for the value contract of the
# `struct` / `statistics` / `array` / `decimal` / `fractions`
# five-pack pinned to atomic 178: `struct` (the documented
# `pack` / `unpack` integer + float wire-format value contract
# with the documented `>i` / `>3i` / `>2f` format directives +
# the documented `calcsize` byte-size contract + the documented
# struct module hasattr surface), `statistics` (the documented
# `mean` / `median` / `mode` / `stdev` / `pstdev` / `variance`
# / `pvariance` / `harmonic_mean` / `geometric_mean` /
# `median_low` / `median_high` / `median_grouped` module-
# level helper float-cast value contract + the documented
# statistics module hasattr surface), `array` (the documented
# `array.array` / `ArrayType` / `typecodes` module hasattr
# surface), `decimal` (the documented `Decimal` module hasattr
# only), and `fractions` (the documented `Fraction` module
# hasattr only).
#
# The matching subset between mamba and CPython is the full
# `struct` pack / unpack integer + float wire-format layer
# (with format-prefix `>i` / `>3i` / `>2f` directives) + the
# full `struct.calcsize` byte-size layer + the full struct
# module hasattr surface (pack / unpack / calcsize / pack_into
# / unpack_from / Struct), the `statistics` float-cast value
# layer (mean / median / mode / stdev / pstdev / variance /
# pvariance / harmonic_mean / geometric_mean / median_low /
# median_high / median_grouped) + the full statistics module
# hasattr surface, the full `array` module hasattr surface
# (array / ArrayType / typecodes — instance layer DIVERGES),
# the `decimal.Decimal` module-level class hasattr (full
# instance + getcontext / setcontext / Context / ROUND_HALF_UP
# / ROUND_HALF_EVEN / InvalidOperation DIVERGE), and the
# `fractions.Fraction` module-level class hasattr (full
# instance DIVERGES).
#
# Surface in this fixture:
#   • struct.pack / unpack — big-endian int (`>i`);
#   • struct.pack / unpack — multi-int (`>3i`);
#   • struct.pack / unpack — multi-float (`>2f`);
#   • struct.calcsize — byte-size contract on `>i` / `>3i` /
#     `>2f`;
#   • struct — module hasattr surface (pack / unpack /
#     calcsize / pack_into / unpack_from / Struct);
#   • statistics.mean / median / mode — central-tendency
#     float-cast value contract;
#   • statistics.stdev / pstdev / variance / pvariance —
#     dispersion float-cast value contract;
#   • statistics.harmonic_mean / geometric_mean — alternative
#     mean float-cast value contract;
#   • statistics.median_low / median_high / median_grouped —
#     positional median value contract;
#   • statistics — module hasattr surface (mean / median /
#     mode / stdev / pstdev / variance / pvariance /
#     harmonic_mean / geometric_mean / median_low /
#     median_high / median_grouped / fmean / multimode /
#     quantiles);
#   • array — module hasattr surface (array / ArrayType /
#     typecodes);
#   • decimal — module hasattr surface (Decimal only —
#     getcontext / setcontext / Context / ROUND_HALF_UP /
#     ROUND_HALF_EVEN / InvalidOperation DIVERGE);
#   • fractions — module hasattr surface (Fraction only).
#
# Behavioral edges that DIVERGE on mamba (array.array('i',
# [1,2,3,4,5]) returns the int handle 0 not the documented
# array instance — array instance surface broken,
# struct.pack("3s", b"abc") packs only first byte +
# zero-fill — string-format directive broken, decimal.
# Decimal(s) returns an integer handle not the documented
# Decimal instance — Decimal arithmetic returns garbage
# small ints, decimal.getcontext / setcontext / Context /
# ROUND_HALF_UP / ROUND_HALF_EVEN / InvalidOperation hasattr
# False, fractions.Fraction(n, d) returns an integer handle
# not the documented Fraction instance — Fraction
# arithmetic returns garbage ints) are covered in the
# matching spec fixture
# `lang_array_decimal_fractions_struct_silent`.
import struct
import statistics
import array
import decimal
import fractions


_ledger: list[int] = []

# 1) struct.pack / unpack — big-endian int `>i`
_p = struct.pack(">i", 42)
assert struct.unpack(">i", _p) == (42,); _ledger.append(1)
assert struct.unpack(">i", struct.pack(">i", -1)) == (-1,); _ledger.append(1)

# 2) struct.pack / unpack — multi-int `>3i`
_p3 = struct.pack(">3i", 1, 2, 3)
assert struct.unpack(">3i", _p3) == (1, 2, 3); _ledger.append(1)

# 3) struct.pack / unpack — multi-float `>2f`
_pf = struct.pack(">2f", 1.0, 2.0)
_uf = struct.unpack(">2f", _pf)
assert _uf[0] == 1.0; _ledger.append(1)
assert _uf[1] == 2.0; _ledger.append(1)

# 4) struct.calcsize — byte-size contract
assert struct.calcsize(">i") == 4; _ledger.append(1)
assert struct.calcsize(">3i") == 12; _ledger.append(1)
assert struct.calcsize(">2f") == 8; _ledger.append(1)

# 5) struct — module hasattr surface
assert hasattr(struct, "pack") == True; _ledger.append(1)
assert hasattr(struct, "unpack") == True; _ledger.append(1)
assert hasattr(struct, "calcsize") == True; _ledger.append(1)
assert hasattr(struct, "pack_into") == True; _ledger.append(1)
assert hasattr(struct, "unpack_from") == True; _ledger.append(1)
assert hasattr(struct, "Struct") == True; _ledger.append(1)

# 6) statistics — central-tendency (float-cast to bypass
#    cpython-int-vs-mamba-float type variation on integer
#    inputs)
assert float(statistics.mean([1, 2, 3, 4, 5])) == 3.0; _ledger.append(1)
assert float(statistics.median([1, 2, 3, 4, 5])) == 3.0; _ledger.append(1)
assert statistics.mode([1, 1, 2, 3]) == 1; _ledger.append(1)

# 7) statistics — dispersion (float-cast)
assert round(float(statistics.stdev([1, 2, 3, 4, 5])), 4) == 1.5811; _ledger.append(1)
assert round(float(statistics.pstdev([1, 2, 3, 4, 5])), 4) == 1.4142; _ledger.append(1)
assert float(statistics.variance([1, 2, 3, 4, 5])) == 2.5; _ledger.append(1)
assert float(statistics.pvariance([1, 2, 3, 4, 5])) == 2.0; _ledger.append(1)

# 8) statistics — alternative means
assert round(float(statistics.harmonic_mean([1, 2, 4])), 4) == 1.7143; _ledger.append(1)
assert float(statistics.geometric_mean([1, 2, 4])) == 2.0; _ledger.append(1)

# 9) statistics — positional medians
assert statistics.median_low([1, 2, 3, 4]) == 2; _ledger.append(1)
assert statistics.median_high([1, 2, 3, 4]) == 3; _ledger.append(1)
assert float(statistics.median_grouped([1, 2, 3, 4])) == 2.5; _ledger.append(1)

# 10) statistics — module hasattr surface
assert hasattr(statistics, "mean") == True; _ledger.append(1)
assert hasattr(statistics, "median") == True; _ledger.append(1)
assert hasattr(statistics, "mode") == True; _ledger.append(1)
assert hasattr(statistics, "stdev") == True; _ledger.append(1)
assert hasattr(statistics, "pstdev") == True; _ledger.append(1)
assert hasattr(statistics, "variance") == True; _ledger.append(1)
assert hasattr(statistics, "pvariance") == True; _ledger.append(1)
assert hasattr(statistics, "harmonic_mean") == True; _ledger.append(1)
assert hasattr(statistics, "geometric_mean") == True; _ledger.append(1)
assert hasattr(statistics, "median_low") == True; _ledger.append(1)
assert hasattr(statistics, "median_high") == True; _ledger.append(1)
assert hasattr(statistics, "median_grouped") == True; _ledger.append(1)
assert hasattr(statistics, "fmean") == True; _ledger.append(1)
assert hasattr(statistics, "multimode") == True; _ledger.append(1)
assert hasattr(statistics, "quantiles") == True; _ledger.append(1)

# 11) array — module hasattr surface (instance DIVERGES —
#     moved to spec fixture)
assert hasattr(array, "array") == True; _ledger.append(1)
assert hasattr(array, "ArrayType") == True; _ledger.append(1)
assert hasattr(array, "typecodes") == True; _ledger.append(1)

# 12) decimal — module hasattr surface (Decimal only —
#     getcontext / setcontext / Context / ROUND_HALF_UP /
#     ROUND_HALF_EVEN / InvalidOperation DIVERGE — moved to
#     spec fixture)
assert hasattr(decimal, "Decimal") == True; _ledger.append(1)

# 13) fractions — module hasattr surface (Fraction only)
assert hasattr(fractions, "Fraction") == True; _ledger.append(1)

# NB: array.array(...) returns an integer handle on mamba —
# entire array instance surface broken, struct.pack("3s",
# b"abc") packs only first byte on mamba, decimal.Decimal(s)
# returns integer-handle + arithmetic returns garbage ints,
# decimal.getcontext / setcontext / Context / ROUND_HALF_UP
# / ROUND_HALF_EVEN / InvalidOperation hasattr False on
# mamba, fractions.Fraction(n, d) returns integer-handle +
# arithmetic returns garbage ints — all DIVERGE on mamba —
# moved to the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_struct_statistics_module_hasattr_value_ops {sum(_ledger)} asserts")
