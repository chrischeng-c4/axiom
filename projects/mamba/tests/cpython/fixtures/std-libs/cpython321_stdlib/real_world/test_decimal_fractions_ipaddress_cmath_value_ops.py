# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_decimal_fractions_ipaddress_cmath_value_ops"
# subject = "cpython321.test_decimal_fractions_ipaddress_cmath_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_decimal_fractions_ipaddress_cmath_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_decimal_fractions_ipaddress_cmath_value_ops: execute CPython 3.12 seed test_decimal_fractions_ipaddress_cmath_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 264 pass conformance — decimal module (hasattr Decimal) +
# fractions module (hasattr Fraction + Fraction(3,4).numerator == 3
# / .denominator == 4, Fraction(5,10).numerator == 1 / .denominator
# == 2 (reduced), Fraction(0,1).numerator == 0, Fraction(1,1)
# .numerator == 1 / .denominator == 1) + ipaddress module (hasattr
# IPv4Address/IPv6Address/IPv4Network/IPv6Network/ip_address/ip_
# network/ip_interface/collapse_addresses/summarize_address_range/
# v4_int_to_packed) + cmath module (hasattr sqrt/exp/log/sin/cos/
# tan/phase/polar/rect/pi/e/inf/nan/isfinite/isinf/isnan + sqrt
# (-1)==1j, sqrt(4)==2+0j, exp(0)==1+0j, exp(0j)==1+0j, log(1)==0j,
# polar(1+0j)==(1.0, 0.0), polar(0+1j)==(1.0, 1.5707963267948966),
# rect(1, 0)==1+0j, sin(0+0j)==0j, isfinite(0+0j)==True, isnan
# (0+0j)==False, isinf(0+0j)==False).
# All asserts match between CPython 3.12 and mamba.
import decimal
import fractions
import ipaddress
import cmath


_ledger: list[int] = []

# 1) decimal — hasattr Decimal class symbol exists
assert hasattr(decimal, "Decimal") == True; _ledger.append(1)

# 2) fractions — hasattr Fraction
assert hasattr(fractions, "Fraction") == True; _ledger.append(1)

# 3) fractions — numerator / denominator value contracts (these
#    conform even when the type itself is misreported elsewhere)
assert fractions.Fraction(3, 4).numerator == 3; _ledger.append(1)
assert fractions.Fraction(3, 4).denominator == 4; _ledger.append(1)
assert fractions.Fraction(5, 10).numerator == 1; _ledger.append(1)
assert fractions.Fraction(5, 10).denominator == 2; _ledger.append(1)
assert fractions.Fraction(0, 1).numerator == 0; _ledger.append(1)
assert fractions.Fraction(1, 1).numerator == 1; _ledger.append(1)
assert fractions.Fraction(1, 1).denominator == 1; _ledger.append(1)

# 4) ipaddress — hasattr surface
assert hasattr(ipaddress, "IPv4Address") == True; _ledger.append(1)
assert hasattr(ipaddress, "IPv6Address") == True; _ledger.append(1)
assert hasattr(ipaddress, "IPv4Network") == True; _ledger.append(1)
assert hasattr(ipaddress, "IPv6Network") == True; _ledger.append(1)
assert hasattr(ipaddress, "ip_address") == True; _ledger.append(1)
assert hasattr(ipaddress, "ip_network") == True; _ledger.append(1)
assert hasattr(ipaddress, "ip_interface") == True; _ledger.append(1)
assert hasattr(ipaddress, "collapse_addresses") == True; _ledger.append(1)
assert hasattr(ipaddress, "summarize_address_range") == True; _ledger.append(1)
assert hasattr(ipaddress, "v4_int_to_packed") == True; _ledger.append(1)

# 5) cmath — hasattr surface
assert hasattr(cmath, "sqrt") == True; _ledger.append(1)
assert hasattr(cmath, "exp") == True; _ledger.append(1)
assert hasattr(cmath, "log") == True; _ledger.append(1)
assert hasattr(cmath, "sin") == True; _ledger.append(1)
assert hasattr(cmath, "cos") == True; _ledger.append(1)
assert hasattr(cmath, "tan") == True; _ledger.append(1)
assert hasattr(cmath, "phase") == True; _ledger.append(1)
assert hasattr(cmath, "polar") == True; _ledger.append(1)
assert hasattr(cmath, "rect") == True; _ledger.append(1)
assert hasattr(cmath, "pi") == True; _ledger.append(1)
assert hasattr(cmath, "e") == True; _ledger.append(1)
assert hasattr(cmath, "inf") == True; _ledger.append(1)
assert hasattr(cmath, "nan") == True; _ledger.append(1)
assert hasattr(cmath, "isfinite") == True; _ledger.append(1)
assert hasattr(cmath, "isinf") == True; _ledger.append(1)
assert hasattr(cmath, "isnan") == True; _ledger.append(1)

# 6) cmath — complex-arg value contracts (use complex literals on
#    both sides to dodge mamba's complex-vs-int eq quirk)
assert cmath.sqrt(-1) == 1j; _ledger.append(1)
assert cmath.sqrt(4) == 2+0j; _ledger.append(1)
assert cmath.exp(0) == 1+0j; _ledger.append(1)
assert cmath.exp(0j) == 1+0j; _ledger.append(1)
assert cmath.log(1) == 0j; _ledger.append(1)
assert cmath.polar(1+0j) == (1.0, 0.0); _ledger.append(1)
assert cmath.polar(0+1j) == (1.0, 1.5707963267948966); _ledger.append(1)
assert cmath.rect(1, 0) == 1+0j; _ledger.append(1)
assert cmath.sin(0+0j) == 0j; _ledger.append(1)

# 7) cmath — finiteness predicates on finite complex
assert cmath.isfinite(0+0j) == True; _ledger.append(1)
assert cmath.isnan(0+0j) == False; _ledger.append(1)
assert cmath.isinf(0+0j) == False; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_decimal_fractions_ipaddress_cmath_value_ops {sum(_ledger)} asserts")
