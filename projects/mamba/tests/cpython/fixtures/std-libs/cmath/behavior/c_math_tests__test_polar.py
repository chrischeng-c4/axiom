# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "c_math_tests__test_polar"
# subject = "cpython.test_cmath.CMathTests.test_polar"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_cmath.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_cmath.py::CMathTests::test_polar
"""Auto-ported test: CMathTests::test_polar (CPython 3.12 oracle)."""


import math
from cmath import pi, polar


def r_assert_almost_equal(a, b, rel_err=2e-15, abs_err=5e-323):
    if math.isnan(a):
        assert math.isnan(b), f"{b!r} should be nan"
        return

    if math.isinf(a):
        assert a == b, f"finite result where infinity expected: expected {a!r}, got {b!r}"
        return

    if not a and not b:
        assert math.copysign(1.0, a) == math.copysign(1.0, b), (
            f"zero has wrong sign: expected {a!r}, got {b!r}"
        )

    try:
        absolute_error = abs(b - a)
    except OverflowError:
        pass
    else:
        if absolute_error <= max(abs_err, rel_err * abs(a)):
            return

    raise AssertionError(f"{a!r} and {b!r} are not sufficiently close")


def check(arg, expected):
    got = polar(arg)
    for e, g in zip(expected, got):
        r_assert_almost_equal(e, g)


check(0, (0.0, 0.0))
check(1, (1.0, 0.0))
check(-1, (1.0, pi))
check(1j, (1.0, pi / 2))
check(-3j, (3.0, -pi / 2))
inf = float("inf")
check(complex(inf, 0), (inf, 0.0))
check(complex(-inf, 0), (inf, pi))
check(complex(3, inf), (inf, pi / 2))
check(complex(5, -inf), (inf, -pi / 2))
check(complex(inf, inf), (inf, pi / 4))
check(complex(inf, -inf), (inf, -pi / 4))
check(complex(-inf, inf), (inf, 3 * pi / 4))
check(complex(-inf, -inf), (inf, -3 * pi / 4))
nan = float("nan")
check(complex(nan, 0), (nan, nan))
check(complex(0, nan), (nan, nan))
check(complex(nan, nan), (nan, nan))
check(complex(inf, nan), (inf, nan))
check(complex(-inf, nan), (inf, nan))
check(complex(nan, inf), (inf, nan))
check(complex(nan, -inf), (inf, nan))

print("CMathTests::test_polar: ok")
