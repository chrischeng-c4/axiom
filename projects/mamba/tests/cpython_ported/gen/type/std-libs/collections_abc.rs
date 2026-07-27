use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/_collections_abc/Buffer____buffer____flags_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__collections_abc_Buffer____buffer____flags_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_collections_abc"
# dimension = "type"
# case = "Buffer____buffer____flags_as_int_wrong"
# subject = "_collections_abc.Buffer.__buffer__(flags: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_collections_abc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _collections_abc.Buffer.__buffer__(flags: int); call it with the wrong type.

typeshed contract: flags is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _collections_abc import Buffer
obj = object.__new__(Buffer)
try:
    obj.__buffer__("not_an_int")  # flags: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_collections_abc/dict_items__isdisjoint__other_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs__collections_abc_dict_items__isdisjoint__other_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_collections_abc"
# dimension = "type"
# case = "dict_items__isdisjoint__other_as_Iterable_wrong"
# subject = "_collections_abc.dict_items.isdisjoint(other: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_collections_abc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _collections_abc.dict_items.isdisjoint(other: Iterable); call it with the wrong type.

typeshed contract: other is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _collections_abc import dict_items
obj = object.__new__(dict_items)
try:
    obj.isdisjoint(_W())  # other: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_collections_abc/dict_keys__isdisjoint__other_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs__collections_abc_dict_keys__isdisjoint__other_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_collections_abc"
# dimension = "type"
# case = "dict_keys__isdisjoint__other_as_Iterable_wrong"
# subject = "_collections_abc.dict_keys.isdisjoint(other: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_collections_abc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _collections_abc.dict_keys.isdisjoint(other: Iterable); call it with the wrong type.

typeshed contract: other is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _collections_abc import dict_keys
obj = object.__new__(dict_keys)
try:
    obj.isdisjoint(_W())  # other: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
