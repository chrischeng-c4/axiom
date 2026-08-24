use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/_json/encode_basestring__s_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__json_encode_basestring__s_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_json"
# dimension = "type"
# case = "encode_basestring__s_as_str_wrong"
# subject = "_json.encode_basestring(s: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_json.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _json.encode_basestring(s: str); call it with the wrong type.

typeshed contract: s is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _json import encode_basestring
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

/// Ported from `tests/cpython/type/std-libs/_json/encode_basestring_ascii__s_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__json_encode_basestring_ascii__s_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_json"
# dimension = "type"
# case = "encode_basestring_ascii__s_as_str_wrong"
# subject = "_json.encode_basestring_ascii(s: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_json.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _json.encode_basestring_ascii(s: str); call it with the wrong type.

typeshed contract: s is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _json import encode_basestring_ascii
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

/// Ported from `tests/cpython/type/std-libs/_json/make_encoder____call_____current_indent_level_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__json_make_encoder____call_____current_indent_level_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_json"
# dimension = "type"
# case = "make_encoder____call_____current_indent_level_as_int_wrong"
# subject = "_json.make_encoder.__call__(_current_indent_level: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_json.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _json.make_encoder.__call__(_current_indent_level: int); call it with the wrong type.

typeshed contract: _current_indent_level is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _json import make_encoder
obj = object.__new__(make_encoder)
try:
    obj.__call__(None, "not_an_int")  # _current_indent_level: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_json/make_encoder____new____markers_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs__json_make_encoder____new____markers_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_json"
# dimension = "type"
# case = "make_encoder____new____markers_as_typed_wrong"
# subject = "_json.make_encoder.__new__(markers: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_json.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _json.make_encoder.__new__(markers: typed); call it with the wrong type.

typeshed contract: markers is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _json import make_encoder
obj = object.__new__(make_encoder)
try:
    obj.__new__(_W(), None, None, None, "", "", True, True, True)  # markers: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_json/make_scanner____call____string_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__json_make_scanner____call____string_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_json"
# dimension = "type"
# case = "make_scanner____call____string_as_str_wrong"
# subject = "_json.make_scanner.__call__(string: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_json.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _json.make_scanner.__call__(string: str); call it with the wrong type.

typeshed contract: string is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _json import make_scanner
obj = object.__new__(make_scanner)
try:
    obj.__call__(12345, 0)  # string: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_json/make_scanner____new____context_as_make_scanner_wrong.py`.
#[test]
fn test_gen_type_std_libs__json_make_scanner____new____context_as_make_scanner_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_json"
# dimension = "type"
# case = "make_scanner____new____context_as_make_scanner_wrong"
# subject = "_json.make_scanner.__new__(context: make_scanner)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_json.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _json.make_scanner.__new__(context: make_scanner); call it with the wrong type.

typeshed contract: context is make_scanner. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _json import make_scanner
obj = object.__new__(make_scanner)
try:
    obj.__new__(_W())  # context: make_scanner <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_json/scanstring__string_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__json_scanstring__string_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_json"
# dimension = "type"
# case = "scanstring__string_as_str_wrong"
# subject = "_json.scanstring(string: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_json.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _json.scanstring(string: str); call it with the wrong type.

typeshed contract: string is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _json import scanstring
try:
    scanstring(12345, 0)  # string: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/json/detect_encoding__b_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_json_detect_encoding__b_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "type"
# case = "detect_encoding__b_as_typed_wrong"
# subject = "json.detect_encoding(b: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/json.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: json.detect_encoding(b: typed); call it with the wrong type.

typeshed contract: b is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from json import detect_encoding
try:
    detect_encoding(_W())  # b: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/json/dump__fp_as_SupportsWrite_wrong.py`.
#[test]
fn test_gen_type_std_libs_json_dump__fp_as_SupportsWrite_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "type"
# case = "dump__fp_as_SupportsWrite_wrong"
# subject = "json.dump(fp: SupportsWrite)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/json.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: json.dump(fp: SupportsWrite); call it with the wrong type.

typeshed contract: fp is SupportsWrite. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from json import dump
try:
    dump(None, _W())  # fp: SupportsWrite <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/json/load__fp_as_SupportsRead_wrong.py`.
#[test]
fn test_gen_type_std_libs_json_load__fp_as_SupportsRead_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "type"
# case = "load__fp_as_SupportsRead_wrong"
# subject = "json.load(fp: SupportsRead)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/json.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: json.load(fp: SupportsRead); call it with the wrong type.

typeshed contract: fp is SupportsRead. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from json import load
try:
    load(_W())  # fp: SupportsRead <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/json/loads__s_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_json_loads__s_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "type"
# case = "loads__s_as_typed_wrong"
# subject = "json.loads(s: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/json.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: json.loads(s: typed); call it with the wrong type.

typeshed contract: s is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from json import loads
try:
    loads(_W())  # s: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
