use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/_weakrefset/WeakSet____and____other_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs__weakrefset_WeakSet____and____other_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_weakrefset"
# dimension = "type"
# case = "WeakSet____and____other_as_Iterable_wrong"
# subject = "_weakrefset.WeakSet.__and__(other: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_weakrefset.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _weakrefset.WeakSet.__and__(other: Iterable); call it with the wrong type.

typeshed contract: other is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _weakrefset import WeakSet
obj = object.__new__(WeakSet)
try:
    obj.__and__(_W())  # other: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_weakrefset/WeakSet____ge____other_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs__weakrefset_WeakSet____ge____other_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_weakrefset"
# dimension = "type"
# case = "WeakSet____ge____other_as_Iterable_wrong"
# subject = "_weakrefset.WeakSet.__ge__(other: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_weakrefset.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _weakrefset.WeakSet.__ge__(other: Iterable); call it with the wrong type.

typeshed contract: other is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _weakrefset import WeakSet
obj = object.__new__(WeakSet)
try:
    obj.__ge__(_W())  # other: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_weakrefset/WeakSet____gt____other_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs__weakrefset_WeakSet____gt____other_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_weakrefset"
# dimension = "type"
# case = "WeakSet____gt____other_as_Iterable_wrong"
# subject = "_weakrefset.WeakSet.__gt__(other: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_weakrefset.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _weakrefset.WeakSet.__gt__(other: Iterable); call it with the wrong type.

typeshed contract: other is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _weakrefset import WeakSet
obj = object.__new__(WeakSet)
try:
    obj.__gt__(_W())  # other: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_weakrefset/WeakSet____iand____other_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs__weakrefset_WeakSet____iand____other_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_weakrefset"
# dimension = "type"
# case = "WeakSet____iand____other_as_Iterable_wrong"
# subject = "_weakrefset.WeakSet.__iand__(other: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_weakrefset.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _weakrefset.WeakSet.__iand__(other: Iterable); call it with the wrong type.

typeshed contract: other is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _weakrefset import WeakSet
obj = object.__new__(WeakSet)
try:
    obj.__iand__(_W())  # other: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_weakrefset/WeakSet____ior____other_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs__weakrefset_WeakSet____ior____other_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_weakrefset"
# dimension = "type"
# case = "WeakSet____ior____other_as_Iterable_wrong"
# subject = "_weakrefset.WeakSet.__ior__(other: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_weakrefset.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _weakrefset.WeakSet.__ior__(other: Iterable); call it with the wrong type.

typeshed contract: other is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _weakrefset import WeakSet
obj = object.__new__(WeakSet)
try:
    obj.__ior__(_W())  # other: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_weakrefset/WeakSet____isub____other_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs__weakrefset_WeakSet____isub____other_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_weakrefset"
# dimension = "type"
# case = "WeakSet____isub____other_as_Iterable_wrong"
# subject = "_weakrefset.WeakSet.__isub__(other: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_weakrefset.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _weakrefset.WeakSet.__isub__(other: Iterable); call it with the wrong type.

typeshed contract: other is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _weakrefset import WeakSet
obj = object.__new__(WeakSet)
try:
    obj.__isub__(_W())  # other: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_weakrefset/WeakSet____ixor____other_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs__weakrefset_WeakSet____ixor____other_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_weakrefset"
# dimension = "type"
# case = "WeakSet____ixor____other_as_Iterable_wrong"
# subject = "_weakrefset.WeakSet.__ixor__(other: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_weakrefset.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _weakrefset.WeakSet.__ixor__(other: Iterable); call it with the wrong type.

typeshed contract: other is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _weakrefset import WeakSet
obj = object.__new__(WeakSet)
try:
    obj.__ixor__(_W())  # other: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_weakrefset/WeakSet____le____other_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs__weakrefset_WeakSet____le____other_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_weakrefset"
# dimension = "type"
# case = "WeakSet____le____other_as_Iterable_wrong"
# subject = "_weakrefset.WeakSet.__le__(other: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_weakrefset.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _weakrefset.WeakSet.__le__(other: Iterable); call it with the wrong type.

