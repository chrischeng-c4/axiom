use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/_operator/abs__a_as_SupportsAbs_wrong.py`.
#[test]
fn test_gen_type_std_libs__operator_abs__a_as_SupportsAbs_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_operator"
# dimension = "type"
# case = "abs__a_as_SupportsAbs_wrong"
# subject = "_operator.abs(a: SupportsAbs)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_operator.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _operator.abs(a: SupportsAbs); call it with the wrong type.

typeshed contract: a is SupportsAbs. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _operator import abs
try:
    abs(_W())  # a: SupportsAbs <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_operator/call__obj_as_Callable_wrong.py`.
#[test]
fn test_gen_type_std_libs__operator_call__obj_as_Callable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_operator"
# dimension = "type"
# case = "call__obj_as_Callable_wrong"
# subject = "_operator.call(obj: Callable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_operator.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _operator.call(obj: Callable); call it with the wrong type.

typeshed contract: obj is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _operator import call
try:
    call(_W())  # obj: Callable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_operator/concat__a_as_Sequence_wrong.py`.
#[test]
fn test_gen_type_std_libs__operator_concat__a_as_Sequence_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_operator"
# dimension = "type"
# case = "concat__a_as_Sequence_wrong"
# subject = "_operator.concat(a: Sequence)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_operator.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _operator.concat(a: Sequence); call it with the wrong type.

typeshed contract: a is Sequence. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _operator import concat
try:
    concat(_W(), None)  # a: Sequence <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_operator/contains__a_as_Container_wrong.py`.
#[test]
fn test_gen_type_std_libs__operator_contains__a_as_Container_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_operator"
# dimension = "type"
# case = "contains__a_as_Container_wrong"
# subject = "_operator.contains(a: Container)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_operator.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _operator.contains(a: Container); call it with the wrong type.

typeshed contract: a is Container. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _operator import contains
try:
    contains(_W(), None)  # a: Container <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_operator/countOf__a_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs__operator_countOf__a_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_operator"
# dimension = "type"
# case = "countOf__a_as_Iterable_wrong"
# subject = "_operator.countOf(a: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_operator.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _operator.countOf(a: Iterable); call it with the wrong type.

typeshed contract: a is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _operator import countOf
try:
    countOf(_W(), None)  # a: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_operator/delitem__a_as_MutableMapping_wrong.py`.
#[test]
fn test_gen_type_std_libs__operator_delitem__a_as_MutableMapping_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_operator"
# dimension = "type"
# case = "delitem__a_as_MutableMapping_wrong"
# subject = "_operator.delitem(a: MutableMapping)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_operator.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _operator.delitem(a: MutableMapping); call it with the wrong type.

typeshed contract: a is MutableMapping. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _operator import delitem
try:
    delitem(_W(), None)  # a: MutableMapping <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_operator/delitem__a_as_MutableSequence_wrong.py`.
#[test]
fn test_gen_type_std_libs__operator_delitem__a_as_MutableSequence_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_operator"
# dimension = "type"
# case = "delitem__a_as_MutableSequence_wrong"
# subject = "_operator.delitem(a: MutableSequence)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_operator.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _operator.delitem(a: MutableSequence); call it with the wrong type.

typeshed contract: a is MutableSequence. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _operator import delitem
try:
    delitem(_W(), 0)  # a: MutableSequence <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_operator/ge__a_as__SupportsComparison_wrong.py`.
#[test]
fn test_gen_type_std_libs__operator_ge__a_as__SupportsComparison_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_operator"
# dimension = "type"
# case = "ge__a_as__SupportsComparison_wrong"
# subject = "_operator.ge(a: _SupportsComparison)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_operator.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _operator.ge(a: _SupportsComparison); call it with the wrong type.

typeshed contract: a is _SupportsComparison. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _operator import ge
try:
    ge(_W(), None)  # a: _SupportsComparison <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_operator/getitem__a_as_Sequence_wrong.py`.
#[test]
fn test_gen_type_std_libs__operator_getitem__a_as_Sequence_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_operator"
# dimension = "type"
# case = "getitem__a_as_Sequence_wrong"
# subject = "_operator.getitem(a: Sequence)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_operator.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _operator.getitem(a: Sequence); call it with the wrong type.

typeshed contract: a is Sequence. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _operator import getitem
try:
    getitem(_W(), None)  # a: Sequence <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_operator/getitem__a_as_SupportsGetItem_wrong.py`.
