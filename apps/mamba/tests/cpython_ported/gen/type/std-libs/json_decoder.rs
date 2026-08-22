use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/json_decoder/JSONDecodeError__init__msg_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_json_decoder_JSONDecodeError__init__msg_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json_decoder"
# dimension = "type"
# case = "JSONDecodeError__init__msg_as_str_wrong"
# subject = "json.decoder.JSONDecodeError.__init__(msg: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/json/decoder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: json.decoder.JSONDecodeError.__init__(msg: str); call it with the wrong type.

typeshed contract: msg is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from json.decoder import JSONDecodeError
try:
    JSONDecodeError(12345, "", 0)  # msg: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/json_decoder/JSONDecoder__decode__s_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_json_decoder_JSONDecoder__decode__s_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json_decoder"
# dimension = "type"
# case = "JSONDecoder__decode__s_as_str_wrong"
# subject = "json.decoder.JSONDecoder.decode(s: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/json/decoder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: json.decoder.JSONDecoder.decode(s: str); call it with the wrong type.

typeshed contract: s is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from json.decoder import JSONDecoder
obj = object.__new__(JSONDecoder)
try:
    obj.decode(12345)  # s: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/json_decoder/JSONDecoder__raw_decode__s_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_json_decoder_JSONDecoder__raw_decode__s_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json_decoder"
# dimension = "type"
# case = "JSONDecoder__raw_decode__s_as_str_wrong"
# subject = "json.decoder.JSONDecoder.raw_decode(s: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/json/decoder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: json.decoder.JSONDecoder.raw_decode(s: str); call it with the wrong type.

typeshed contract: s is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from json.decoder import JSONDecoder
obj = object.__new__(JSONDecoder)
try:
    obj.raw_decode(12345)  # s: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
