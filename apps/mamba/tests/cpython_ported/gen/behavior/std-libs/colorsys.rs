use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/colorsys/colorsys_test__test_hls_nearwhite.py`.
#[test]
fn test_gen_behavior_std_libs_colorsys_colorsys_test__test_hls_nearwhite() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "colorsys"
# dimension = "behavior"
# case = "colorsys_test__test_hls_nearwhite"
# subject = "cpython.test_colorsys.ColorsysTest.test_hls_nearwhite"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_colorsys.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_colorsys.py::ColorsysTest::test_hls_nearwhite
"""Auto-ported test: ColorsysTest::test_hls_nearwhite (CPython 3.12 oracle)."""


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
values = (((0.9999999999999999, 1, 1), (0.5, 1.0, 1.0)), ((1, 0.9999999999999999, 0.9999999999999999), (0.0, 1.0, 1.0)))
for rgb, hls in values:
    assertTripleEqual(hls, colorsys.rgb_to_hls(*rgb))
    assertTripleEqual((1.0, 1.0, 1.0), colorsys.hls_to_rgb(*hls))
print("ColorsysTest::test_hls_nearwhite: ok")
"###);
    assert_output(&out, r###"ColorsysTest::test_hls_nearwhite: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/colorsys/colorsys_test__test_hls_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_colorsys_colorsys_test__test_hls_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "colorsys"
# dimension = "behavior"
# case = "colorsys_test__test_hls_roundtrip"
# subject = "cpython.test_colorsys.ColorsysTest.test_hls_roundtrip"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_colorsys.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_colorsys.py::ColorsysTest::test_hls_roundtrip
"""Auto-ported test: ColorsysTest::test_hls_roundtrip (CPython 3.12 oracle)."""


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
for r in frange(0.0, 1.0, 0.2):
    for g in frange(0.0, 1.0, 0.2):
        for b in frange(0.0, 1.0, 0.2):
            rgb = (r, g, b)
            assertTripleEqual(rgb, colorsys.hls_to_rgb(*colorsys.rgb_to_hls(*rgb)))
print("ColorsysTest::test_hls_roundtrip: ok")
"###);
    assert_output(&out, r###"ColorsysTest::test_hls_roundtrip: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/colorsys/colorsys_test__test_hls_values.py`.
