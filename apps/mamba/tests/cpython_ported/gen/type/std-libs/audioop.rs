use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/audioop/add__fragment1_as_Buffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_audioop_add__fragment1_as_Buffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "audioop"
# dimension = "type"
# case = "add__fragment1_as_Buffer_wrong"
# subject = "audioop.add(fragment1: Buffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/audioop.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: audioop.add(fragment1: Buffer); call it with the wrong type.

typeshed contract: fragment1 is Buffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from audioop import add
try:
    add(_W(), None, 0)  # fragment1: Buffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/audioop/adpcm2lin__fragment_as_Buffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_audioop_adpcm2lin__fragment_as_Buffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "audioop"
# dimension = "type"
# case = "adpcm2lin__fragment_as_Buffer_wrong"
# subject = "audioop.adpcm2lin(fragment: Buffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/audioop.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: audioop.adpcm2lin(fragment: Buffer); call it with the wrong type.

typeshed contract: fragment is Buffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from audioop import adpcm2lin
try:
    adpcm2lin(_W(), 0, None)  # fragment: Buffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/audioop/alaw2lin__fragment_as_Buffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_audioop_alaw2lin__fragment_as_Buffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "audioop"
# dimension = "type"
# case = "alaw2lin__fragment_as_Buffer_wrong"
# subject = "audioop.alaw2lin(fragment: Buffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/audioop.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: audioop.alaw2lin(fragment: Buffer); call it with the wrong type.

typeshed contract: fragment is Buffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from audioop import alaw2lin
try:
    alaw2lin(_W(), 0)  # fragment: Buffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/audioop/avg__fragment_as_Buffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_audioop_avg__fragment_as_Buffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "audioop"
# dimension = "type"
# case = "avg__fragment_as_Buffer_wrong"
# subject = "audioop.avg(fragment: Buffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/audioop.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: audioop.avg(fragment: Buffer); call it with the wrong type.

typeshed contract: fragment is Buffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from audioop import avg
try:
    avg(_W(), 0)  # fragment: Buffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/audioop/avgpp__fragment_as_Buffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_audioop_avgpp__fragment_as_Buffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "audioop"
# dimension = "type"
# case = "avgpp__fragment_as_Buffer_wrong"
# subject = "audioop.avgpp(fragment: Buffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/audioop.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: audioop.avgpp(fragment: Buffer); call it with the wrong type.

typeshed contract: fragment is Buffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from audioop import avgpp
try:
    avgpp(_W(), 0)  # fragment: Buffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/audioop/bias__fragment_as_Buffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_audioop_bias__fragment_as_Buffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "audioop"
# dimension = "type"
# case = "bias__fragment_as_Buffer_wrong"
# subject = "audioop.bias(fragment: Buffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/audioop.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: audioop.bias(fragment: Buffer); call it with the wrong type.

typeshed contract: fragment is Buffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from audioop import bias
try:
    bias(_W(), 0, 0)  # fragment: Buffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/audioop/byteswap__fragment_as_Buffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_audioop_byteswap__fragment_as_Buffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "audioop"
# dimension = "type"
# case = "byteswap__fragment_as_Buffer_wrong"
# subject = "audioop.byteswap(fragment: Buffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/audioop.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: audioop.byteswap(fragment: Buffer); call it with the wrong type.

typeshed contract: fragment is Buffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from audioop import byteswap
try:
    byteswap(_W(), 0)  # fragment: Buffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/audioop/cross__fragment_as_Buffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_audioop_cross__fragment_as_Buffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "audioop"
# dimension = "type"
# case = "cross__fragment_as_Buffer_wrong"
# subject = "audioop.cross(fragment: Buffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/audioop.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: audioop.cross(fragment: Buffer); call it with the wrong type.

typeshed contract: fragment is Buffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from audioop import cross
try:
    cross(_W(), 0)  # fragment: Buffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/audioop/findfactor__fragment_as_Buffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_audioop_findfactor__fragment_as_Buffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "audioop"
# dimension = "type"
# case = "findfactor__fragment_as_Buffer_wrong"
# subject = "audioop.findfactor(fragment: Buffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/audioop.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: audioop.findfactor(fragment: Buffer); call it with the wrong type.

