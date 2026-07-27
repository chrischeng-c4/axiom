use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/marshal/dump__value_as__Marshallable_wrong.py`.
#[test]
fn test_gen_type_std_libs_marshal_dump__value_as__Marshallable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "marshal"
# dimension = "type"
# case = "dump__value_as__Marshallable_wrong"
# subject = "marshal.dump(value: _Marshallable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/marshal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: marshal.dump(value: _Marshallable); call it with the wrong type.

typeshed contract: value is _Marshallable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from marshal import dump
try:
    dump(_W(), None)  # value: _Marshallable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/marshal/dumps__value_as__Marshallable_wrong.py`.
#[test]
fn test_gen_type_std_libs_marshal_dumps__value_as__Marshallable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "marshal"
# dimension = "type"
# case = "dumps__value_as__Marshallable_wrong"
# subject = "marshal.dumps(value: _Marshallable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/marshal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: marshal.dumps(value: _Marshallable); call it with the wrong type.

typeshed contract: value is _Marshallable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from marshal import dumps
try:
    dumps(_W())  # value: _Marshallable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/marshal/load__file_as_SupportsRead_wrong.py`.
#[test]
fn test_gen_type_std_libs_marshal_load__file_as_SupportsRead_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "marshal"
# dimension = "type"
# case = "load__file_as_SupportsRead_wrong"
# subject = "marshal.load(file: SupportsRead)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/marshal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: marshal.load(file: SupportsRead); call it with the wrong type.

typeshed contract: file is SupportsRead. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from marshal import load
try:
    load(_W())  # file: SupportsRead <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/marshal/loads__bytes_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_marshal_loads__bytes_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "marshal"
# dimension = "type"
# case = "loads__bytes_as_ReadableBuffer_wrong"
# subject = "marshal.loads(bytes: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/marshal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: marshal.loads(bytes: ReadableBuffer); call it with the wrong type.

typeshed contract: bytes is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from marshal import loads
try:
    loads(_W())  # bytes: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