#[test]
fn test_gen_behavior_std_libs_colorsys_colorsys_test__test_hls_values() {
    let out = jit_capture(r###"# /// script
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
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_colorsys.py"
# status = "filled"
# ///
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
"###);
    assert_output(&out, r###"ColorsysTest::test_hls_values: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/colorsys/colorsys_test__test_hsv_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_colorsys_colorsys_test__test_hsv_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "colorsys"
# dimension = "behavior"
# case = "colorsys_test__test_hsv_roundtrip"
# subject = "cpython.test_colorsys.ColorsysTest.test_hsv_roundtrip"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_colorsys.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_colorsys.py::ColorsysTest::test_hsv_roundtrip
"""Auto-ported test: ColorsysTest::test_hsv_roundtrip (CPython 3.12 oracle)."""


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
for r in frange(0.0, 1.0, 0.2):
    for g in frange(0.0, 1.0, 0.2):
        for b in frange(0.0, 1.0, 0.2):
            rgb = (r, g, b)
            assertTripleEqual(rgb, colorsys.hsv_to_rgb(*colorsys.rgb_to_hsv(*rgb)))
print("ColorsysTest::test_hsv_roundtrip: ok")
"###);
    assert_output(&out, r###"ColorsysTest::test_hsv_roundtrip: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/colorsys/colorsys_test__test_hsv_values.py`.
#[test]
fn test_gen_behavior_std_libs_colorsys_colorsys_test__test_hsv_values() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "colorsys"
# dimension = "behavior"
# case = "colorsys_test__test_hsv_values"
# subject = "cpython.test_colorsys.ColorsysTest.test_hsv_values"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_colorsys.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_colorsys.py::ColorsysTest::test_hsv_values
"""Auto-ported test: ColorsysTest::test_hsv_values (CPython 3.12 oracle)."""


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
values = [((0.0, 0.0, 0.0), (0, 0.0, 0.0)), ((0.0, 0.0, 1.0), (4.0 / 6.0, 1.0, 1.0)), ((0.0, 1.0, 0.0), (2.0 / 6.0, 1.0, 1.0)), ((0.0, 1.0, 1.0), (3.0 / 6.0, 1.0, 1.0)), ((1.0, 0.0, 0.0), (0, 1.0, 1.0)), ((1.0, 0.0, 1.0), (5.0 / 6.0, 1.0, 1.0)), ((1.0, 1.0, 0.0), (1.0 / 6.0, 1.0, 1.0)), ((1.0, 1.0, 1.0), (0, 0.0, 1.0)), ((0.5, 0.5, 0.5), (0, 0.0, 0.5))]
for rgb, hsv in values:
    assertTripleEqual(hsv, colorsys.rgb_to_hsv(*rgb))
    assertTripleEqual(rgb, colorsys.hsv_to_rgb(*hsv))
print("ColorsysTest::test_hsv_values: ok")
"###);
    assert_output(&out, r###"ColorsysTest::test_hsv_values: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/colorsys/colorsys_test__test_yiq_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_colorsys_colorsys_test__test_yiq_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "colorsys"
# dimension = "behavior"
# case = "colorsys_test__test_yiq_roundtrip"
# subject = "cpython.test_colorsys.ColorsysTest.test_yiq_roundtrip"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_colorsys.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_colorsys.py::ColorsysTest::test_yiq_roundtrip
"""Auto-ported test: ColorsysTest::test_yiq_roundtrip (CPython 3.12 oracle)."""


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
for r in frange(0.0, 1.0, 0.2):
    for g in frange(0.0, 1.0, 0.2):
        for b in frange(0.0, 1.0, 0.2):
            rgb = (r, g, b)
            assertTripleEqual(rgb, colorsys.yiq_to_rgb(*colorsys.rgb_to_yiq(*rgb)))
print("ColorsysTest::test_yiq_roundtrip: ok")
"###);
    assert_output(&out, r###"ColorsysTest::test_yiq_roundtrip: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/colorsys/colorsys_test__test_yiq_values.py`.
#[test]
fn test_gen_behavior_std_libs_colorsys_colorsys_test__test_yiq_values() {
    let out = jit_capture(r###"# /// script
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
"###);
    assert_output(&out, r###"ColorsysTest::test_yiq_values: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/colorsys/hls_nearwhite_one_way.py`.
#[test]
fn test_gen_behavior_std_libs_colorsys_hls_nearwhite_one_way() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "colorsys"
# dimension = "behavior"
# case = "hls_nearwhite_one_way"
# subject = "colorsys.rgb_to_hls"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_colorsys.py"
# status = "filled"
# ///
"""colorsys.rgb_to_hls: gh-106498 near-white inputs convert forward to a stable HLS even though the inverse is not exact: rgb_to_hls(0.9999999999999999,1,1)==(0.5,1.0,1.0) and hls_to_rgb of that is (1.0,1.0,1.0)"""
import colorsys

EPS = 1e-9
# These do not round-trip in reverse (gh-106498); only the forward and the
# canonical inverse are stable.
cases = [
    ((0.9999999999999999, 1, 1), (0.5, 1.0, 1.0)),
    ((1, 0.9999999999999999, 0.9999999999999999), (0.0, 1.0, 1.0)),
]
for rgb, hls in cases:
    fwd = colorsys.rgb_to_hls(*rgb)
    for got, want in zip(fwd, hls):
        assert abs(got - want) < EPS, ("forward", rgb, fwd)
    back = colorsys.hls_to_rgb(*hls)
    for got in back:
        assert abs(got - 1.0) < EPS, ("inverse", hls, back)

print("hls_nearwhite_one_way OK")
"###);
    assert_output(&out, r###"hls_nearwhite_one_way OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/colorsys/hls_primary_lightness_half.py`.
#[test]
fn test_gen_behavior_std_libs_colorsys_hls_primary_lightness_half() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "colorsys"
# dimension = "behavior"
# case = "hls_primary_lightness_half"
# subject = "colorsys.rgb_to_hls"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""colorsys.rgb_to_hls: each fully-saturated primary color (red/green/blue) has HLS lightness exactly 0.5"""
import colorsys

EPS = 1e-9
for r, g, b in [(1, 0, 0), (0, 1, 0), (0, 0, 1)]:
    h, l, s = colorsys.rgb_to_hls(r, g, b)
    assert abs(l - 0.5) < EPS, ("primary lightness", (r, g, b), l)

print("hls_primary_lightness_half OK")
"###);
    assert_output(&out, r###"hls_primary_lightness_half OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/colorsys/hls_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_colorsys_hls_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "colorsys"
# dimension = "behavior"
# case = "hls_roundtrip"
# subject = "colorsys.rgb_to_hls"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_colorsys.py"
# status = "filled"
# ///
"""colorsys.rgb_to_hls: rgb_to_hls then hls_to_rgb recovers the original RGB across a representative color table"""
import colorsys

EPS = 1e-9
colors = [
    (1.0, 0.0, 0.0),  # red
    (0.0, 1.0, 0.0),  # green
    (0.0, 0.0, 1.0),  # blue
    (0.5, 0.5, 0.5),  # gray
    (0.2, 0.4, 0.8),  # arbitrary
]
for r, g, b in colors:
    h, l, s = colorsys.rgb_to_hls(r, g, b)
    r2, g2, b2 = colorsys.hls_to_rgb(h, l, s)
    assert abs(r2 - r) < EPS, ("r", r, r2)
    assert abs(g2 - g) < EPS, ("g", g, g2)
    assert abs(b2 - b) < EPS, ("b", b, b2)

print("hls_roundtrip OK")
"###);
    assert_output(&out, r###"hls_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/colorsys/hls_value_table.py`.
#[test]
fn test_gen_behavior_std_libs_colorsys_hls_value_table() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "colorsys"
# dimension = "behavior"
# case = "hls_value_table"
# subject = "colorsys.rgb_to_hls"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_colorsys.py"
# status = "filled"
# ///
"""colorsys.rgb_to_hls: the canonical CPython RGB<->HLS value table (black/blue/green/cyan/red/purple/yellow/white/grey) matches in both directions"""
import colorsys

EPS = 1e-7
# (rgb, hls) — straight from CPython's test_colorsys.test_hls_values.
table = [
    ((0.0, 0.0, 0.0), (0.0, 0.0, 0.0)),         # black
    ((0.0, 0.0, 1.0), (4. / 6., 0.5, 1.0)),     # blue
    ((0.0, 1.0, 0.0), (2. / 6., 0.5, 1.0)),     # green
    ((0.0, 1.0, 1.0), (3. / 6., 0.5, 1.0)),     # cyan
    ((1.0, 0.0, 0.0), (0.0, 0.5, 1.0)),         # red
    ((1.0, 0.0, 1.0), (5. / 6., 0.5, 1.0)),     # purple
    ((1.0, 1.0, 0.0), (1. / 6., 0.5, 1.0)),     # yellow
    ((1.0, 1.0, 1.0), (0.0, 1.0, 0.0)),         # white
    ((0.5, 0.5, 0.5), (0.0, 0.5, 0.0)),         # grey
]
for rgb, hls in table:
    got = colorsys.rgb_to_hls(*rgb)
    for a, b in zip(got, hls):
        assert abs(a - b) < EPS, ("rgb->hls", rgb, got)
    back = colorsys.hls_to_rgb(*hls)
    for a, b in zip(back, rgb):
        assert abs(a - b) < EPS, ("hls->rgb", hls, back)

print("hls_value_table OK")
"###);
    assert_output(&out, r###"hls_value_table OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/colorsys/hsv_gray_zero_saturation.py`.
#[test]
fn test_gen_behavior_std_libs_colorsys_hsv_gray_zero_saturation() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "colorsys"
# dimension = "behavior"
# case = "hsv_gray_zero_saturation"
# subject = "colorsys.rgb_to_hsv"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""colorsys.rgb_to_hsv: any gray (r==g==b) has HSV saturation 0 across the [0,1] value range"""
import colorsys

EPS = 1e-9
for v in [0.0, 0.25, 0.5, 0.75, 1.0]:
    h, s, val = colorsys.rgb_to_hsv(v, v, v)
    assert abs(s) < EPS, ("gray saturation", v, s)

print("hsv_gray_zero_saturation OK")
"###);
    assert_output(&out, r###"hsv_gray_zero_saturation OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/colorsys/hsv_primary_hues.py`.
#[test]
fn test_gen_behavior_std_libs_colorsys_hsv_primary_hues() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "colorsys"
# dimension = "behavior"
# case = "hsv_primary_hues"
# subject = "colorsys.rgb_to_hsv"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""colorsys.rgb_to_hsv: the HSV hue of pure red is 0, pure green is 1/3, pure blue is 2/3"""
import colorsys

EPS = 1e-9
assert abs(colorsys.rgb_to_hsv(1, 0, 0)[0] - 0.0) < EPS, "red hue"
assert abs(colorsys.rgb_to_hsv(0, 1, 0)[0] - 1.0 / 3) < EPS, "green hue"
assert abs(colorsys.rgb_to_hsv(0, 0, 1)[0] - 2.0 / 3) < EPS, "blue hue"

print("hsv_primary_hues OK")
"###);
    assert_output(&out, r###"hsv_primary_hues OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/colorsys/hsv_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_colorsys_hsv_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "colorsys"
# dimension = "behavior"
# case = "hsv_roundtrip"
# subject = "colorsys.rgb_to_hsv"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_colorsys.py"
# status = "filled"
# ///
"""colorsys.rgb_to_hsv: rgb_to_hsv then hsv_to_rgb recovers the original RGB across a representative color table (red/green/blue/gray/arbitrary)"""
import colorsys

EPS = 1e-9
colors = [
    (1.0, 0.0, 0.0),  # red
    (0.0, 1.0, 0.0),  # green
    (0.0, 0.0, 1.0),  # blue
    (0.5, 0.5, 0.5),  # gray
    (0.2, 0.4, 0.8),  # arbitrary
]
for r, g, b in colors:
    h, s, v = colorsys.rgb_to_hsv(r, g, b)
    r2, g2, b2 = colorsys.hsv_to_rgb(h, s, v)
    assert abs(r2 - r) < EPS, ("r", r, r2)
    assert abs(g2 - g) < EPS, ("g", g, g2)
    assert abs(b2 - b) < EPS, ("b", b, b2)

print("hsv_roundtrip OK")
"###);
    assert_output(&out, r###"hsv_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/colorsys/hsv_value_table.py`.
#[test]
fn test_gen_behavior_std_libs_colorsys_hsv_value_table() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "colorsys"
# dimension = "behavior"
# case = "hsv_value_table"
# subject = "colorsys.rgb_to_hsv"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_colorsys.py"
# status = "filled"
# ///
"""colorsys.rgb_to_hsv: the canonical CPython RGB<->HSV value table (black/blue/green/cyan/red/purple/yellow/white/grey) matches in both directions"""
import colorsys

EPS = 1e-7
# (rgb, hsv) — straight from CPython's test_colorsys.test_hsv_values.
table = [
    ((0.0, 0.0, 0.0), (0.0, 0.0, 0.0)),         # black
    ((0.0, 0.0, 1.0), (4. / 6., 1.0, 1.0)),     # blue
    ((0.0, 1.0, 0.0), (2. / 6., 1.0, 1.0)),     # green
    ((0.0, 1.0, 1.0), (3. / 6., 1.0, 1.0)),     # cyan
    ((1.0, 0.0, 0.0), (0.0, 1.0, 1.0)),         # red
    ((1.0, 0.0, 1.0), (5. / 6., 1.0, 1.0)),     # purple
    ((1.0, 1.0, 0.0), (1. / 6., 1.0, 1.0)),     # yellow
    ((1.0, 1.0, 1.0), (0.0, 0.0, 1.0)),         # white
    ((0.5, 0.5, 0.5), (0.0, 0.0, 0.5)),         # grey
]
for rgb, hsv in table:
    got = colorsys.rgb_to_hsv(*rgb)
    for a, b in zip(got, hsv):
        assert abs(a - b) < EPS, ("rgb->hsv", rgb, got)
    back = colorsys.hsv_to_rgb(*hsv)
    for a, b in zip(back, rgb):
        assert abs(a - b) < EPS, ("hsv->rgb", hsv, back)

print("hsv_value_table OK")
"###);
    assert_output(&out, r###"hsv_value_table OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/colorsys/hsv_white_black_extremes.py`.
#[test]
fn test_gen_behavior_std_libs_colorsys_hsv_white_black_extremes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "colorsys"
# dimension = "behavior"
# case = "hsv_white_black_extremes"
# subject = "colorsys.rgb_to_hsv"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""colorsys.rgb_to_hsv: white (1,1,1) maps to HSV saturation 0 / value 1 and black (0,0,0) maps to value 0"""
import colorsys

EPS = 1e-9
wh, ws, wv = colorsys.rgb_to_hsv(1.0, 1.0, 1.0)
assert abs(ws) < EPS, "white saturation"
assert abs(wv - 1.0) < EPS, "white value"

bh, bs, bv = colorsys.rgb_to_hsv(0.0, 0.0, 0.0)
assert abs(bv) < EPS, "black value"

print("hsv_white_black_extremes OK")
"###);
    assert_output(&out, r###"hsv_white_black_extremes OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/colorsys/yiq_red_luminance.py`.
#[test]
fn test_gen_behavior_std_libs_colorsys_yiq_red_luminance() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "colorsys"
# dimension = "behavior"
# case = "yiq_red_luminance"
# subject = "colorsys.rgb_to_yiq"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""colorsys.rgb_to_yiq: the YIQ luminance (Y) of pure red is the documented 0.3 coefficient"""
import colorsys

EPS = 1e-9
y, i, q = colorsys.rgb_to_yiq(1, 0, 0)
assert abs(y - 0.3) < EPS, ("red Y", y)

print("yiq_red_luminance OK")
"###);
    assert_output(&out, r###"yiq_red_luminance OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/colorsys/yiq_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_colorsys_yiq_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "colorsys"
# dimension = "behavior"
# case = "yiq_roundtrip"
# subject = "colorsys.rgb_to_yiq"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_colorsys.py"
# status = "filled"
# ///
"""colorsys.rgb_to_yiq: rgb_to_yiq then yiq_to_rgb recovers the original RGB across a representative color table"""
import colorsys

# YIQ involves a coarser matrix inverse, so allow a slightly wider tolerance.
EPS = 1e-7
colors = [
    (1.0, 0.0, 0.0),  # red
    (0.0, 1.0, 0.0),  # green
    (0.0, 0.0, 1.0),  # blue
    (0.5, 0.5, 0.5),  # gray
    (0.2, 0.4, 0.8),  # arbitrary
]
for r, g, b in colors:
    y, i, q = colorsys.rgb_to_yiq(r, g, b)
    r2, g2, b2 = colorsys.yiq_to_rgb(y, i, q)
    assert abs(r2 - r) < EPS, ("r", r, r2)
    assert abs(g2 - g) < EPS, ("g", g, g2)
    assert abs(b2 - b) < EPS, ("b", b, b2)

print("yiq_roundtrip OK")
"###);
    assert_output(&out, r###"yiq_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/colorsys/yiq_value_table.py`.
#[test]
fn test_gen_behavior_std_libs_colorsys_yiq_value_table() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "colorsys"
# dimension = "behavior"
# case = "yiq_value_table"
# subject = "colorsys.rgb_to_yiq"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_colorsys.py"
# status = "filled"
# ///
"""colorsys.rgb_to_yiq: the canonical CPython RGB<->YIQ value table (black/blue/green/cyan/red/purple/yellow/white/grey) matches in both directions"""
import colorsys

# YIQ matrix constants are rounded to 4 decimals in the oracle table, so the
# tolerance is intentionally looser than the float-noise EPS used elsewhere.
EPS = 1e-4
# (rgb, yiq) — straight from CPython's test_colorsys.test_yiq_values.
table = [
    ((0.0, 0.0, 0.0), (0.0, 0.0, 0.0)),             # black
    ((0.0, 0.0, 1.0), (0.11, -0.3217, 0.3121)),     # blue
    ((0.0, 1.0, 0.0), (0.59, -0.2773, -0.5251)),    # green
    ((0.0, 1.0, 1.0), (0.7, -0.599, -0.213)),       # cyan
    ((1.0, 0.0, 0.0), (0.3, 0.599, 0.213)),         # red
    ((1.0, 0.0, 1.0), (0.41, 0.2773, 0.5251)),      # purple
    ((1.0, 1.0, 0.0), (0.89, 0.3217, -0.3121)),     # yellow
    ((1.0, 1.0, 1.0), (1.0, 0.0, 0.0)),             # white
    ((0.5, 0.5, 0.5), (0.5, 0.0, 0.0)),             # grey
]
for rgb, yiq in table:
    got = colorsys.rgb_to_yiq(*rgb)
    for a, b in zip(got, yiq):
        assert abs(a - b) < EPS, ("rgb->yiq", rgb, got)
    back = colorsys.yiq_to_rgb(*yiq)
    for a, b in zip(back, rgb):
        assert abs(a - b) < EPS, ("yiq->rgb", yiq, back)

print("yiq_value_table OK")
"###);
    assert_output(&out, r###"yiq_value_table OK
"###);
}