typeshed contract: other is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _weakrefset import WeakSet
obj = object.__new__(WeakSet)
try:
    obj.__le__(_W())  # other: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_weakrefset/WeakSet____lt____other_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs__weakrefset_WeakSet____lt____other_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_weakrefset"
# dimension = "type"
# case = "WeakSet____lt____other_as_Iterable_wrong"
# subject = "_weakrefset.WeakSet.__lt__(other: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_weakrefset.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _weakrefset.WeakSet.__lt__(other: Iterable); call it with the wrong type.

typeshed contract: other is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _weakrefset import WeakSet
obj = object.__new__(WeakSet)
try:
    obj.__lt__(_W())  # other: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_weakrefset/WeakSet____or____other_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs__weakrefset_WeakSet____or____other_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_weakrefset"
# dimension = "type"
# case = "WeakSet____or____other_as_Iterable_wrong"
# subject = "_weakrefset.WeakSet.__or__(other: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_weakrefset.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _weakrefset.WeakSet.__or__(other: Iterable); call it with the wrong type.

typeshed contract: other is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _weakrefset import WeakSet
obj = object.__new__(WeakSet)
try:
    obj.__or__(_W())  # other: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_weakrefset/WeakSet____sub____other_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs__weakrefset_WeakSet____sub____other_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_weakrefset"
# dimension = "type"
# case = "WeakSet____sub____other_as_Iterable_wrong"
# subject = "_weakrefset.WeakSet.__sub__(other: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_weakrefset.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _weakrefset.WeakSet.__sub__(other: Iterable); call it with the wrong type.

typeshed contract: other is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _weakrefset import WeakSet
obj = object.__new__(WeakSet)
try:
    obj.__sub__(_W())  # other: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_weakrefset/WeakSet____xor____other_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs__weakrefset_WeakSet____xor____other_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_weakrefset"
# dimension = "type"
# case = "WeakSet____xor____other_as_Iterable_wrong"
# subject = "_weakrefset.WeakSet.__xor__(other: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_weakrefset.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _weakrefset.WeakSet.__xor__(other: Iterable); call it with the wrong type.

typeshed contract: other is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _weakrefset import WeakSet
obj = object.__new__(WeakSet)
try:
    obj.__xor__(_W())  # other: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_weakrefset/WeakSet__difference__other_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs__weakrefset_WeakSet__difference__other_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_weakrefset"
# dimension = "type"
# case = "WeakSet__difference__other_as_Iterable_wrong"
# subject = "_weakrefset.WeakSet.difference(other: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_weakrefset.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _weakrefset.WeakSet.difference(other: Iterable); call it with the wrong type.

typeshed contract: other is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _weakrefset import WeakSet
obj = object.__new__(WeakSet)
try:
    obj.difference(_W())  # other: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_weakrefset/WeakSet__difference_update__other_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs__weakrefset_WeakSet__difference_update__other_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_weakrefset"
# dimension = "type"
# case = "WeakSet__difference_update__other_as_Iterable_wrong"
# subject = "_weakrefset.WeakSet.difference_update(other: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_weakrefset.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _weakrefset.WeakSet.difference_update(other: Iterable); call it with the wrong type.

typeshed contract: other is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _weakrefset import WeakSet
obj = object.__new__(WeakSet)
try:
    obj.difference_update(_W())  # other: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_weakrefset/WeakSet__intersection__other_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs__weakrefset_WeakSet__intersection__other_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_weakrefset"
# dimension = "type"
# case = "WeakSet__intersection__other_as_Iterable_wrong"
# subject = "_weakrefset.WeakSet.intersection(other: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_weakrefset.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _weakrefset.WeakSet.intersection(other: Iterable); call it with the wrong type.

typeshed contract: other is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _weakrefset import WeakSet
obj = object.__new__(WeakSet)
try:
    obj.intersection(_W())  # other: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_weakrefset/WeakSet__intersection_update__other_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs__weakrefset_WeakSet__intersection_update__other_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_weakrefset"
# dimension = "type"
# case = "WeakSet__intersection_update__other_as_Iterable_wrong"
# subject = "_weakrefset.WeakSet.intersection_update(other: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_weakrefset.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _weakrefset.WeakSet.intersection_update(other: Iterable); call it with the wrong type.