#[test]
fn test_gen_type_std_libs__operator_getitem__a_as_SupportsGetItem_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_operator"
# dimension = "type"
# case = "getitem__a_as_SupportsGetItem_wrong"
# subject = "_operator.getitem(a: SupportsGetItem)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_operator.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _operator.getitem(a: SupportsGetItem); call it with the wrong type.

typeshed contract: a is SupportsGetItem. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _operator import getitem
try:
    getitem(_W(), None)  # a: SupportsGetItem <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_operator/gt__a_as__SupportsComparison_wrong.py`.
#[test]
fn test_gen_type_std_libs__operator_gt__a_as__SupportsComparison_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_operator"
# dimension = "type"
# case = "gt__a_as__SupportsComparison_wrong"
# subject = "_operator.gt(a: _SupportsComparison)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_operator.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _operator.gt(a: _SupportsComparison); call it with the wrong type.

typeshed contract: a is _SupportsComparison. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _operator import gt
try:
    gt(_W(), None)  # a: _SupportsComparison <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_operator/indexOf__a_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs__operator_indexOf__a_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_operator"
# dimension = "type"
# case = "indexOf__a_as_Iterable_wrong"
# subject = "_operator.indexOf(a: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_operator.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _operator.indexOf(a: Iterable); call it with the wrong type.

typeshed contract: a is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _operator import indexOf
try:
    indexOf(_W(), None)  # a: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_operator/index__a_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs__operator_index__a_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_operator"
# dimension = "type"
# case = "index__a_as_SupportsIndex_wrong"
# subject = "_operator.index(a: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_operator.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _operator.index(a: SupportsIndex); call it with the wrong type.

typeshed contract: a is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _operator import index
try:
    index(_W())  # a: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_operator/inv__a_as__SupportsInversion_wrong.py`.
#[test]
fn test_gen_type_std_libs__operator_inv__a_as__SupportsInversion_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_operator"
# dimension = "type"
# case = "inv__a_as__SupportsInversion_wrong"
# subject = "_operator.inv(a: _SupportsInversion)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_operator.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _operator.inv(a: _SupportsInversion); call it with the wrong type.

typeshed contract: a is _SupportsInversion. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _operator import inv
try:
    inv(_W())  # a: _SupportsInversion <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_operator/invert__a_as__SupportsInversion_wrong.py`.
#[test]
fn test_gen_type_std_libs__operator_invert__a_as__SupportsInversion_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_operator"
# dimension = "type"
# case = "invert__a_as__SupportsInversion_wrong"
# subject = "_operator.invert(a: _SupportsInversion)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_operator.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _operator.invert(a: _SupportsInversion); call it with the wrong type.

typeshed contract: a is _SupportsInversion. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _operator import invert
try:
    invert(_W())  # a: _SupportsInversion <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_operator/is_not_none__a_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs__operator_is_not_none__a_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_operator"
# dimension = "type"
# case = "is_not_none__a_as_typed_wrong"
# subject = "_operator.is_not_none(a: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_operator.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _operator.is_not_none(a: typed); call it with the wrong type.

typeshed contract: a is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _operator import is_not_none
try:
    is_not_none(_W())  # a: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_operator/le__a_as__SupportsComparison_wrong.py`.
#[test]
fn test_gen_type_std_libs__operator_le__a_as__SupportsComparison_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_operator"
# dimension = "type"
# case = "le__a_as__SupportsComparison_wrong"
# subject = "_operator.le(a: _SupportsComparison)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_operator.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _operator.le(a: _SupportsComparison); call it with the wrong type.

typeshed contract: a is _SupportsComparison. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _operator import le
try:
    le(_W(), None)  # a: _SupportsComparison <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_operator/length_hint__default_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__operator_length_hint__default_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_operator"
# dimension = "type"
# case = "length_hint__default_as_int_wrong"
# subject = "_operator.length_hint(default: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_operator.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _operator.length_hint(default: int); call it with the wrong type.

typeshed contract: default is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _operator import length_hint
try:
    length_hint(None, "not_an_int")  # default: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_operator/lt__a_as__SupportsComparison_wrong.py`.
#[test]
fn test_gen_type_std_libs__operator_lt__a_as__SupportsComparison_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_operator"
# dimension = "type"
# case = "lt__a_as__SupportsComparison_wrong"
# subject = "_operator.lt(a: _SupportsComparison)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_operator.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _operator.lt(a: _SupportsComparison); call it with the wrong type.

typeshed contract: a is _SupportsComparison. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _operator import lt
try:
    lt(_W(), None)  # a: _SupportsComparison <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_operator/neg__a_as__SupportsNeg_wrong.py`.