typeshed contract: fragment is Buffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from audioop import findfactor
try:
    findfactor(_W(), None)  # fragment: Buffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/audioop/findfit__fragment_as_Buffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_audioop_findfit__fragment_as_Buffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "audioop"
# dimension = "type"
# case = "findfit__fragment_as_Buffer_wrong"
# subject = "audioop.findfit(fragment: Buffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/audioop.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: audioop.findfit(fragment: Buffer); call it with the wrong type.

typeshed contract: fragment is Buffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from audioop import findfit
try:
    findfit(_W(), None)  # fragment: Buffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/audioop/findmax__fragment_as_Buffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_audioop_findmax__fragment_as_Buffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "audioop"
# dimension = "type"
# case = "findmax__fragment_as_Buffer_wrong"
# subject = "audioop.findmax(fragment: Buffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/audioop.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: audioop.findmax(fragment: Buffer); call it with the wrong type.

typeshed contract: fragment is Buffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from audioop import findmax
try:
    findmax(_W(), 0)  # fragment: Buffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/audioop/getsample__fragment_as_Buffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_audioop_getsample__fragment_as_Buffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "audioop"
# dimension = "type"
# case = "getsample__fragment_as_Buffer_wrong"
# subject = "audioop.getsample(fragment: Buffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/audioop.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: audioop.getsample(fragment: Buffer); call it with the wrong type.

typeshed contract: fragment is Buffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from audioop import getsample
try:
    getsample(_W(), 0, 0)  # fragment: Buffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/audioop/lin2adpcm__fragment_as_Buffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_audioop_lin2adpcm__fragment_as_Buffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "audioop"
# dimension = "type"
# case = "lin2adpcm__fragment_as_Buffer_wrong"
# subject = "audioop.lin2adpcm(fragment: Buffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/audioop.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: audioop.lin2adpcm(fragment: Buffer); call it with the wrong type.

typeshed contract: fragment is Buffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from audioop import lin2adpcm
try:
    lin2adpcm(_W(), 0, None)  # fragment: Buffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/audioop/lin2alaw__fragment_as_Buffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_audioop_lin2alaw__fragment_as_Buffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "audioop"
# dimension = "type"
# case = "lin2alaw__fragment_as_Buffer_wrong"
# subject = "audioop.lin2alaw(fragment: Buffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/audioop.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: audioop.lin2alaw(fragment: Buffer); call it with the wrong type.

typeshed contract: fragment is Buffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from audioop import lin2alaw
try:
    lin2alaw(_W(), 0)  # fragment: Buffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/audioop/lin2lin__fragment_as_Buffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_audioop_lin2lin__fragment_as_Buffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "audioop"
# dimension = "type"
# case = "lin2lin__fragment_as_Buffer_wrong"
# subject = "audioop.lin2lin(fragment: Buffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/audioop.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: audioop.lin2lin(fragment: Buffer); call it with the wrong type.

typeshed contract: fragment is Buffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from audioop import lin2lin
try:
    lin2lin(_W(), 0, 0)  # fragment: Buffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/audioop/lin2ulaw__fragment_as_Buffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_audioop_lin2ulaw__fragment_as_Buffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "audioop"
# dimension = "type"
# case = "lin2ulaw__fragment_as_Buffer_wrong"
# subject = "audioop.lin2ulaw(fragment: Buffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/audioop.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: audioop.lin2ulaw(fragment: Buffer); call it with the wrong type.

typeshed contract: fragment is Buffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from audioop import lin2ulaw
try:
    lin2ulaw(_W(), 0)  # fragment: Buffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/audioop/max__fragment_as_Buffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_audioop_max__fragment_as_Buffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "audioop"
# dimension = "type"
# case = "max__fragment_as_Buffer_wrong"
# subject = "audioop.max(fragment: Buffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/audioop.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: audioop.max(fragment: Buffer); call it with the wrong type.

typeshed contract: fragment is Buffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from audioop import max
try:
    max(_W(), 0)  # fragment: Buffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/audioop/maxpp__fragment_as_Buffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_audioop_maxpp__fragment_as_Buffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "audioop"
