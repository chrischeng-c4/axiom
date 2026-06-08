# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pow"
# dimension = "behavior"
# case = "pow_test__test_negative_exponent"
# subject = "cpython.test_pow.PowTest.test_negative_exponent"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pow.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pow.py::PowTest::test_negative_exponent
"""Auto-ported test: PowTest::test_negative_exponent (CPython 3.12 oracle)."""


import math
import unittest


# --- test body ---
def powtest(type):
    if type != float:
        for i in range(-1000, 1000):

            assert pow(type(i), 0) == 1

            assert pow(type(i), 1) == type(i)

            assert pow(type(0), 1) == type(0)

            assert pow(type(1), 1) == type(1)
        for i in range(-100, 100):

            assert pow(type(i), 3) == i * i * i
        pow2 = 1
        for i in range(0, 31):

            assert pow(2, i) == pow2
            if i != 30:
                pow2 = pow2 * 2
        for i in list(range(-10, 0)) + list(range(1, 10)):
            ii = type(i)
            inv = pow(ii, -1)
            for jj in range(-10, 0):

                assert abs(pow(ii, jj) - pow(inv, -jj)) < 1e-07
    for othertype in (int, float):
        for i in range(1, 100):
            zero = type(0)
            exp = -othertype(i / 10.0)
            if exp == 0:
                continue

            try:
                pow(zero, exp)
                raise AssertionError('expected ZeroDivisionError')
            except ZeroDivisionError:
                pass
    il, ih = (-20, 20)
    jl, jh = (-5, 5)
    kl, kh = (-10, 10)
    asseq = self_assertEqual
    if type == float:
        il = 1
        asseq = self_assertAlmostEqual
    elif type == int:
        jl = 0
    elif type == int:
        jl, jh = (0, 15)
    for i in range(il, ih + 1):
        for j in range(jl, jh + 1):
            for k in range(kl, kh + 1):
                if k != 0:
                    if type == float or j < 0:

                        try:
                            pow(type(i), j, k)
                            raise AssertionError('expected TypeError')
                        except TypeError:
                            pass
                        continue
                    asseq(pow(type(i), j, k), pow(type(i), j) % type(k))
for a in range(-50, 50):
    for m in range(-50, 50):
        if m != 0 and math.gcd(a, m) == 1:
            inv = pow(a, -1, m)

            assert inv == inv % m

            assert (inv * a - 1) % m == 0

            assert pow(a, -2, m) == pow(inv, 2, m)

            assert pow(a, -3, m) == pow(inv, 3, m)

            assert pow(a, -1001, m) == pow(inv, 1001, m)
        else:
            try:
                pow(a, -1, m)
                raise AssertionError('expected ValueError')
            except ValueError:
                pass
            try:
                pow(a, -2, m)
                raise AssertionError('expected ValueError')
            except ValueError:
                pass
            try:
                pow(a, -1001, m)
                raise AssertionError('expected ValueError')
            except ValueError:
                pass
print("PowTest::test_negative_exponent: ok")
