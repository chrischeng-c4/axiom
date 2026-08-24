use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/distutils_dep_util/newer__source_as_StrOrBytesPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_dep_util_newer__source_as_StrOrBytesPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_dep_util"
# dimension = "type"
# case = "newer__source_as_StrOrBytesPath_wrong"
# subject = "distutils.dep_util.newer(source: StrOrBytesPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/dep_util.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.dep_util.newer(source: StrOrBytesPath); call it with the wrong type.

typeshed contract: source is StrOrBytesPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils.dep_util import newer
try:
    newer(_W(), None)  # source: StrOrBytesPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/distutils_dep_util/newer_group__sources_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_dep_util_newer_group__sources_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_dep_util"
# dimension = "type"
# case = "newer_group__sources_as_Iterable_wrong"
# subject = "distutils.dep_util.newer_group(sources: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/dep_util.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.dep_util.newer_group(sources: Iterable); call it with the wrong type.

typeshed contract: sources is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils.dep_util import newer_group
try:
    newer_group(_W(), None)  # sources: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/distutils_dep_util/newer_pairwise__sources_as_SupportsLenAndGetItem_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_dep_util_newer_pairwise__sources_as_SupportsLenAndGetItem_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_dep_util"
# dimension = "type"
# case = "newer_pairwise__sources_as_SupportsLenAndGetItem_wrong"
# subject = "distutils.dep_util.newer_pairwise(sources: SupportsLenAndGetItem)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/dep_util.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.dep_util.newer_pairwise(sources: SupportsLenAndGetItem); call it with the wrong type.

typeshed contract: sources is SupportsLenAndGetItem. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils.dep_util import newer_pairwise
try:
    newer_pairwise(_W(), None)  # sources: SupportsLenAndGetItem <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
