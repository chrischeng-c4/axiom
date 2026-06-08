# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pow"
# dimension = "behavior"
# case = "pow_test__test_big_exp"
# subject = "cpython.test_pow.PowTest.test_big_exp"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pow.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pow.py::PowTest::test_big_exp
"""Auto-ported test: PowTest::test_big_exp (CPython 3.12 oracle)."""


import math
import unittest


# --- test body ---
import random

assert pow(2, 50000) == 1 << 50000
prime = 1000000000039
for i in range(10):
    a = random.randrange(1000, 1000000)
    bpower = random.randrange(1000, 50000)
    b = random.randrange(1 << bpower - 1, 1 << bpower)
    b1 = random.randrange(1, b)
    b2 = b - b1
    got1 = pow(a, b, prime)
    got2 = pow(a, b1, prime) * pow(a, b2, prime) % prime
    if got1 != got2:

        raise AssertionError(f'a={a:x} b1={b1:x} b2={b2:x} got1={got1:x} got2={got2:x}')
    got3 = pow(a, b1 * b2, prime)
    got4 = pow(pow(a, b1, prime), b2, prime)
    if got3 != got4:

        raise AssertionError(f'a={a:x} b1={b1:x} b2={b2:x} got3={got3:x} got4={got4:x}')
print("PowTest::test_big_exp: ok")
