use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/lib2to3_btm_matcher/BottomMatcher__add__pattern_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_lib2to3_btm_matcher_BottomMatcher__add__pattern_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_btm_matcher"
# dimension = "type"
# case = "BottomMatcher__add__pattern_as_typed_wrong"
# subject = "lib2to3.btm_matcher.BottomMatcher.add(pattern: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/btm_matcher.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.btm_matcher.BottomMatcher.add(pattern: typed); call it with the wrong type.

typeshed contract: pattern is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lib2to3.btm_matcher import BottomMatcher
obj = object.__new__(BottomMatcher)
try:
    obj.add(_W(), None)  # pattern: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lib2to3_btm_matcher/BottomMatcher__add_fixer__fixer_as_BaseFix_wrong.py`.
#[test]
fn test_gen_type_std_libs_lib2to3_btm_matcher_BottomMatcher__add_fixer__fixer_as_BaseFix_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_btm_matcher"
# dimension = "type"
# case = "BottomMatcher__add_fixer__fixer_as_BaseFix_wrong"
# subject = "lib2to3.btm_matcher.BottomMatcher.add_fixer(fixer: BaseFix)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/btm_matcher.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.btm_matcher.BottomMatcher.add_fixer(fixer: BaseFix); call it with the wrong type.

typeshed contract: fixer is BaseFix. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lib2to3.btm_matcher import BottomMatcher
obj = object.__new__(BottomMatcher)
try:
    obj.add_fixer(_W())  # fixer: BaseFix <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lib2to3_btm_matcher/BottomMatcher__run__leaves_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_lib2to3_btm_matcher_BottomMatcher__run__leaves_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_btm_matcher"
# dimension = "type"
# case = "BottomMatcher__run__leaves_as_Iterable_wrong"
# subject = "lib2to3.btm_matcher.BottomMatcher.run(leaves: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/btm_matcher.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.btm_matcher.BottomMatcher.run(leaves: Iterable); call it with the wrong type.

typeshed contract: leaves is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lib2to3.btm_matcher import BottomMatcher
obj = object.__new__(BottomMatcher)
try:
    obj.run(_W())  # leaves: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lib2to3_btm_matcher/type_repr__type_num_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_lib2to3_btm_matcher_type_repr__type_num_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_btm_matcher"
# dimension = "type"
# case = "type_repr__type_num_as_int_wrong"
# subject = "lib2to3.btm_matcher.type_repr(type_num: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/btm_matcher.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.btm_matcher.type_repr(type_num: int); call it with the wrong type.

typeshed contract: type_num is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from lib2to3.btm_matcher import type_repr
try:
    type_repr("not_an_int")  # type_num: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
