use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/_blake2/blake2b____new____data_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs__blake2_blake2b____new____data_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_blake2"
# dimension = "type"
# case = "blake2b____new____data_as_ReadableBuffer_wrong"
# subject = "_blake2.blake2b.__new__(data: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_blake2.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _blake2.blake2b.__new__(data: ReadableBuffer); call it with the wrong type.

typeshed contract: data is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _blake2 import blake2b
obj = object.__new__(blake2b)
try:
    obj.__new__(_W())  # data: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_blake2/blake2b__update__data_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs__blake2_blake2b__update__data_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_blake2"
# dimension = "type"
# case = "blake2b__update__data_as_ReadableBuffer_wrong"
# subject = "_blake2.blake2b.update(data: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_blake2.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _blake2.blake2b.update(data: ReadableBuffer); call it with the wrong type.

typeshed contract: data is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _blake2 import blake2b
obj = object.__new__(blake2b)
try:
    obj.update(_W())  # data: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_blake2/blake2s____new____data_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs__blake2_blake2s____new____data_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_blake2"
# dimension = "type"
# case = "blake2s____new____data_as_ReadableBuffer_wrong"
# subject = "_blake2.blake2s.__new__(data: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_blake2.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _blake2.blake2s.__new__(data: ReadableBuffer); call it with the wrong type.

typeshed contract: data is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _blake2 import blake2s
obj = object.__new__(blake2s)
try:
    obj.__new__(_W())  # data: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_blake2/blake2s__update__data_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs__blake2_blake2s__update__data_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_blake2"
# dimension = "type"
# case = "blake2s__update__data_as_ReadableBuffer_wrong"
# subject = "_blake2.blake2s.update(data: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_blake2.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _blake2.blake2s.update(data: ReadableBuffer); call it with the wrong type.

typeshed contract: data is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _blake2 import blake2s
obj = object.__new__(blake2s)
try:
    obj.update(_W())  # data: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
