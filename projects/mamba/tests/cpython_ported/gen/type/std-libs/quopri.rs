use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/quopri/decode__input_as__Input_wrong.py`.
#[test]
fn test_gen_type_std_libs_quopri_decode__input_as__Input_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "quopri"
# dimension = "type"
# case = "decode__input_as__Input_wrong"
# subject = "quopri.decode(input: _Input)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/quopri.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: quopri.decode(input: _Input); call it with the wrong type.

typeshed contract: input is _Input. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from quopri import decode
try:
    decode(_W(), None)  # input: _Input <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/quopri/decodestring__s_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_quopri_decodestring__s_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "quopri"
# dimension = "type"
# case = "decodestring__s_as_typed_wrong"
# subject = "quopri.decodestring(s: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/quopri.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: quopri.decodestring(s: typed); call it with the wrong type.

typeshed contract: s is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from quopri import decodestring
try:
    decodestring(_W())  # s: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/quopri/encode__input_as__Input_wrong.py`.
#[test]
fn test_gen_type_std_libs_quopri_encode__input_as__Input_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "quopri"
# dimension = "type"
# case = "encode__input_as__Input_wrong"
# subject = "quopri.encode(input: _Input)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/quopri.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: quopri.encode(input: _Input); call it with the wrong type.

typeshed contract: input is _Input. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from quopri import encode
try:
    encode(_W(), None, 0)  # input: _Input <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/quopri/encodestring__s_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_quopri_encodestring__s_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "quopri"
# dimension = "type"
# case = "encodestring__s_as_ReadableBuffer_wrong"
# subject = "quopri.encodestring(s: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/quopri.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: quopri.encodestring(s: ReadableBuffer); call it with the wrong type.

typeshed contract: s is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from quopri import encodestring
try:
    encodestring(_W())  # s: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