# dimension = "type"
# case = "maxpp__fragment_as_Buffer_wrong"
# subject = "audioop.maxpp(fragment: Buffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/audioop.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: audioop.maxpp(fragment: Buffer); call it with the wrong type.

typeshed contract: fragment is Buffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from audioop import maxpp
try:
    maxpp(_W(), 0)  # fragment: Buffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/audioop/minmax__fragment_as_Buffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_audioop_minmax__fragment_as_Buffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "audioop"
# dimension = "type"
# case = "minmax__fragment_as_Buffer_wrong"
# subject = "audioop.minmax(fragment: Buffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/audioop.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: audioop.minmax(fragment: Buffer); call it with the wrong type.

typeshed contract: fragment is Buffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from audioop import minmax
try:
    minmax(_W(), 0)  # fragment: Buffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/audioop/mul__fragment_as_Buffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_audioop_mul__fragment_as_Buffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "audioop"
# dimension = "type"
# case = "mul__fragment_as_Buffer_wrong"
# subject = "audioop.mul(fragment: Buffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/audioop.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: audioop.mul(fragment: Buffer); call it with the wrong type.

typeshed contract: fragment is Buffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from audioop import mul
try:
    mul(_W(), 0, 0.0)  # fragment: Buffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/audioop/ratecv__fragment_as_Buffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_audioop_ratecv__fragment_as_Buffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "audioop"
# dimension = "type"
# case = "ratecv__fragment_as_Buffer_wrong"
# subject = "audioop.ratecv(fragment: Buffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/audioop.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: audioop.ratecv(fragment: Buffer); call it with the wrong type.

typeshed contract: fragment is Buffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from audioop import ratecv
try:
    ratecv(_W(), 0, 0, 0, 0, None)  # fragment: Buffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/audioop/reverse__fragment_as_Buffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_audioop_reverse__fragment_as_Buffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "audioop"
# dimension = "type"
# case = "reverse__fragment_as_Buffer_wrong"
# subject = "audioop.reverse(fragment: Buffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/audioop.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: audioop.reverse(fragment: Buffer); call it with the wrong type.

typeshed contract: fragment is Buffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from audioop import reverse
try:
    reverse(_W(), 0)  # fragment: Buffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/audioop/rms__fragment_as_Buffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_audioop_rms__fragment_as_Buffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "audioop"
# dimension = "type"
# case = "rms__fragment_as_Buffer_wrong"
# subject = "audioop.rms(fragment: Buffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/audioop.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: audioop.rms(fragment: Buffer); call it with the wrong type.

typeshed contract: fragment is Buffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from audioop import rms
try:
    rms(_W(), 0)  # fragment: Buffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/audioop/tomono__fragment_as_Buffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_audioop_tomono__fragment_as_Buffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "audioop"
# dimension = "type"
# case = "tomono__fragment_as_Buffer_wrong"
# subject = "audioop.tomono(fragment: Buffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/audioop.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: audioop.tomono(fragment: Buffer); call it with the wrong type.

typeshed contract: fragment is Buffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from audioop import tomono
try:
    tomono(_W(), 0, 0.0, 0.0)  # fragment: Buffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/audioop/tostereo__fragment_as_Buffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_audioop_tostereo__fragment_as_Buffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "audioop"
# dimension = "type"
# case = "tostereo__fragment_as_Buffer_wrong"
# subject = "audioop.tostereo(fragment: Buffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/audioop.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: audioop.tostereo(fragment: Buffer); call it with the wrong type.

typeshed contract: fragment is Buffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from audioop import tostereo
try:
    tostereo(_W(), 0, 0.0, 0.0)  # fragment: Buffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/audioop/ulaw2lin__fragment_as_Buffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_audioop_ulaw2lin__fragment_as_Buffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "audioop"
# dimension = "type"
# case = "ulaw2lin__fragment_as_Buffer_wrong"
# subject = "audioop.ulaw2lin(fragment: Buffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/audioop.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: audioop.ulaw2lin(fragment: Buffer); call it with the wrong type.

typeshed contract: fragment is Buffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from audioop import ulaw2lin
try:
    ulaw2lin(_W(), 0)  # fragment: Buffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
