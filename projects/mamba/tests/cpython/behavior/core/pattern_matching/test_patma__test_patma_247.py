# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_247"
# subject = "cpython.test_patma.TestPatma.test_patma_247"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_247
"""Auto-ported test: TestPatma::test_patma_247 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def f(x):
    match x:
        case [y, [a, b, c, d, e, f, g, h, i, 9] | [h, g, i, a, b, d, e, c, f, 10] | [g, b, a, c, d, -5, e, h, i, f] | [-1, d, f, b, g, e, i, a, h, c], z]:
            w = 0
    out = locals()
    del out['x']
    return out
alts = [dict(a=0, b=1, c=2, d=3, e=4, f=5, g=6, h=7, i=8, w=0, y=False, z=True), dict(h=1, g=2, i=3, a=4, b=5, d=6, e=7, c=8, f=9, w=0, y=False, z=True), dict(g=0, b=-1, a=-2, c=-3, d=-4, e=-6, h=-7, i=-8, f=-9, w=0, y=False, z=True), dict(d=-2, f=-3, b=-4, g=-5, e=-6, i=-7, a=-8, h=-9, c=-10, w=0, y=False, z=True), dict()]

assert f((False, range(10), True)) == alts[0]

assert f((False, range(1, 11), True)) == alts[1]

assert f((False, range(0, -10, -1), True)) == alts[2]

assert f((False, range(-1, -11, -1), True)) == alts[3]

assert f((False, range(10, 20), True)) == alts[4]
print("TestPatma::test_patma_247: ok")
