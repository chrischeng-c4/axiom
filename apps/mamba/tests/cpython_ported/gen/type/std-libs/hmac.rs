use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/hmac/HMAC__init__key_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_hmac_HMAC__init__key_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "type"
# case = "HMAC__init__key_as_typed_wrong"
# subject = "hmac.HMAC.__init__(key: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/hmac.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: hmac.HMAC.__init__(key: typed); call it with the wrong type.

typeshed contract: key is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from hmac import HMAC
try:
    HMAC(_W())  # key: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/hmac/HMAC__update__msg_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_hmac_HMAC__update__msg_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "type"
# case = "HMAC__update__msg_as_ReadableBuffer_wrong"
# subject = "hmac.HMAC.update(msg: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/hmac.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: hmac.HMAC.update(msg: ReadableBuffer); call it with the wrong type.

typeshed contract: msg is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from hmac import HMAC
obj = object.__new__(HMAC)
try:
    obj.update(_W())  # msg: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/hmac/digest__key_as_SizedBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_hmac_digest__key_as_SizedBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "type"
# case = "digest__key_as_SizedBuffer_wrong"
# subject = "hmac.digest(key: SizedBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/hmac.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: hmac.digest(key: SizedBuffer); call it with the wrong type.

typeshed contract: key is SizedBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from hmac import digest
try:
    digest(_W(), None, None)  # key: SizedBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/hmac/new__key_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_hmac_new__key_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "type"
# case = "new__key_as_typed_wrong"
# subject = "hmac.new(key: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/hmac.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: hmac.new(key: typed); call it with the wrong type.

typeshed contract: key is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from hmac import new
try:
    new(_W(), None, None)  # key: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
