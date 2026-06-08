# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_pow_modular_divmod_ops"
# subject = "cpython321.test_pow_modular_divmod_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_pow_modular_divmod_ops.py"
# status = "filled"
# ///
"""cpython321.test_pow_modular_divmod_ops: execute CPython 3.12 seed test_pow_modular_divmod_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for pow() modular forms, modular
# inverse (Py3.8+), and divmod/floor-div/modulo sign convention.
# Surface:
#   • pow(0, 0) == 1 — Python convention for the indeterminate form;
#   • pow(a, 0) == 1 for any nonzero a;
#   • pow(a, e) for negative exponent on int yields float (`pow(2,-1)
#     == 0.5`);
#   • float base: pow(2.0, 3) == 8.0;
#   • negative base, even/odd exponent: pow(-2,2)==4, pow(-2,3)==-8;
#   • three-arg pow(a, e, m) — modular exponent for large e;
#   • pow(a, -1, m) — modular inverse for gcd(a,m)==1 (Py3.8+);
#   • Fermat's little theorem: pow(a, p-1, p) == 1 for prime p with
#     gcd(a,p)==1 (verified at p=7 and p=11);
#   • divmod sign convention: divmod(-17, 5) == (-4, 3),
#     divmod(17, -5) == (-4, -3), divmod(-17, -5) == (3, -2) —
#     Python's floor-division + positive-modulus-w.r.t.-divisor-sign;
#   • identity: a == b*q + r where (q, r) = divmod(a, b) for all
#     sign combinations;
#   • // and % agree with divmod elementwise;
#   • float divmod: divmod(17.0, 5.0) == (3.0, 2.0).
_ledger: list[int] = []

# pow corner cases
assert pow(0, 0) == 1; _ledger.append(1)
assert pow(5, 0) == 1; _ledger.append(1)
assert pow(1, 100) == 1; _ledger.append(1)
assert pow(2, 10) == 1024; _ledger.append(1)
assert pow(2, -1) == 0.5; _ledger.append(1)
assert pow(2, -2) == 0.25; _ledger.append(1)
assert pow(4, -2) == 0.0625; _ledger.append(1)

# pow with float base
assert pow(2.0, 3) == 8.0; _ledger.append(1)
assert pow(2.0, -1) == 0.5; _ledger.append(1)
assert pow(9.0, 0.5) == 3.0; _ledger.append(1)  # sqrt

# negative base, even/odd exponent
assert pow(-2, 3) == -8; _ledger.append(1)
assert pow(-2, 2) == 4; _ledger.append(1)
assert pow(-3, 4) == 81; _ledger.append(1)
assert pow(-3, 5) == -243; _ledger.append(1)

# three-arg pow(a, e, m) — modular exponent
assert pow(2, 10, 1000) == 24; _ledger.append(1)
assert pow(3, 100, 7) == 4; _ledger.append(1)
assert pow(7, 100, 13) == 9; _ledger.append(1)

# Modular inverse (Py3.8+): pow(a, -1, m) where gcd(a,m)==1
assert pow(2, -1, 5) == 3; _ledger.append(1)   # 2*3=6 ≡ 1 (mod 5)
assert pow(3, -1, 7) == 5; _ledger.append(1)   # 3*5=15 ≡ 1 (mod 7)
assert pow(7, -1, 11) == 8; _ledger.append(1)  # 7*8=56 ≡ 1 (mod 11)
assert pow(10, -1, 17) == 12; _ledger.append(1)  # 10*12=120 ≡ 1 (mod 17)

# Inverse property: a * inv(a) ≡ 1 (mod m)
_inv = pow(7, -1, 11)
assert (7 * _inv) % 11 == 1; _ledger.append(1)
_inv2 = pow(13, -1, 100)
assert (13 * _inv2) % 100 == 1; _ledger.append(1)

# Fermat's little theorem: a^(p-1) ≡ 1 (mod p), p prime, gcd(a,p)=1
assert pow(2, 6, 7) == 1; _ledger.append(1)
assert pow(3, 10, 11) == 1; _ledger.append(1)
assert pow(5, 12, 13) == 1; _ledger.append(1)

# divmod sign convention — Python floor-div + same-sign-as-divisor remainder
assert divmod(17, 5) == (3, 2); _ledger.append(1)
assert divmod(-17, 5) == (-4, 3); _ledger.append(1)
assert divmod(17, -5) == (-4, -3); _ledger.append(1)
assert divmod(-17, -5) == (3, -2); _ledger.append(1)

# divmod identity: a == b*q + r for (q,r) = divmod(a,b)
# (loops are unrolled because mamba 0.3.60 drops the numeric type of
# loop-unpacked tuple variables, breaking `_a // _b` / `_a % _b` at
# compile time.)
_q1, _r1 = divmod(17, 5)
assert 17 == 5 * _q1 + _r1; _ledger.append(1)
_q2, _r2 = divmod(-17, 5)
assert -17 == 5 * _q2 + _r2; _ledger.append(1)
_q3, _r3 = divmod(17, -5)
assert 17 == -5 * _q3 + _r3; _ledger.append(1)
_q4, _r4 = divmod(-17, -5)
assert -17 == -5 * _q4 + _r4; _ledger.append(1)
_q5, _r5 = divmod(100, 7)
assert 100 == 7 * _q5 + _r5; _ledger.append(1)
_q6, _r6 = divmod(-100, 7)
assert -100 == 7 * _q6 + _r6; _ledger.append(1)

# // and % consistency with divmod
assert 17 // 5 == _q1; _ledger.append(1)
assert 17 % 5 == _r1; _ledger.append(1)
assert -17 // 5 == _q2; _ledger.append(1)
assert -17 % 5 == _r2; _ledger.append(1)
assert 17 // -5 == _q3; _ledger.append(1)
assert 17 % -5 == _r3; _ledger.append(1)
assert -17 // -5 == _q4; _ledger.append(1)
assert -17 % -5 == _r4; _ledger.append(1)

# float divmod
assert divmod(17.0, 5.0) == (3.0, 2.0); _ledger.append(1)
assert divmod(7.5, 2.5) == (3.0, 0.0); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_pow_modular_divmod_ops {sum(_ledger)} asserts")
