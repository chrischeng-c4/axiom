use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/poplib/POP3_SSL__init__host_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_poplib_POP3_SSL__init__host_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "poplib"
# dimension = "type"
# case = "POP3_SSL__init__host_as_str_wrong"
# subject = "poplib.POP3_SSL.__init__(host: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/poplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: poplib.POP3_SSL.__init__(host: str); call it with the wrong type.

typeshed contract: host is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from poplib import POP3_SSL
try:
    POP3_SSL(12345)  # host: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/poplib/POP3__apop__user_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_poplib_POP3__apop__user_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "poplib"
# dimension = "type"
# case = "POP3__apop__user_as_str_wrong"
# subject = "poplib.POP3.apop(user: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/poplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: poplib.POP3.apop(user: str); call it with the wrong type.

typeshed contract: user is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from poplib import POP3
obj = object.__new__(POP3)
try:
    obj.apop(12345, "")  # user: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/poplib/POP3__init__host_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_poplib_POP3__init__host_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "poplib"
# dimension = "type"
# case = "POP3__init__host_as_str_wrong"
# subject = "poplib.POP3.__init__(host: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/poplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: poplib.POP3.__init__(host: str); call it with the wrong type.

typeshed contract: host is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from poplib import POP3
try:
    POP3(12345)  # host: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/poplib/POP3__list__which_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_poplib_POP3__list__which_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "poplib"
# dimension = "type"
# case = "POP3__list__which_as_typed_wrong"
# subject = "poplib.POP3.list(which: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/poplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: poplib.POP3.list(which: typed); call it with the wrong type.

typeshed contract: which is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from poplib import POP3
obj = object.__new__(POP3)
try:
    obj.list(_W())  # which: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/poplib/POP3__pass___pswd_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_poplib_POP3__pass___pswd_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "poplib"
# dimension = "type"
# case = "POP3__pass___pswd_as_str_wrong"
# subject = "poplib.POP3.pass_(pswd: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/poplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: poplib.POP3.pass_(pswd: str); call it with the wrong type.

typeshed contract: pswd is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from poplib import POP3
obj = object.__new__(POP3)
try:
    obj.pass_(12345)  # pswd: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/poplib/POP3__rpop__user_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_poplib_POP3__rpop__user_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "poplib"
# dimension = "type"
# case = "POP3__rpop__user_as_str_wrong"
# subject = "poplib.POP3.rpop(user: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/poplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: poplib.POP3.rpop(user: str); call it with the wrong type.

typeshed contract: user is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from poplib import POP3
obj = object.__new__(POP3)
try:
    obj.rpop(12345)  # user: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/poplib/POP3__set_debuglevel__level_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_poplib_POP3__set_debuglevel__level_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "poplib"
# dimension = "type"
# case = "POP3__set_debuglevel__level_as_int_wrong"
# subject = "poplib.POP3.set_debuglevel(level: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/poplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: poplib.POP3.set_debuglevel(level: int); call it with the wrong type.

typeshed contract: level is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from poplib import POP3
obj = object.__new__(POP3)
try:
    obj.set_debuglevel("not_an_int")  # level: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/poplib/POP3__stls__context_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_poplib_POP3__stls__context_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "poplib"
# dimension = "type"
# case = "POP3__stls__context_as_typed_wrong"
# subject = "poplib.POP3.stls(context: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/poplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: poplib.POP3.stls(context: typed); call it with the wrong type.

typeshed contract: context is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from poplib import POP3
obj = object.__new__(POP3)
try:
    obj.stls(_W())  # context: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/poplib/POP3__top__howmuch_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_poplib_POP3__top__howmuch_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "poplib"
# dimension = "type"
# case = "POP3__top__howmuch_as_int_wrong"
# subject = "poplib.POP3.top(howmuch: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/poplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: poplib.POP3.top(howmuch: int); call it with the wrong type.

typeshed contract: howmuch is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from poplib import POP3
obj = object.__new__(POP3)
try:
    obj.top(None, "not_an_int")  # howmuch: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/poplib/POP3__user__user_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_poplib_POP3__user__user_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "poplib"
# dimension = "type"
# case = "POP3__user__user_as_str_wrong"
# subject = "poplib.POP3.user(user: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/poplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: poplib.POP3.user(user: str); call it with the wrong type.

typeshed contract: user is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from poplib import POP3
obj = object.__new__(POP3)
try:
    obj.user(12345)  # user: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
