use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/_bisect/bisect_left__a_as_SupportsGetItem_wrong.py`.
#[test]
fn test_gen_type_std_libs__bisect_bisect_left__a_as_SupportsGetItem_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_bisect"
# dimension = "type"
# case = "bisect_left__a_as_SupportsGetItem_wrong"
# subject = "_bisect.bisect_left(a: SupportsGetItem)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_bisect.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _bisect.bisect_left(a: SupportsGetItem); call it with the wrong type.

typeshed contract: a is SupportsGetItem. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _bisect import bisect_left
try:
    bisect_left(_W(), None, 0, 0)  # a: SupportsGetItem <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_bisect/bisect_left__a_as_SupportsLenAndGetItem_wrong.py`.
#[test]
fn test_gen_type_std_libs__bisect_bisect_left__a_as_SupportsLenAndGetItem_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_bisect"
# dimension = "type"
# case = "bisect_left__a_as_SupportsLenAndGetItem_wrong"
# subject = "_bisect.bisect_left(a: SupportsLenAndGetItem)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_bisect.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _bisect.bisect_left(a: SupportsLenAndGetItem); call it with the wrong type.

typeshed contract: a is SupportsLenAndGetItem. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _bisect import bisect_left
try:
    bisect_left(_W(), None)  # a: SupportsLenAndGetItem <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_bisect/bisect_right__a_as_SupportsGetItem_wrong.py`.
#[test]
fn test_gen_type_std_libs__bisect_bisect_right__a_as_SupportsGetItem_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_bisect"
# dimension = "type"
# case = "bisect_right__a_as_SupportsGetItem_wrong"
# subject = "_bisect.bisect_right(a: SupportsGetItem)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_bisect.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _bisect.bisect_right(a: SupportsGetItem); call it with the wrong type.

typeshed contract: a is SupportsGetItem. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _bisect import bisect_right
try:
    bisect_right(_W(), None, 0, 0)  # a: SupportsGetItem <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_bisect/bisect_right__a_as_SupportsLenAndGetItem_wrong.py`.
#[test]
fn test_gen_type_std_libs__bisect_bisect_right__a_as_SupportsLenAndGetItem_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_bisect"
# dimension = "type"
# case = "bisect_right__a_as_SupportsLenAndGetItem_wrong"
# subject = "_bisect.bisect_right(a: SupportsLenAndGetItem)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_bisect.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _bisect.bisect_right(a: SupportsLenAndGetItem); call it with the wrong type.

typeshed contract: a is SupportsLenAndGetItem. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _bisect import bisect_right
try:
    bisect_right(_W(), None)  # a: SupportsLenAndGetItem <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_bisect/insort_left__a_as_MutableSequence_wrong.py`.
#[test]
fn test_gen_type_std_libs__bisect_insort_left__a_as_MutableSequence_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_bisect"
# dimension = "type"
# case = "insort_left__a_as_MutableSequence_wrong"
# subject = "_bisect.insort_left(a: MutableSequence)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_bisect.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _bisect.insort_left(a: MutableSequence); call it with the wrong type.

typeshed contract: a is MutableSequence. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _bisect import insort_left
try:
    insort_left(_W(), None)  # a: MutableSequence <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_bisect/insort_right__a_as_MutableSequence_wrong.py`.
#[test]
fn test_gen_type_std_libs__bisect_insort_right__a_as_MutableSequence_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_bisect"
# dimension = "type"
# case = "insort_right__a_as_MutableSequence_wrong"
# subject = "_bisect.insort_right(a: MutableSequence)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_bisect.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _bisect.insort_right(a: MutableSequence); call it with the wrong type.

typeshed contract: a is MutableSequence. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _bisect import insort_right
try:
    insort_right(_W(), None)  # a: MutableSequence <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
