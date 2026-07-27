use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/curses_ascii/isalnum__c_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_curses_ascii_isalnum__c_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "curses_ascii"
# dimension = "type"
# case = "isalnum__c_as_typed_wrong"
# subject = "curses.ascii.isalnum(c: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/curses/ascii.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: curses.ascii.isalnum(c: typed); call it with the wrong type.

typeshed contract: c is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from curses.ascii import isalnum
try:
    isalnum(_W())  # c: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/curses_ascii/isalpha__c_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_curses_ascii_isalpha__c_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "curses_ascii"
# dimension = "type"
# case = "isalpha__c_as_typed_wrong"
# subject = "curses.ascii.isalpha(c: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/curses/ascii.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: curses.ascii.isalpha(c: typed); call it with the wrong type.

typeshed contract: c is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from curses.ascii import isalpha
try:
    isalpha(_W())  # c: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/curses_ascii/isascii__c_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_curses_ascii_isascii__c_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "curses_ascii"
# dimension = "type"
# case = "isascii__c_as_typed_wrong"
# subject = "curses.ascii.isascii(c: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/curses/ascii.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: curses.ascii.isascii(c: typed); call it with the wrong type.

typeshed contract: c is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from curses.ascii import isascii
try:
    isascii(_W())  # c: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/curses_ascii/isblank__c_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_curses_ascii_isblank__c_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "curses_ascii"
# dimension = "type"
# case = "isblank__c_as_typed_wrong"
# subject = "curses.ascii.isblank(c: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/curses/ascii.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: curses.ascii.isblank(c: typed); call it with the wrong type.

typeshed contract: c is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from curses.ascii import isblank
try:
    isblank(_W())  # c: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/curses_ascii/iscntrl__c_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_curses_ascii_iscntrl__c_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "curses_ascii"
# dimension = "type"
# case = "iscntrl__c_as_typed_wrong"
# subject = "curses.ascii.iscntrl(c: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/curses/ascii.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: curses.ascii.iscntrl(c: typed); call it with the wrong type.

typeshed contract: c is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from curses.ascii import iscntrl
try:
    iscntrl(_W())  # c: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/curses_ascii/isctrl__c_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_curses_ascii_isctrl__c_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "curses_ascii"
# dimension = "type"
# case = "isctrl__c_as_typed_wrong"
# subject = "curses.ascii.isctrl(c: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/curses/ascii.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: curses.ascii.isctrl(c: typed); call it with the wrong type.

typeshed contract: c is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from curses.ascii import isctrl
try:
    isctrl(_W())  # c: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/curses_ascii/isdigit__c_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_curses_ascii_isdigit__c_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "curses_ascii"
# dimension = "type"
# case = "isdigit__c_as_typed_wrong"
# subject = "curses.ascii.isdigit(c: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/curses/ascii.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: curses.ascii.isdigit(c: typed); call it with the wrong type.

typeshed contract: c is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from curses.ascii import isdigit
try:
    isdigit(_W())  # c: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/curses_ascii/isgraph__c_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_curses_ascii_isgraph__c_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "curses_ascii"
# dimension = "type"
# case = "isgraph__c_as_typed_wrong"
# subject = "curses.ascii.isgraph(c: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/curses/ascii.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: curses.ascii.isgraph(c: typed); call it with the wrong type.

typeshed contract: c is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from curses.ascii import isgraph
try:
    isgraph(_W())  # c: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/curses_ascii/islower__c_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_curses_ascii_islower__c_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "curses_ascii"
# dimension = "type"
# case = "islower__c_as_typed_wrong"
# subject = "curses.ascii.islower(c: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/curses/ascii.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: curses.ascii.islower(c: typed); call it with the wrong type.

typeshed contract: c is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from curses.ascii import islower
try:
    islower(_W())  # c: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/curses_ascii/ismeta__c_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_curses_ascii_ismeta__c_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "curses_ascii"
# dimension = "type"
# case = "ismeta__c_as_typed_wrong"
# subject = "curses.ascii.ismeta(c: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/curses/ascii.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: curses.ascii.ismeta(c: typed); call it with the wrong type.

typeshed contract: c is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from curses.ascii import ismeta
try:
    ismeta(_W())  # c: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/curses_ascii/isprint__c_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_curses_ascii_isprint__c_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "curses_ascii"
# dimension = "type"
# case = "isprint__c_as_typed_wrong"
# subject = "curses.ascii.isprint(c: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/curses/ascii.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: curses.ascii.isprint(c: typed); call it with the wrong type.

typeshed contract: c is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from curses.ascii import isprint
try:
    isprint(_W())  # c: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/curses_ascii/ispunct__c_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_curses_ascii_ispunct__c_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "curses_ascii"
# dimension = "type"
# case = "ispunct__c_as_typed_wrong"
# subject = "curses.ascii.ispunct(c: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/curses/ascii.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: curses.ascii.ispunct(c: typed); call it with the wrong type.

typeshed contract: c is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from curses.ascii import ispunct
try:
    ispunct(_W())  # c: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/curses_ascii/isspace__c_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_curses_ascii_isspace__c_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "curses_ascii"
# dimension = "type"
# case = "isspace__c_as_typed_wrong"
# subject = "curses.ascii.isspace(c: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/curses/ascii.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: curses.ascii.isspace(c: typed); call it with the wrong type.

typeshed contract: c is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from curses.ascii import isspace
try:
    isspace(_W())  # c: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/curses_ascii/isupper__c_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_curses_ascii_isupper__c_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "curses_ascii"
# dimension = "type"
# case = "isupper__c_as_typed_wrong"
# subject = "curses.ascii.isupper(c: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/curses/ascii.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: curses.ascii.isupper(c: typed); call it with the wrong type.

typeshed contract: c is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from curses.ascii import isupper
try:
    isupper(_W())  # c: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/curses_ascii/isxdigit__c_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_curses_ascii_isxdigit__c_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "curses_ascii"
# dimension = "type"
# case = "isxdigit__c_as_typed_wrong"
# subject = "curses.ascii.isxdigit(c: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/curses/ascii.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: curses.ascii.isxdigit(c: typed); call it with the wrong type.

typeshed contract: c is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from curses.ascii import isxdigit
try:
    isxdigit(_W())  # c: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/curses_ascii/unctrl__c_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_curses_ascii_unctrl__c_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "curses_ascii"
# dimension = "type"
# case = "unctrl__c_as_typed_wrong"
# subject = "curses.ascii.unctrl(c: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/curses/ascii.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: curses.ascii.unctrl(c: typed); call it with the wrong type.

typeshed contract: c is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from curses.ascii import unctrl
try:
    unctrl(_W())  # c: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
