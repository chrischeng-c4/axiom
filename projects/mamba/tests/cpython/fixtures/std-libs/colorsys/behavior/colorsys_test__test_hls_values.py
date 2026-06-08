# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "colorsys"
# dimension = "behavior"
# case = "colorsys_test__test_hls_values"
# subject = "cpython.test_colorsys.ColorsysTest.test_hls_values"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_colorsys.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_colorsys.py::ColorsysTest::test_hls_values
"""Auto-ported test: ColorsysTest::test_hls_values (CPython 3.12 oracle)."""


import unittest
import colorsys


def frange(start, stop, step):
    while start <= stop:
        yield start
        start += step


# --- test body ---
def assertTripleEqual(tr1, tr2):

    assert len(tr1) == 3

    assert len(tr2) == 3

    assert abs(tr1[0] - tr2[0]) < 1e-07

    assert abs(tr1[1] - tr2[1]) < 1e-07

    assert abs(tr1[2] - tr2[2]) < 1e-07
values = [((0.0, 0.0, 0.0), (0, 0.0, 0.0)), ((0.0, 0.0, 1.0), (4.0 / 6.0, 0.5, 1.0)), ((0.0, 1.0, 0.0), (2.0 / 6.0, 0.5, 1.0)), ((0.0, 1.0, 1.0), (3.0 / 6.0, 0.5, 1.0)), ((1.0, 0.0, 0.0), (0, 0.5, 1.0)), ((1.0, 0.0, 1.0), (5.0 / 6.0, 0.5, 1.0)), ((1.0, 1.0, 0.0), (1.0 / 6.0, 0.5, 1.0)), ((1.0, 1.0, 1.0), (0, 1.0, 0.0)), ((0.5, 0.5, 0.5), (0, 0.5, 0.0))]
for rgb, hls in values:
    assertTripleEqual(hls, colorsys.rgb_to_hls(*rgb))
    assertTripleEqual(rgb, colorsys.hls_to_rgb(*hls))
print("ColorsysTest::test_hls_values: ok")
