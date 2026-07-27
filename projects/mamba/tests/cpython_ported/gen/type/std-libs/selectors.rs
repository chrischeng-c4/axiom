use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/selectors/BaseSelector__get_key__fileobj_as_FileDescriptorLike_wrong.py`.
#[test]
fn test_gen_type_std_libs_selectors_BaseSelector__get_key__fileobj_as_FileDescriptorLike_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "type"
# case = "BaseSelector__get_key__fileobj_as_FileDescriptorLike_wrong"
# subject = "selectors.BaseSelector.get_key(fileobj: FileDescriptorLike)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/selectors.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: selectors.BaseSelector.get_key(fileobj: FileDescriptorLike); call it with the wrong type.

typeshed contract: fileobj is FileDescriptorLike. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from selectors import BaseSelector
obj = object.__new__(BaseSelector)
try:
    obj.get_key(_W())  # fileobj: FileDescriptorLike <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/selectors/BaseSelector__modify__fileobj_as_FileDescriptorLike_wrong.py`.
#[test]
fn test_gen_type_std_libs_selectors_BaseSelector__modify__fileobj_as_FileDescriptorLike_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "type"
# case = "BaseSelector__modify__fileobj_as_FileDescriptorLike_wrong"
# subject = "selectors.BaseSelector.modify(fileobj: FileDescriptorLike)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/selectors.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: selectors.BaseSelector.modify(fileobj: FileDescriptorLike); call it with the wrong type.

typeshed contract: fileobj is FileDescriptorLike. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from selectors import BaseSelector
obj = object.__new__(BaseSelector)
try:
    obj.modify(_W(), 0)  # fileobj: FileDescriptorLike <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/selectors/BaseSelector__register__fileobj_as_FileDescriptorLike_wrong.py`.
#[test]
fn test_gen_type_std_libs_selectors_BaseSelector__register__fileobj_as_FileDescriptorLike_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "type"
# case = "BaseSelector__register__fileobj_as_FileDescriptorLike_wrong"
# subject = "selectors.BaseSelector.register(fileobj: FileDescriptorLike)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/selectors.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: selectors.BaseSelector.register(fileobj: FileDescriptorLike); call it with the wrong type.

typeshed contract: fileobj is FileDescriptorLike. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from selectors import BaseSelector
obj = object.__new__(BaseSelector)
try:
    obj.register(_W(), 0)  # fileobj: FileDescriptorLike <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/selectors/BaseSelector__select__timeout_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_selectors_BaseSelector__select__timeout_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "type"
# case = "BaseSelector__select__timeout_as_typed_wrong"
# subject = "selectors.BaseSelector.select(timeout: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/selectors.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: selectors.BaseSelector.select(timeout: typed); call it with the wrong type.

typeshed contract: timeout is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from selectors import BaseSelector
obj = object.__new__(BaseSelector)
try:
    obj.select(_W())  # timeout: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/selectors/BaseSelector__unregister__fileobj_as_FileDescriptorLike_wrong.py`.
#[test]
fn test_gen_type_std_libs_selectors_BaseSelector__unregister__fileobj_as_FileDescriptorLike_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "type"
# case = "BaseSelector__unregister__fileobj_as_FileDescriptorLike_wrong"
# subject = "selectors.BaseSelector.unregister(fileobj: FileDescriptorLike)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/selectors.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: selectors.BaseSelector.unregister(fileobj: FileDescriptorLike); call it with the wrong type.

typeshed contract: fileobj is FileDescriptorLike. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from selectors import BaseSelector
obj = object.__new__(BaseSelector)
try:
    obj.unregister(_W())  # fileobj: FileDescriptorLike <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/selectors/DefaultSelector__select__timeout_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_selectors_DefaultSelector__select__timeout_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "type"
# case = "DefaultSelector__select__timeout_as_typed_wrong"
# subject = "selectors.DefaultSelector.select(timeout: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/selectors.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: selectors.DefaultSelector.select(timeout: typed); call it with the wrong type.

typeshed contract: timeout is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from selectors import DefaultSelector
obj = object.__new__(DefaultSelector)
try:
    obj.select(_W())  # timeout: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/selectors/KqueueSelector__select__timeout_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_selectors_KqueueSelector__select__timeout_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "type"
# case = "KqueueSelector__select__timeout_as_typed_wrong"
# subject = "selectors.KqueueSelector.select(timeout: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/selectors.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: selectors.KqueueSelector.select(timeout: typed); call it with the wrong type.

typeshed contract: timeout is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from selectors import KqueueSelector
obj = object.__new__(KqueueSelector)
try:
    obj.select(_W())  # timeout: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/selectors/SelectSelector__select__timeout_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_selectors_SelectSelector__select__timeout_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "type"
# case = "SelectSelector__select__timeout_as_typed_wrong"
# subject = "selectors.SelectSelector.select(timeout: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/selectors.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: selectors.SelectSelector.select(timeout: typed); call it with the wrong type.

typeshed contract: timeout is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from selectors import SelectSelector
obj = object.__new__(SelectSelector)
try:
    obj.select(_W())  # timeout: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
