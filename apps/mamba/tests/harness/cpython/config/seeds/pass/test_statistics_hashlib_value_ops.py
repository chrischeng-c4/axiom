# Operational AssertionPass seed for the value contract of four
# bootstrap stdlib modules used by every numeric-statistics /
# arbitrary-precision / rational-arithmetic / cryptographic-digest
# path: `statistics` (the documented `mean` / `median` /
# `median_low` / `median_high` / `mode` / `variance` / `stdev` /
# `pvariance` / `pstdev` / `fmean` / `quantiles` numeric-summary
# surface), `decimal` (the documented `Decimal` class identifier
# only — arithmetic and quantize contracts diverge silently and
# move to the spec fixture), `fractions` (the documented
# `Fraction` class identifier + `numerator` / `denominator`
# attribute surface only — arithmetic and constructor contracts
# diverge silently and move to the spec fixture), and `hashlib`
# (the documented `sha256` / `md5` / `sha1` / `sha512` / `new` /
# `algorithms_guaranteed` attribute surface + the documented
# `hexdigest` / `digest_size` / `name` / `update` instance-method
# surface).
#
# The matching subset between mamba and CPython is the statistics
# float-arithmetic layer (median odd / median even / median_low /
# median_high / fmean / mean / variance / stdev / pvariance /
# pstdev under explicit-float inputs), the integer-mode integer-
# bag layer (mode([1,1,2,3]) == 1), the decimal hasattr surface,
# the fractions hasattr surface + numerator / denominator
# attribute layer, and the hashlib full digest layer
# (sha256 / md5 / sha1 + hexdigest output bytes + digest_size +
# name attribute + incremental update + new(name) factory).
#
# Surface in this fixture:
#   • statistics — mean / median / median_low / median_high /
#     mode / variance / stdev / pvariance / pstdev / fmean
#     under explicit-float inputs + module hasattr surface;
#   • decimal — `Decimal` class hasattr only;
#   • fractions — `Fraction` class hasattr + numerator /
#     denominator attribute;
#   • hashlib — sha256 / md5 / sha1 / sha512 / new digest
#     constructors + hexdigest / digest_size / name attribute +
#     incremental update chain.
#
# Behavioral edges that DIVERGE on mamba (statistics.mode on a
# str-bag returns None instead of the modal element,
# decimal.getcontext / ROUND_HALF_UP / ROUND_DOWN absent,
# decimal.Decimal('1.1') + decimal.Decimal('2.2') corrupts to
# an int-handle pattern returning -140737488355327, str /
# repr / equality / quantize on Decimal all broken, Fraction
# arithmetic returns garbage int-handle values, Fraction
# string and float constructors return int-handle garbage,
# limit_denominator broken) are covered in the matching spec
# fixture `lang_decimal_fraction_mode_silent`.
import statistics
import decimal
import fractions
import hashlib


_ledger: list[int] = []

# 1) statistics — float-arithmetic surface
assert statistics.mean([1.0, 2.0, 3.0, 4.0, 5.0]) == 3.0; _ledger.append(1)
assert statistics.median([1, 3, 5]) == 3; _ledger.append(1)
assert statistics.median([1, 2, 3, 4]) == 2.5; _ledger.append(1)
assert statistics.median_low([1, 2, 3, 4]) == 2; _ledger.append(1)
assert statistics.median_high([1, 2, 3, 4]) == 3; _ledger.append(1)
assert statistics.mode([1, 1, 2, 3]) == 1; _ledger.append(1)
assert statistics.variance([1.0, 2.0, 3.0, 4.0, 5.0]) == 2.5; _ledger.append(1)
assert statistics.pstdev([1, 2, 3, 4, 5]) == 1.4142135623730951; _ledger.append(1)
assert statistics.fmean([1, 2, 3]) == 2.0; _ledger.append(1)

# 2) statistics — module attribute surface
assert hasattr(statistics, "mean") == True; _ledger.append(1)
assert hasattr(statistics, "median") == True; _ledger.append(1)
assert hasattr(statistics, "mode") == True; _ledger.append(1)
assert hasattr(statistics, "stdev") == True; _ledger.append(1)
assert hasattr(statistics, "variance") == True; _ledger.append(1)
assert hasattr(statistics, "pstdev") == True; _ledger.append(1)
assert hasattr(statistics, "pvariance") == True; _ledger.append(1)
assert hasattr(statistics, "fmean") == True; _ledger.append(1)
assert hasattr(statistics, "median_low") == True; _ledger.append(1)
assert hasattr(statistics, "median_high") == True; _ledger.append(1)
assert hasattr(statistics, "quantiles") == True; _ledger.append(1)

# 3) decimal — class identifier hasattr
assert hasattr(decimal, "Decimal") == True; _ledger.append(1)

# 4) fractions — class identifier + numerator / denominator
assert hasattr(fractions, "Fraction") == True; _ledger.append(1)
_f = fractions.Fraction(1, 2)
assert _f.numerator == 1; _ledger.append(1)
assert _f.denominator == 2; _ledger.append(1)

# 5) hashlib — module attribute surface
assert hasattr(hashlib, "sha256") == True; _ledger.append(1)
assert hasattr(hashlib, "md5") == True; _ledger.append(1)
assert hasattr(hashlib, "sha1") == True; _ledger.append(1)
assert hasattr(hashlib, "sha512") == True; _ledger.append(1)
assert hasattr(hashlib, "new") == True; _ledger.append(1)
assert hasattr(hashlib, "algorithms_guaranteed") == True; _ledger.append(1)

# 6) hashlib — sha256 + hexdigest + digest_size + name
_h_sha256 = hashlib.sha256(b"hello")
assert _h_sha256.hexdigest() == "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"; _ledger.append(1)
assert _h_sha256.digest_size == 32; _ledger.append(1)
assert _h_sha256.name == "sha256"; _ledger.append(1)

# 7) hashlib — md5 + sha1
assert hashlib.md5(b"abc").hexdigest() == "900150983cd24fb0d6963f7d28e17f72"; _ledger.append(1)
assert hashlib.sha1(b"abc").hexdigest() == "a9993e364706816aba3e25717850c26c9cd0d89d"; _ledger.append(1)

# 8) hashlib — new(name) factory + update
_h_new = hashlib.new("sha256")
_h_new.update(b"hello")
assert _h_new.hexdigest() == "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"; _ledger.append(1)

# 9) hashlib — incremental update produces the same digest
_h_inc = hashlib.sha256()
_h_inc.update(b"hel")
_h_inc.update(b"lo")
assert _h_inc.hexdigest() == "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"; _ledger.append(1)

# NB: statistics.mode on a str-bag returns None instead of the
# modal element, decimal.getcontext / ROUND_HALF_UP / ROUND_DOWN
# absent, Decimal arithmetic / str / repr / equality / quantize
# all broken (int-handle pattern), Fraction arithmetic returns
# garbage int-handles, Fraction string and float constructors
# return garbage, limit_denominator broken — all DIVERGE on
# mamba — moved to the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_statistics_hashlib_value_ops {sum(_ledger)} asserts")
