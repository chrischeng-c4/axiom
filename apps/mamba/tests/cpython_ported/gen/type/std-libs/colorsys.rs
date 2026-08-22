use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/colorsys/hls_to_rgb__h_as_float_wrong.py`.
#[test]
fn test_gen_type_std_libs_colorsys_hls_to_rgb__h_as_float_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "colorsys"
# dimension = "type"
# case = "hls_to_rgb__h_as_float_wrong"
# subject = "colorsys.hls_to_rgb(h: float)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/colorsys.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: colorsys.hls_to_rgb(h: float); call it with the wrong type.

typeshed contract: h is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from colorsys import hls_to_rgb
try:
    hls_to_rgb("not_a_float", 0.0, 0.0)  # h: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/colorsys/hsv_to_rgb__h_as_float_wrong.py`.
#[test]
fn test_gen_type_std_libs_colorsys_hsv_to_rgb__h_as_float_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "colorsys"
# dimension = "type"
# case = "hsv_to_rgb__h_as_float_wrong"
# subject = "colorsys.hsv_to_rgb(h: float)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/colorsys.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: colorsys.hsv_to_rgb(h: float); call it with the wrong type.

typeshed contract: h is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from colorsys import hsv_to_rgb
try:
    hsv_to_rgb("not_a_float", 0.0, 0.0)  # h: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/colorsys/rgb_to_hls__r_as_float_wrong.py`.
#[test]
fn test_gen_type_std_libs_colorsys_rgb_to_hls__r_as_float_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "colorsys"
# dimension = "type"
# case = "rgb_to_hls__r_as_float_wrong"
# subject = "colorsys.rgb_to_hls(r: float)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/colorsys.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: colorsys.rgb_to_hls(r: float); call it with the wrong type.

typeshed contract: r is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from colorsys import rgb_to_hls
try:
    rgb_to_hls("not_a_float", 0.0, 0.0)  # r: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/colorsys/rgb_to_hsv__r_as_float_wrong.py`.
#[test]
fn test_gen_type_std_libs_colorsys_rgb_to_hsv__r_as_float_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "colorsys"
# dimension = "type"
# case = "rgb_to_hsv__r_as_float_wrong"
# subject = "colorsys.rgb_to_hsv(r: float)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/colorsys.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: colorsys.rgb_to_hsv(r: float); call it with the wrong type.

typeshed contract: r is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from colorsys import rgb_to_hsv
try:
    rgb_to_hsv("not_a_float", 0.0, 0.0)  # r: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/colorsys/rgb_to_yiq__r_as_float_wrong.py`.
#[test]
fn test_gen_type_std_libs_colorsys_rgb_to_yiq__r_as_float_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "colorsys"
# dimension = "type"
# case = "rgb_to_yiq__r_as_float_wrong"
# subject = "colorsys.rgb_to_yiq(r: float)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/colorsys.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: colorsys.rgb_to_yiq(r: float); call it with the wrong type.

typeshed contract: r is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from colorsys import rgb_to_yiq
try:
    rgb_to_yiq("not_a_float", 0.0, 0.0)  # r: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/colorsys/yiq_to_rgb__y_as_float_wrong.py`.
#[test]
fn test_gen_type_std_libs_colorsys_yiq_to_rgb__y_as_float_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "colorsys"
# dimension = "type"
# case = "yiq_to_rgb__y_as_float_wrong"
# subject = "colorsys.yiq_to_rgb(y: float)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/colorsys.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: colorsys.yiq_to_rgb(y: float); call it with the wrong type.

typeshed contract: y is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from colorsys import yiq_to_rgb
try:
    yiq_to_rgb("not_a_float", 0.0, 0.0)  # y: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
