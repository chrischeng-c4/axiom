use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/cgi/FieldStorage____contains____key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_cgi_FieldStorage____contains____key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cgi"
# dimension = "type"
# case = "FieldStorage____contains____key_as_str_wrong"
# subject = "cgi.FieldStorage.__contains__(key: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cgi.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cgi.FieldStorage.__contains__(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from cgi import FieldStorage
obj = object.__new__(FieldStorage)
try:
    obj.__contains__(12345)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cgi/FieldStorage____getitem____key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_cgi_FieldStorage____getitem____key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cgi"
# dimension = "type"
# case = "FieldStorage____getitem____key_as_str_wrong"
# subject = "cgi.FieldStorage.__getitem__(key: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cgi.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cgi.FieldStorage.__getitem__(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from cgi import FieldStorage
obj = object.__new__(FieldStorage)
try:
    obj.__getitem__(12345)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cgi/FieldStorage__getfirst__key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_cgi_FieldStorage__getfirst__key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cgi"
# dimension = "type"
# case = "FieldStorage__getfirst__key_as_str_wrong"
# subject = "cgi.FieldStorage.getfirst(key: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cgi.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cgi.FieldStorage.getfirst(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from cgi import FieldStorage
obj = object.__new__(FieldStorage)
try:
    obj.getfirst(12345)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cgi/FieldStorage__getlist__key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_cgi_FieldStorage__getlist__key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cgi"
# dimension = "type"
# case = "FieldStorage__getlist__key_as_str_wrong"
# subject = "cgi.FieldStorage.getlist(key: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cgi.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cgi.FieldStorage.getlist(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from cgi import FieldStorage
obj = object.__new__(FieldStorage)
try:
    obj.getlist(12345)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cgi/FieldStorage__getvalue__key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_cgi_FieldStorage__getvalue__key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cgi"
# dimension = "type"
# case = "FieldStorage__getvalue__key_as_str_wrong"
# subject = "cgi.FieldStorage.getvalue(key: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cgi.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cgi.FieldStorage.getvalue(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from cgi import FieldStorage
obj = object.__new__(FieldStorage)
try:
    obj.getvalue(12345)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cgi/FieldStorage__init__fp_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_cgi_FieldStorage__init__fp_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cgi"
# dimension = "type"
# case = "FieldStorage__init__fp_as_typed_wrong"
# subject = "cgi.FieldStorage.__init__(fp: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cgi.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cgi.FieldStorage.__init__(fp: typed); call it with the wrong type.

typeshed contract: fp is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from cgi import FieldStorage
try:
    FieldStorage(_W())  # fp: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cgi/parse__fp_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_cgi_parse__fp_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cgi"
# dimension = "type"
# case = "parse__fp_as_typed_wrong"
# subject = "cgi.parse(fp: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cgi.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cgi.parse(fp: typed); call it with the wrong type.

typeshed contract: fp is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from cgi import parse
try:
    parse(_W())  # fp: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cgi/parse_header__line_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_cgi_parse_header__line_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cgi"
# dimension = "type"
# case = "parse_header__line_as_str_wrong"
# subject = "cgi.parse_header(line: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cgi.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cgi.parse_header(line: str); call it with the wrong type.

typeshed contract: line is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from cgi import parse_header
try:
    parse_header(12345)  # line: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cgi/parse_multipart__fp_as_IO_wrong.py`.
#[test]
fn test_gen_type_std_libs_cgi_parse_multipart__fp_as_IO_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cgi"
# dimension = "type"
# case = "parse_multipart__fp_as_IO_wrong"
# subject = "cgi.parse_multipart(fp: IO)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cgi.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cgi.parse_multipart(fp: IO); call it with the wrong type.

typeshed contract: fp is IO. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from cgi import parse_multipart
try:
    parse_multipart(_W(), None)  # fp: IO <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cgi/print_environ__environ_as__Environ_wrong.py`.
#[test]
fn test_gen_type_std_libs_cgi_print_environ__environ_as__Environ_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cgi"
# dimension = "type"
# case = "print_environ__environ_as__Environ_wrong"
# subject = "cgi.print_environ(environ: _Environ)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cgi.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cgi.print_environ(environ: _Environ); call it with the wrong type.

typeshed contract: environ is _Environ. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from cgi import print_environ
try:
    print_environ(_W())  # environ: _Environ <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cgi/print_exception__type_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_cgi_print_exception__type_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cgi"
# dimension = "type"
# case = "print_exception__type_as_typed_wrong"
# subject = "cgi.print_exception(type: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cgi.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cgi.print_exception(type: typed); call it with the wrong type.

typeshed contract: type is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from cgi import print_exception
try:
    print_exception(_W())  # type: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cgi/print_form__form_as_dict_wrong.py`.
#[test]
fn test_gen_type_std_libs_cgi_print_form__form_as_dict_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cgi"
# dimension = "type"
# case = "print_form__form_as_dict_wrong"
# subject = "cgi.print_form(form: dict)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cgi.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cgi.print_form(form: dict); call it with the wrong type.

typeshed contract: form is dict. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from cgi import print_form
try:
    print_form(12345)  # form: dict <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cgi/test__environ_as__Environ_wrong.py`.
#[test]
fn test_gen_type_std_libs_cgi_test__environ_as__Environ_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cgi"
# dimension = "type"
# case = "test__environ_as__Environ_wrong"
# subject = "cgi.test(environ: _Environ)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cgi.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cgi.test(environ: _Environ); call it with the wrong type.

typeshed contract: environ is _Environ. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from cgi import test
try:
    test(_W())  # environ: _Environ <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
