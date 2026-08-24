use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/msvcrt/SetErrorMode__mode_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_msvcrt_SetErrorMode__mode_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "msvcrt"
# dimension = "type"
# case = "SetErrorMode__mode_as_int_wrong"
# subject = "msvcrt.SetErrorMode(mode: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/msvcrt.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: msvcrt.SetErrorMode(mode: int); call it with the wrong type.

typeshed contract: mode is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from msvcrt import SetErrorMode
try:
    SetErrorMode("not_an_int")  # mode: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/msvcrt/get_osfhandle__fd_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_msvcrt_get_osfhandle__fd_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "msvcrt"
# dimension = "type"
# case = "get_osfhandle__fd_as_int_wrong"
# subject = "msvcrt.get_osfhandle(fd: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/msvcrt.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: msvcrt.get_osfhandle(fd: int); call it with the wrong type.

typeshed contract: fd is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from msvcrt import get_osfhandle
try:
    get_osfhandle("not_an_int")  # fd: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/msvcrt/locking__fd_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_msvcrt_locking__fd_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "msvcrt"
# dimension = "type"
# case = "locking__fd_as_int_wrong"
# subject = "msvcrt.locking(fd: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/msvcrt.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: msvcrt.locking(fd: int); call it with the wrong type.

typeshed contract: fd is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from msvcrt import locking
try:
    locking("not_an_int", 0, 0)  # fd: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/msvcrt/open_osfhandle__handle_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_msvcrt_open_osfhandle__handle_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "msvcrt"
# dimension = "type"
# case = "open_osfhandle__handle_as_int_wrong"
# subject = "msvcrt.open_osfhandle(handle: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/msvcrt.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: msvcrt.open_osfhandle(handle: int); call it with the wrong type.

typeshed contract: handle is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from msvcrt import open_osfhandle
try:
    open_osfhandle("not_an_int", 0)  # handle: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/msvcrt/putch__char_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_msvcrt_putch__char_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "msvcrt"
# dimension = "type"
# case = "putch__char_as_typed_wrong"
# subject = "msvcrt.putch(char: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/msvcrt.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: msvcrt.putch(char: typed); call it with the wrong type.

typeshed contract: char is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from msvcrt import putch
try:
    putch(_W())  # char: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/msvcrt/putwch__unicode_char_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_msvcrt_putwch__unicode_char_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "msvcrt"
# dimension = "type"
# case = "putwch__unicode_char_as_str_wrong"
# subject = "msvcrt.putwch(unicode_char: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/msvcrt.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: msvcrt.putwch(unicode_char: str); call it with the wrong type.

typeshed contract: unicode_char is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from msvcrt import putwch
try:
    putwch(12345)  # unicode_char: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/msvcrt/setmode__fd_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_msvcrt_setmode__fd_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "msvcrt"
# dimension = "type"
# case = "setmode__fd_as_int_wrong"
# subject = "msvcrt.setmode(fd: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/msvcrt.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: msvcrt.setmode(fd: int); call it with the wrong type.

typeshed contract: fd is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from msvcrt import setmode
try:
    setmode("not_an_int", 0)  # fd: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/msvcrt/ungetch__char_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_msvcrt_ungetch__char_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "msvcrt"
# dimension = "type"
# case = "ungetch__char_as_typed_wrong"
# subject = "msvcrt.ungetch(char: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/msvcrt.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: msvcrt.ungetch(char: typed); call it with the wrong type.

typeshed contract: char is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from msvcrt import ungetch
try:
    ungetch(_W())  # char: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/msvcrt/ungetwch__unicode_char_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_msvcrt_ungetwch__unicode_char_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "msvcrt"
# dimension = "type"
# case = "ungetwch__unicode_char_as_str_wrong"
# subject = "msvcrt.ungetwch(unicode_char: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/msvcrt.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: msvcrt.ungetwch(unicode_char: str); call it with the wrong type.

typeshed contract: unicode_char is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from msvcrt import ungetwch
try:
    ungetwch(12345)  # unicode_char: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