typeshed contract: other is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _weakrefset import WeakSet
obj = object.__new__(WeakSet)
try:
    obj.intersection_update(_W())  # other: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_weakrefset/WeakSet__isdisjoint__other_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs__weakrefset_WeakSet__isdisjoint__other_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_weakrefset"
# dimension = "type"
# case = "WeakSet__isdisjoint__other_as_Iterable_wrong"
# subject = "_weakrefset.WeakSet.isdisjoint(other: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_weakrefset.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _weakrefset.WeakSet.isdisjoint(other: Iterable); call it with the wrong type.

typeshed contract: other is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _weakrefset import WeakSet
obj = object.__new__(WeakSet)
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

/// Ported from `tests/cpython/type/std-libs/_weakrefset/WeakSet__issubset__other_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs__weakrefset_WeakSet__issubset__other_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_weakrefset"
# dimension = "type"
# case = "WeakSet__issubset__other_as_Iterable_wrong"
# subject = "_weakrefset.WeakSet.issubset(other: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_weakrefset.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _weakrefset.WeakSet.issubset(other: Iterable); call it with the wrong type.

typeshed contract: other is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _weakrefset import WeakSet
obj = object.__new__(WeakSet)
try:
    obj.issubset(_W())  # other: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_weakrefset/WeakSet__issuperset__other_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs__weakrefset_WeakSet__issuperset__other_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_weakrefset"
# dimension = "type"
# case = "WeakSet__issuperset__other_as_Iterable_wrong"
# subject = "_weakrefset.WeakSet.issuperset(other: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_weakrefset.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _weakrefset.WeakSet.issuperset(other: Iterable); call it with the wrong type.

typeshed contract: other is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _weakrefset import WeakSet
obj = object.__new__(WeakSet)
try:
    obj.issuperset(_W())  # other: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_weakrefset/WeakSet__symmetric_difference__other_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs__weakrefset_WeakSet__symmetric_difference__other_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_weakrefset"
# dimension = "type"
# case = "WeakSet__symmetric_difference__other_as_Iterable_wrong"
# subject = "_weakrefset.WeakSet.symmetric_difference(other: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_weakrefset.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _weakrefset.WeakSet.symmetric_difference(other: Iterable); call it with the wrong type.

typeshed contract: other is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _weakrefset import WeakSet
obj = object.__new__(WeakSet)
try:
    obj.symmetric_difference(_W())  # other: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_weakrefset/WeakSet__symmetric_difference_update__other_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs__weakrefset_WeakSet__symmetric_difference_update__other_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_weakrefset"
# dimension = "type"
# case = "WeakSet__symmetric_difference_update__other_as_Iterable_wrong"
# subject = "_weakrefset.WeakSet.symmetric_difference_update(other: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_weakrefset.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _weakrefset.WeakSet.symmetric_difference_update(other: Iterable); call it with the wrong type.

typeshed contract: other is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _weakrefset import WeakSet
obj = object.__new__(WeakSet)
try:
    obj.symmetric_difference_update(_W())  # other: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_weakrefset/WeakSet__union__other_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs__weakrefset_WeakSet__union__other_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_weakrefset"
# dimension = "type"
# case = "WeakSet__union__other_as_Iterable_wrong"
# subject = "_weakrefset.WeakSet.union(other: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_weakrefset.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _weakrefset.WeakSet.union(other: Iterable); call it with the wrong type.

typeshed contract: other is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _weakrefset import WeakSet
obj = object.__new__(WeakSet)
try:
    obj.union(_W())  # other: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_weakrefset/WeakSet__update__other_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs__weakrefset_WeakSet__update__other_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_weakrefset"
# dimension = "type"
# case = "WeakSet__update__other_as_Iterable_wrong"
# subject = "_weakrefset.WeakSet.update(other: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_weakrefset.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _weakrefset.WeakSet.update(other: Iterable); call it with the wrong type.

typeshed contract: other is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _weakrefset import WeakSet
obj = object.__new__(WeakSet)
try:
    obj.update(_W())  # other: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
