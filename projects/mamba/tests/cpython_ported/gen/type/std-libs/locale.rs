use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/_locale/bind_textdomain_codeset__domain_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__locale_bind_textdomain_codeset__domain_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_locale"
# dimension = "type"
# case = "bind_textdomain_codeset__domain_as_str_wrong"
# subject = "_locale.bind_textdomain_codeset(domain: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_locale.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _locale.bind_textdomain_codeset(domain: str); call it with the wrong type.

typeshed contract: domain is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _locale import bind_textdomain_codeset
try:
    bind_textdomain_codeset(12345, None)  # domain: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_locale/bindtextdomain__domain_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__locale_bindtextdomain__domain_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_locale"
# dimension = "type"
# case = "bindtextdomain__domain_as_str_wrong"
# subject = "_locale.bindtextdomain(domain: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_locale.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _locale.bindtextdomain(domain: str); call it with the wrong type.

typeshed contract: domain is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _locale import bindtextdomain
try:
    bindtextdomain(12345, None)  # domain: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_locale/dcgettext__domain_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs__locale_dcgettext__domain_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_locale"
# dimension = "type"
# case = "dcgettext__domain_as_typed_wrong"
# subject = "_locale.dcgettext(domain: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_locale.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _locale.dcgettext(domain: typed); call it with the wrong type.

typeshed contract: domain is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _locale import dcgettext
try:
    dcgettext(_W(), "", 0)  # domain: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_locale/dgettext__domain_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs__locale_dgettext__domain_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_locale"
# dimension = "type"
# case = "dgettext__domain_as_typed_wrong"
# subject = "_locale.dgettext(domain: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_locale.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _locale.dgettext(domain: typed); call it with the wrong type.

typeshed contract: domain is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _locale import dgettext
try:
    dgettext(_W(), "")  # domain: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_locale/gettext__msg_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__locale_gettext__msg_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_locale"
# dimension = "type"
# case = "gettext__msg_as_str_wrong"
# subject = "_locale.gettext(msg: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_locale.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _locale.gettext(msg: str); call it with the wrong type.

typeshed contract: msg is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _locale import gettext
try:
    gettext(12345)  # msg: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_locale/nl_langinfo__key_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__locale_nl_langinfo__key_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_locale"
# dimension = "type"
# case = "nl_langinfo__key_as_int_wrong"
# subject = "_locale.nl_langinfo(key: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_locale.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _locale.nl_langinfo(key: int); call it with the wrong type.

typeshed contract: key is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _locale import nl_langinfo
try:
    nl_langinfo("not_an_int")  # key: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_locale/setlocale__category_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__locale_setlocale__category_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_locale"
# dimension = "type"
# case = "setlocale__category_as_int_wrong"
# subject = "_locale.setlocale(category: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_locale.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _locale.setlocale(category: int); call it with the wrong type.

typeshed contract: category is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _locale import setlocale
try:
    setlocale("not_an_int")  # category: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_locale/strcoll__os1_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__locale_strcoll__os1_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_locale"
# dimension = "type"
# case = "strcoll__os1_as_str_wrong"
# subject = "_locale.strcoll(os1: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_locale.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _locale.strcoll(os1: str); call it with the wrong type.

typeshed contract: os1 is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _locale import strcoll
try:
    strcoll(12345, "")  # os1: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_locale/strxfrm__string_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__locale_strxfrm__string_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_locale"
# dimension = "type"
# case = "strxfrm__string_as_str_wrong"
# subject = "_locale.strxfrm(string: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_locale.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _locale.strxfrm(string: str); call it with the wrong type.

typeshed contract: string is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _locale import strxfrm
try:
    strxfrm(12345)  # string: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_locale/textdomain__domain_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs__locale_textdomain__domain_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_locale"
# dimension = "type"
# case = "textdomain__domain_as_typed_wrong"
# subject = "_locale.textdomain(domain: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_locale.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _locale.textdomain(domain: typed); call it with the wrong type.

typeshed contract: domain is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _locale import textdomain
try:
    textdomain(_W())  # domain: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/locale/atof__string_as__str_wrong.py`.
#[test]
fn test_gen_type_std_libs_locale_atof__string_as__str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "type"
# case = "atof__string_as__str_wrong"
# subject = "locale.atof(string: _str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/locale.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: locale.atof(string: _str); call it with the wrong type.

typeshed contract: string is _str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from locale import atof
try:
    atof(_W())  # string: _str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/locale/atoi__string_as__str_wrong.py`.
#[test]
fn test_gen_type_std_libs_locale_atoi__string_as__str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "type"
# case = "atoi__string_as__str_wrong"
# subject = "locale.atoi(string: _str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/locale.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: locale.atoi(string: _str); call it with the wrong type.

typeshed contract: string is _str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from locale import atoi
try:
    atoi(_W())  # string: _str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/locale/currency__val_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_locale_currency__val_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "type"