#[test]
fn test_gen_type_std_libs__operator_neg__a_as__SupportsNeg_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_operator"
# dimension = "type"
# case = "neg__a_as__SupportsNeg_wrong"
# subject = "_operator.neg(a: _SupportsNeg)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_operator.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _operator.neg(a: _SupportsNeg); call it with the wrong type.

typeshed contract: a is _SupportsNeg. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _operator import neg
try:
    neg(_W())  # a: _SupportsNeg <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_operator/pos__a_as__SupportsPos_wrong.py`.
#[test]
fn test_gen_type_std_libs__operator_pos__a_as__SupportsPos_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_operator"
# dimension = "type"
# case = "pos__a_as__SupportsPos_wrong"
# subject = "_operator.pos(a: _SupportsPos)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_operator.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _operator.pos(a: _SupportsPos); call it with the wrong type.

typeshed contract: a is _SupportsPos. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _operator import pos
try:
    pos(_W())  # a: _SupportsPos <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_operator/setitem__a_as_MutableMapping_wrong.py`.
#[test]
fn test_gen_type_std_libs__operator_setitem__a_as_MutableMapping_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_operator"
# dimension = "type"
# case = "setitem__a_as_MutableMapping_wrong"
# subject = "_operator.setitem(a: MutableMapping)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_operator.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _operator.setitem(a: MutableMapping); call it with the wrong type.

typeshed contract: a is MutableMapping. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _operator import setitem
try:
    setitem(_W(), None, None)  # a: MutableMapping <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_operator/setitem__a_as_MutableSequence_wrong.py`.
#[test]
fn test_gen_type_std_libs__operator_setitem__a_as_MutableSequence_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_operator"
# dimension = "type"
# case = "setitem__a_as_MutableSequence_wrong"
# subject = "_operator.setitem(a: MutableSequence)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_operator.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _operator.setitem(a: MutableSequence); call it with the wrong type.

typeshed contract: a is MutableSequence. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _operator import setitem
try:
    setitem(_W(), 0, None)  # a: MutableSequence <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/operator/attrgetter____new____attr_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_operator_attrgetter____new____attr_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "type"
# case = "attrgetter____new____attr_as_str_wrong"
# subject = "operator.attrgetter.__new__(attr: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/operator.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: operator.attrgetter.__new__(attr: str); call it with the wrong type.

typeshed contract: attr is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from operator import attrgetter
obj = object.__new__(attrgetter)
try:
    obj.__new__(12345)  # attr: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/operator/itemgetter____call____obj_as_SupportsGetItem_wrong.py`.
#[test]
fn test_gen_type_std_libs_operator_itemgetter____call____obj_as_SupportsGetItem_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "type"
# case = "itemgetter____call____obj_as_SupportsGetItem_wrong"
# subject = "operator.itemgetter.__call__(obj: SupportsGetItem)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/operator.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: operator.itemgetter.__call__(obj: SupportsGetItem); call it with the wrong type.

typeshed contract: obj is SupportsGetItem. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from operator import itemgetter
obj = object.__new__(itemgetter)
try:
    obj.__call__(_W())  # obj: SupportsGetItem <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/operator/itemgetter____new____item1_as__T1_wrong.py`.
#[test]
fn test_gen_type_std_libs_operator_itemgetter____new____item1_as__T1_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "type"
# case = "itemgetter____new____item1_as__T1_wrong"
# subject = "operator.itemgetter.__new__(item1: _T1)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/operator.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: operator.itemgetter.__new__(item1: _T1); call it with the wrong type.

typeshed contract: item1 is _T1. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from operator import itemgetter
obj = object.__new__(itemgetter)
try:
    obj.__new__(_W(), None)  # item1: _T1 <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/operator/itemgetter____new____item_as__T_wrong.py`.
#[test]
fn test_gen_type_std_libs_operator_itemgetter____new____item_as__T_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "type"
# case = "itemgetter____new____item_as__T_wrong"
# subject = "operator.itemgetter.__new__(item: _T)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/operator.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: operator.itemgetter.__new__(item: _T); call it with the wrong type.

typeshed contract: item is _T. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from operator import itemgetter
obj = object.__new__(itemgetter)
try:
    obj.__new__(_W())  # item: _T <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/operator/methodcaller____new____name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_operator_methodcaller____new____name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "type"
# case = "methodcaller____new____name_as_str_wrong"
# subject = "operator.methodcaller.__new__(name: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/operator.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: operator.methodcaller.__new__(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from operator import methodcaller
obj = object.__new__(methodcaller)
try:
    obj.__new__(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
