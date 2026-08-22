use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/json_encoder/JSONEncoder__iterencode___one_shot_as_bool_wrong.py`.
#[test]
fn test_gen_type_std_libs_json_encoder_JSONEncoder__iterencode___one_shot_as_bool_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json_encoder"
# dimension = "type"
# case = "JSONEncoder__iterencode___one_shot_as_bool_wrong"
# subject = "json.encoder.JSONEncoder.iterencode(_one_shot: bool)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/json/encoder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: json.encoder.JSONEncoder.iterencode(_one_shot: bool); call it with the wrong type.

typeshed contract: _one_shot is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from json.encoder import JSONEncoder
obj = object.__new__(JSONEncoder)
try:
    obj.iterencode(None, "not_a_bool")  # _one_shot: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/json_encoder/encode_basestring__s_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_json_encoder_encode_basestring__s_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json_encoder"
# dimension = "type"
# case = "encode_basestring__s_as_str_wrong"
# subject = "json.encoder.encode_basestring(s: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/json/encoder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: json.encoder.encode_basestring(s: str); call it with the wrong type.

typeshed contract: s is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from json.encoder import encode_basestring
try:
    encode_basestring(12345)  # s: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/json_encoder/encode_basestring_ascii__s_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_json_encoder_encode_basestring_ascii__s_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json_encoder"
# dimension = "type"
# case = "encode_basestring_ascii__s_as_str_wrong"
# subject = "json.encoder.encode_basestring_ascii(s: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/json/encoder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: json.encoder.encode_basestring_ascii(s: str); call it with the wrong type.

typeshed contract: s is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from json.encoder import encode_basestring_ascii
try:
    encode_basestring_ascii(12345)  # s: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/json_encoder/py_encode_basestring__s_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_json_encoder_py_encode_basestring__s_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json_encoder"
# dimension = "type"
# case = "py_encode_basestring__s_as_str_wrong"
# subject = "json.encoder.py_encode_basestring(s: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/json/encoder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: json.encoder.py_encode_basestring(s: str); call it with the wrong type.

typeshed contract: s is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from json.encoder import py_encode_basestring
try:
    py_encode_basestring(12345)  # s: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/json_encoder/py_encode_basestring_ascii__s_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_json_encoder_py_encode_basestring_ascii__s_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json_encoder"
# dimension = "type"
# case = "py_encode_basestring_ascii__s_as_str_wrong"
# subject = "json.encoder.py_encode_basestring_ascii(s: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/json/encoder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: json.encoder.py_encode_basestring_ascii(s: str); call it with the wrong type.

typeshed contract: s is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from json.encoder import py_encode_basestring_ascii
try:
    py_encode_basestring_ascii(12345)  # s: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
