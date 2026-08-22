use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/http_cookies/BaseCookie____setitem____key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_cookies_BaseCookie____setitem____key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "type"
# case = "BaseCookie____setitem____key_as_str_wrong"
# subject = "http.cookies.BaseCookie.__setitem__(key: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookies.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookies.BaseCookie.__setitem__(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from http.cookies import BaseCookie
obj = object.__new__(BaseCookie)
try:
    obj.__setitem__(12345, None)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_cookies/BaseCookie__init__input_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_cookies_BaseCookie__init__input_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "type"
# case = "BaseCookie__init__input_as_typed_wrong"
# subject = "http.cookies.BaseCookie.__init__(input: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookies.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookies.BaseCookie.__init__(input: typed); call it with the wrong type.

typeshed contract: input is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.cookies import BaseCookie
try:
    BaseCookie(_W())  # input: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_cookies/BaseCookie__js_output__attrs_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_cookies_BaseCookie__js_output__attrs_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "type"
# case = "BaseCookie__js_output__attrs_as_typed_wrong"
# subject = "http.cookies.BaseCookie.js_output(attrs: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookies.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookies.BaseCookie.js_output(attrs: typed); call it with the wrong type.

typeshed contract: attrs is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.cookies import BaseCookie
obj = object.__new__(BaseCookie)
try:
    obj.js_output(_W())  # attrs: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_cookies/BaseCookie__load__rawdata_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_cookies_BaseCookie__load__rawdata_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "type"
# case = "BaseCookie__load__rawdata_as_typed_wrong"
# subject = "http.cookies.BaseCookie.load(rawdata: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookies.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookies.BaseCookie.load(rawdata: typed); call it with the wrong type.

typeshed contract: rawdata is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.cookies import BaseCookie
obj = object.__new__(BaseCookie)
try:
    obj.load(_W())  # rawdata: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_cookies/BaseCookie__output__attrs_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_cookies_BaseCookie__output__attrs_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "type"
# case = "BaseCookie__output__attrs_as_typed_wrong"
# subject = "http.cookies.BaseCookie.output(attrs: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookies.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookies.BaseCookie.output(attrs: typed); call it with the wrong type.

typeshed contract: attrs is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.cookies import BaseCookie
obj = object.__new__(BaseCookie)
try:
    obj.output(_W())  # attrs: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_cookies/BaseCookie__value_decode__val_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_cookies_BaseCookie__value_decode__val_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "type"
# case = "BaseCookie__value_decode__val_as_str_wrong"
# subject = "http.cookies.BaseCookie.value_decode(val: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookies.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookies.BaseCookie.value_decode(val: str); call it with the wrong type.

typeshed contract: val is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from http.cookies import BaseCookie
obj = object.__new__(BaseCookie)
try:
    obj.value_decode(12345)  # val: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_cookies/Morsel__OutputString__attrs_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_cookies_Morsel__OutputString__attrs_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "type"
# case = "Morsel__OutputString__attrs_as_typed_wrong"
# subject = "http.cookies.Morsel.OutputString(attrs: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookies.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookies.Morsel.OutputString(attrs: typed); call it with the wrong type.

typeshed contract: attrs is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.cookies import Morsel
obj = object.__new__(Morsel)
try:
    obj.OutputString(_W())  # attrs: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_cookies/Morsel____setitem____K_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_cookies_Morsel____setitem____K_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "type"
# case = "Morsel____setitem____K_as_str_wrong"
# subject = "http.cookies.Morsel.__setitem__(K: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookies.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookies.Morsel.__setitem__(K: str); call it with the wrong type.

typeshed contract: K is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from http.cookies import Morsel
obj = object.__new__(Morsel)
try:
    obj.__setitem__(12345, None)  # K: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_cookies/Morsel__isReservedKey__K_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_cookies_Morsel__isReservedKey__K_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "type"
# case = "Morsel__isReservedKey__K_as_str_wrong"
# subject = "http.cookies.Morsel.isReservedKey(K: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookies.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookies.Morsel.isReservedKey(K: str); call it with the wrong type.

typeshed contract: K is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from http.cookies import Morsel
obj = object.__new__(Morsel)
try:
    obj.isReservedKey(12345)  # K: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_cookies/Morsel__js_output__attrs_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_cookies_Morsel__js_output__attrs_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "type"
# case = "Morsel__js_output__attrs_as_typed_wrong"
# subject = "http.cookies.Morsel.js_output(attrs: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookies.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookies.Morsel.js_output(attrs: typed); call it with the wrong type.

typeshed contract: attrs is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.cookies import Morsel
obj = object.__new__(Morsel)
try:
    obj.js_output(_W())  # attrs: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_cookies/Morsel__output__attrs_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_cookies_Morsel__output__attrs_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "type"
# case = "Morsel__output__attrs_as_typed_wrong"
# subject = "http.cookies.Morsel.output(attrs: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookies.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookies.Morsel.output(attrs: typed); call it with the wrong type.

typeshed contract: attrs is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.cookies import Morsel
obj = object.__new__(Morsel)
try:
    obj.output(_W())  # attrs: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_cookies/Morsel__set__key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_cookies_Morsel__set__key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "type"
# case = "Morsel__set__key_as_str_wrong"
# subject = "http.cookies.Morsel.set(key: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookies.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookies.Morsel.set(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from http.cookies import Morsel
obj = object.__new__(Morsel)
try:
    obj.set(12345, "", None)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_cookies/Morsel__setdefault__key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_cookies_Morsel__setdefault__key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "type"
# case = "Morsel__setdefault__key_as_str_wrong"
# subject = "http.cookies.Morsel.setdefault(key: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookies.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookies.Morsel.setdefault(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from http.cookies import Morsel
obj = object.__new__(Morsel)
try:
    obj.setdefault(12345)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_cookies/Morsel__update__values_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_cookies_Morsel__update__values_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "type"
# case = "Morsel__update__values_as_typed_wrong"
# subject = "http.cookies.Morsel.update(values: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookies.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookies.Morsel.update(values: typed); call it with the wrong type.

typeshed contract: values is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.cookies import Morsel
obj = object.__new__(Morsel)
try:
    obj.update(_W())  # values: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