# case = "currency__val_as_typed_wrong"
# subject = "locale.currency(val: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/locale.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: locale.currency(val: typed); call it with the wrong type.

typeshed contract: val is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from locale import currency
try:
    currency(_W())  # val: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/locale/delocalize__string_as__str_wrong.py`.
#[test]
fn test_gen_type_std_libs_locale_delocalize__string_as__str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "type"
# case = "delocalize__string_as__str_wrong"
# subject = "locale.delocalize(string: _str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/locale.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: locale.delocalize(string: _str); call it with the wrong type.

typeshed contract: string is _str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from locale import delocalize
try:
    delocalize(_W())  # string: _str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/locale/format__percent_as__str_wrong.py`.
#[test]
fn test_gen_type_std_libs_locale_format__percent_as__str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "type"
# case = "format__percent_as__str_wrong"
# subject = "locale.format(percent: _str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/locale.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: locale.format(percent: _str); call it with the wrong type.

typeshed contract: percent is _str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from locale import format
try:
    format(_W(), None)  # percent: _str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/locale/format_string__f_as__str_wrong.py`.
#[test]
fn test_gen_type_std_libs_locale_format_string__f_as__str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "type"
# case = "format_string__f_as__str_wrong"
# subject = "locale.format_string(f: _str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/locale.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: locale.format_string(f: _str); call it with the wrong type.

typeshed contract: f is _str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from locale import format_string
try:
    format_string(_W(), None)  # f: _str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/locale/getdefaultlocale__envvars_as_tuple_wrong.py`.
#[test]
fn test_gen_type_std_libs_locale_getdefaultlocale__envvars_as_tuple_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "type"
# case = "getdefaultlocale__envvars_as_tuple_wrong"
# subject = "locale.getdefaultlocale(envvars: tuple)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/locale.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: locale.getdefaultlocale(envvars: tuple); call it with the wrong type.

typeshed contract: envvars is tuple. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from locale import getdefaultlocale
try:
    getdefaultlocale(12345)  # envvars: tuple <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/locale/getlocale__category_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_locale_getlocale__category_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "type"
# case = "getlocale__category_as_int_wrong"
# subject = "locale.getlocale(category: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/locale.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: locale.getlocale(category: int); call it with the wrong type.

typeshed contract: category is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from locale import getlocale
try:
    getlocale("not_an_int")  # category: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/locale/getpreferredencoding__do_setlocale_as_bool_wrong.py`.
#[test]
fn test_gen_type_std_libs_locale_getpreferredencoding__do_setlocale_as_bool_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "type"
# case = "getpreferredencoding__do_setlocale_as_bool_wrong"
# subject = "locale.getpreferredencoding(do_setlocale: bool)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/locale.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: locale.getpreferredencoding(do_setlocale: bool); call it with the wrong type.

typeshed contract: do_setlocale is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from locale import getpreferredencoding
try:
    getpreferredencoding("not_a_bool")  # do_setlocale: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/locale/localize__string_as__str_wrong.py`.
#[test]
fn test_gen_type_std_libs_locale_localize__string_as__str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "type"
# case = "localize__string_as__str_wrong"
# subject = "locale.localize(string: _str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/locale.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: locale.localize(string: _str); call it with the wrong type.

typeshed contract: string is _str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from locale import localize
try:
    localize(_W())  # string: _str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/locale/normalize__localename_as__str_wrong.py`.
#[test]
fn test_gen_type_std_libs_locale_normalize__localename_as__str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "type"
# case = "normalize__localename_as__str_wrong"
# subject = "locale.normalize(localename: _str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/locale.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: locale.normalize(localename: _str); call it with the wrong type.

typeshed contract: localename is _str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from locale import normalize
try:
    normalize(_W())  # localename: _str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/locale/resetlocale__category_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_locale_resetlocale__category_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "type"
# case = "resetlocale__category_as_int_wrong"
# subject = "locale.resetlocale(category: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/locale.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: locale.resetlocale(category: int); call it with the wrong type.

typeshed contract: category is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from locale import resetlocale
try:
    resetlocale("not_an_int")  # category: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/locale/setlocale__category_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_locale_setlocale__category_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "type"
# case = "setlocale__category_as_int_wrong"
# subject = "locale.setlocale(category: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/locale.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: locale.setlocale(category: int); call it with the wrong type.

typeshed contract: category is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from locale import setlocale
try:
    setlocale("not_an_int")  # category: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/locale/str__val_as_float_wrong.py`.
#[test]
fn test_gen_type_std_libs_locale_str__val_as_float_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "type"
# case = "str__val_as_float_wrong"
# subject = "locale.str(val: float)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/locale.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: locale.str(val: float); call it with the wrong type.

typeshed contract: val is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from locale import str
try:
    str("not_a_float")  # val: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
