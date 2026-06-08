# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_decimal_fractions_ipaddress_cmath_silent"
# subject = "cpython321.lang_decimal_fractions_ipaddress_cmath_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_decimal_fractions_ipaddress_cmath_silent.py"
# status = "filled"
# ///
"""cpython321.lang_decimal_fractions_ipaddress_cmath_silent: execute CPython 3.12 seed lang_decimal_fractions_ipaddress_cmath_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `str(decimal.Decimal('1.5'))` (the
# documented "Decimal('1.5') stringifies as '1.5'" — mamba returns
# an opaque integer-handle string e.g. '70368744177664', revealing
# that Decimal is a thin int-handle wrapper), `hasattr(decimal,
# 'getcontext')` (the documented "decimal exposes the context-
# management API getcontext()/setcontext()" — mamba returns False),
# `type(decimal.Decimal('1')).__name__` (the documented "Decimal
# constructor returns a Decimal instance" — mamba returns 'int'),
# `decimal.Decimal('0.1') + decimal.Decimal('0.2') == decimal
# .Decimal('0.3')` (the documented "Decimal arithmetic is exact —
# 0.1 + 0.2 == 0.3 by value" — mamba returns False because the
# operands are unrelated int handles), `str(fractions.Fraction(1,
# 2))` (the documented "Fraction(1,2) stringifies as '1/2'" — mamba
# returns an opaque integer-handle string), `type(fractions
# .Fraction(1, 2)).__name__` (the documented "Fraction constructor
# returns a Fraction instance" — mamba returns 'int'), `str
# (ipaddress.ip_address('192.168.1.1'))` (the documented
# "ip_address stringifies to its dotted-quad form" — mamba returns
# an opaque integer-handle string), `ipaddress.IPv4Network('192
# .168.1.0/24').num_addresses` (the documented "IPv4Network exposes
# num_addresses == 256 for a /24" — mamba raises AttributeError
# because IPv4Network is a plain dict), `cmath.pi == 3.14159265
# 3589793` (the documented "cmath exposes pi as a float constant"
# — mamba returns False because cmath.pi is an opaque int holding
# the IEEE-754 bit pattern), and `cmath.phase(1j) == 1.5707963
# 267948966` (the documented "phase(1j) returns the angle as a
# float — pi/2" — mamba returns False because phase result is an
# opaque int).
# Ten-pack pinned to atomic 264.
#
# Behavioral edges that CONFORM on mamba (decimal — hasattr Decimal.
# fractions — hasattr Fraction + Fraction(3,4).numerator/denominator,
# Fraction(5,10) reduced to 1/2, Fraction(0,1)/(1,1) numerator/
# denominator value contracts. ipaddress — hasattr IPv4Address/
# IPv6Address/IPv4Network/IPv6Network/ip_address/ip_network/
# ip_interface/collapse_addresses/summarize_address_range/v4_int_
# to_packed surface. cmath — hasattr sqrt/exp/log/sin/cos/tan/
# phase/polar/rect/pi/e/inf/nan/isfinite/isinf/isnan + complex-arg
# value contracts sqrt(-1)/sqrt(4)/exp(0)/exp(0j)/log(1)/polar
# (1+0j)/polar(0+1j)/rect(1,0)/sin(0+0j) and finiteness predicates
# isfinite/isnan/isinf on 0+0j) are covered in the matching pass
# fixture `test_decimal_fractions_ipaddress_cmath_value_ops`.
import decimal
import fractions
import ipaddress
import cmath
from typing import Any


_ledger: list[int] = []

# 1) str(decimal.Decimal('1.5')) == '1.5'
#    (mamba: returns opaque int-handle string)
assert str(decimal.Decimal("1.5")) == "1.5"; _ledger.append(1)

# 2) hasattr(decimal, 'getcontext') — context-management API
#    (mamba: returns False)
assert hasattr(decimal, "getcontext") == True; _ledger.append(1)

# 3) type(decimal.Decimal('1')).__name__ == 'Decimal'
#    (mamba: returns 'int')
assert type(decimal.Decimal("1")).__name__ == "Decimal"; _ledger.append(1)

# 4) Decimal('0.1') + Decimal('0.2') == Decimal('0.3')
#    (mamba: returns False — int handles don't satisfy decimal
#     arithmetic equality)
assert (decimal.Decimal("0.1") + decimal.Decimal("0.2")) == decimal.Decimal("0.3"); _ledger.append(1)

# 5) str(fractions.Fraction(1, 2)) == '1/2'
#    (mamba: returns opaque int-handle string)
assert str(fractions.Fraction(1, 2)) == "1/2"; _ledger.append(1)

# 6) type(fractions.Fraction(1, 2)).__name__ == 'Fraction'
#    (mamba: returns 'int')
assert type(fractions.Fraction(1, 2)).__name__ == "Fraction"; _ledger.append(1)

# 7) str(ipaddress.ip_address('192.168.1.1')) == '192.168.1.1'
#    (mamba: returns opaque int-handle string)
assert str(ipaddress.ip_address("192.168.1.1")) == "192.168.1.1"; _ledger.append(1)

# 8) ipaddress.IPv4Network('192.168.1.0/24').num_addresses == 256
#    (mamba: AttributeError — IPv4Network is plain dict)
def _ipv4_num_addresses() -> Any:
    try:
        return ipaddress.IPv4Network("192.168.1.0/24").num_addresses
    except AttributeError:
        return None
assert _ipv4_num_addresses() == 256; _ledger.append(1)

# 9) cmath.pi == 3.141592653589793
#    (mamba: returns False — cmath.pi is opaque int of bit pattern)
assert cmath.pi == 3.141592653589793; _ledger.append(1)

# 10) cmath.phase(1j) == 1.5707963267948966
#     (mamba: returns False — phase result is opaque int)
assert cmath.phase(1j) == 1.5707963267948966; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_decimal_fractions_ipaddress_cmath_silent {sum(_ledger)} asserts")
