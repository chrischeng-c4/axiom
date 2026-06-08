# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "colorsys"
# dimension = "behavior"
# case = "colorsys_test__test_yiq_values"
# subject = "cpython.test_colorsys.ColorsysTest.test_yiq_values"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_colorsys.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_colorsys.py::ColorsysTest::test_yiq_values
"""Auto-ported test: ColorsysTest::test_yiq_values (CPython 3.12 oracle)."""


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
values = [((0.0, 0.0, 0.0), (0.0, 0.0, 0.0)), ((0.0, 0.0, 1.0), (0.11, -0.3217, 0.3121)), ((0.0, 1.0, 0.0), (0.59, -0.2773, -0.5251)), ((0.0, 1.0, 1.0), (0.7, -0.599, -0.213)), ((1.0, 0.0, 0.0), (0.3, 0.599, 0.213)), ((1.0, 0.0, 1.0), (0.41, 0.2773, 0.5251)), ((1.0, 1.0, 0.0), (0.89, 0.3217, -0.3121)), ((1.0, 1.0, 1.0), (1.0, 0.0, 0.0)), ((0.5, 0.5, 0.5), (0.5, 0.0, 0.0))]
for rgb, yiq in values:
    assertTripleEqual(yiq, colorsys.rgb_to_yiq(*rgb))
    assertTripleEqual(rgb, colorsys.yiq_to_rgb(*yiq))
print("ColorsysTest::test_yiq_values: ok")
