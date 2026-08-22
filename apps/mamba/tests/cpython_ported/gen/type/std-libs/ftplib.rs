use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/ftplib/FTP_TLS__init__host_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_ftplib_FTP_TLS__init__host_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ftplib"
# dimension = "type"
# case = "FTP_TLS__init__host_as_str_wrong"
# subject = "ftplib.FTP_TLS.__init__(host: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ftplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ftplib.FTP_TLS.__init__(host: str); call it with the wrong type.

typeshed contract: host is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ftplib import FTP_TLS
try:
    FTP_TLS(12345)  # host: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ftplib/FTP_TLS__login__user_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_ftplib_FTP_TLS__login__user_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ftplib"
# dimension = "type"
# case = "FTP_TLS__login__user_as_str_wrong"
# subject = "ftplib.FTP_TLS.login(user: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ftplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ftplib.FTP_TLS.login(user: str); call it with the wrong type.

typeshed contract: user is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ftplib import FTP_TLS
obj = object.__new__(FTP_TLS)
try:
    obj.login(12345)  # user: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ftplib/FTP____exit____exc_type_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_ftplib_FTP____exit____exc_type_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ftplib"
# dimension = "type"
# case = "FTP____exit____exc_type_as_typed_wrong"
# subject = "ftplib.FTP.__exit__(exc_type: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ftplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ftplib.FTP.__exit__(exc_type: typed); call it with the wrong type.

typeshed contract: exc_type is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ftplib import FTP
obj = object.__new__(FTP)
try:
    obj.__exit__(_W(), None, None)  # exc_type: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ftplib/FTP__acct__password_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_ftplib_FTP__acct__password_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ftplib"
# dimension = "type"
# case = "FTP__acct__password_as_str_wrong"
# subject = "ftplib.FTP.acct(password: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ftplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ftplib.FTP.acct(password: str); call it with the wrong type.

typeshed contract: password is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ftplib import FTP
obj = object.__new__(FTP)
try:
    obj.acct(12345)  # password: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ftplib/FTP__connect__host_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_ftplib_FTP__connect__host_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ftplib"
# dimension = "type"
# case = "FTP__connect__host_as_str_wrong"
# subject = "ftplib.FTP.connect(host: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ftplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ftplib.FTP.connect(host: str); call it with the wrong type.

typeshed contract: host is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ftplib import FTP
obj = object.__new__(FTP)
try:
    obj.connect(12345)  # host: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ftplib/FTP__cwd__dirname_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_ftplib_FTP__cwd__dirname_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ftplib"
# dimension = "type"
# case = "FTP__cwd__dirname_as_str_wrong"
# subject = "ftplib.FTP.cwd(dirname: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ftplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ftplib.FTP.cwd(dirname: str); call it with the wrong type.

typeshed contract: dirname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ftplib import FTP
obj = object.__new__(FTP)
try:
    obj.cwd(12345)  # dirname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ftplib/FTP__debug__level_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_ftplib_FTP__debug__level_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ftplib"
# dimension = "type"
# case = "FTP__debug__level_as_int_wrong"
# subject = "ftplib.FTP.debug(level: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ftplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ftplib.FTP.debug(level: int); call it with the wrong type.

typeshed contract: level is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ftplib import FTP
obj = object.__new__(FTP)
try:
    obj.debug("not_an_int")  # level: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ftplib/FTP__delete__filename_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_ftplib_FTP__delete__filename_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ftplib"
# dimension = "type"
# case = "FTP__delete__filename_as_str_wrong"
# subject = "ftplib.FTP.delete(filename: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ftplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ftplib.FTP.delete(filename: str); call it with the wrong type.

typeshed contract: filename is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ftplib import FTP
obj = object.__new__(FTP)
try:
    obj.delete(12345)  # filename: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ftplib/FTP__init__host_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_ftplib_FTP__init__host_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ftplib"
# dimension = "type"
# case = "FTP__init__host_as_str_wrong"
# subject = "ftplib.FTP.__init__(host: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ftplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ftplib.FTP.__init__(host: str); call it with the wrong type.

typeshed contract: host is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ftplib import FTP
try:
    FTP(12345)  # host: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ftplib/FTP__login__user_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_ftplib_FTP__login__user_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ftplib"
# dimension = "type"
# case = "FTP__login__user_as_str_wrong"
# subject = "ftplib.FTP.login(user: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ftplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ftplib.FTP.login(user: str); call it with the wrong type.

typeshed contract: user is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ftplib import FTP
obj = object.__new__(FTP)
try:
    obj.login(12345)  # user: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ftplib/FTP__mkd__dirname_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_ftplib_FTP__mkd__dirname_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ftplib"
# dimension = "type"
# case = "FTP__mkd__dirname_as_str_wrong"
# subject = "ftplib.FTP.mkd(dirname: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ftplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ftplib.FTP.mkd(dirname: str); call it with the wrong type.

typeshed contract: dirname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ftplib import FTP
obj = object.__new__(FTP)
try:
    obj.mkd(12345)  # dirname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ftplib/FTP__mlsd__path_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_ftplib_FTP__mlsd__path_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ftplib"
# dimension = "type"
# case = "FTP__mlsd__path_as_str_wrong"
# subject = "ftplib.FTP.mlsd(path: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ftplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ftplib.FTP.mlsd(path: str); call it with the wrong type.

typeshed contract: path is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ftplib import FTP
obj = object.__new__(FTP)
try:
    obj.mlsd(12345)  # path: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ftplib/FTP__ntransfercmd__cmd_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_ftplib_FTP__ntransfercmd__cmd_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ftplib"
# dimension = "type"
# case = "FTP__ntransfercmd__cmd_as_str_wrong"
# subject = "ftplib.FTP.ntransfercmd(cmd: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ftplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ftplib.FTP.ntransfercmd(cmd: str); call it with the wrong type.

typeshed contract: cmd is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ftplib import FTP
obj = object.__new__(FTP)
try:
    obj.ntransfercmd(12345)  # cmd: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ftplib/FTP__putcmd__line_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_ftplib_FTP__putcmd__line_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ftplib"
# dimension = "type"
# case = "FTP__putcmd__line_as_str_wrong"
# subject = "ftplib.FTP.putcmd(line: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ftplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ftplib.FTP.putcmd(line: str); call it with the wrong type.

typeshed contract: line is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ftplib import FTP
obj = object.__new__(FTP)
try:
    obj.putcmd(12345)  # line: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ftplib/FTP__putline__line_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_ftplib_FTP__putline__line_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ftplib"
# dimension = "type"
# case = "FTP__putline__line_as_str_wrong"
# subject = "ftplib.FTP.putline(line: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ftplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ftplib.FTP.putline(line: str); call it with the wrong type.

typeshed contract: line is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ftplib import FTP
obj = object.__new__(FTP)
try:
    obj.putline(12345)  # line: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ftplib/FTP__rename__fromname_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_ftplib_FTP__rename__fromname_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ftplib"
# dimension = "type"
# case = "FTP__rename__fromname_as_str_wrong"
# subject = "ftplib.FTP.rename(fromname: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ftplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ftplib.FTP.rename(fromname: str); call it with the wrong type.

typeshed contract: fromname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ftplib import FTP
obj = object.__new__(FTP)
try:
    obj.rename(12345, "")  # fromname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ftplib/FTP__retrbinary__cmd_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_ftplib_FTP__retrbinary__cmd_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ftplib"
# dimension = "type"
# case = "FTP__retrbinary__cmd_as_str_wrong"
# subject = "ftplib.FTP.retrbinary(cmd: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ftplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ftplib.FTP.retrbinary(cmd: str); call it with the wrong type.

typeshed contract: cmd is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ftplib import FTP
obj = object.__new__(FTP)
try:
    obj.retrbinary(12345, None)  # cmd: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ftplib/FTP__retrlines__cmd_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_ftplib_FTP__retrlines__cmd_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ftplib"
# dimension = "type"
# case = "FTP__retrlines__cmd_as_str_wrong"
# subject = "ftplib.FTP.retrlines(cmd: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ftplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ftplib.FTP.retrlines(cmd: str); call it with the wrong type.

typeshed contract: cmd is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ftplib import FTP
obj = object.__new__(FTP)
try:
    obj.retrlines(12345)  # cmd: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ftplib/FTP__rmd__dirname_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_ftplib_FTP__rmd__dirname_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ftplib"
# dimension = "type"
# case = "FTP__rmd__dirname_as_str_wrong"
# subject = "ftplib.FTP.rmd(dirname: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ftplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ftplib.FTP.rmd(dirname: str); call it with the wrong type.

typeshed contract: dirname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ftplib import FTP
obj = object.__new__(FTP)
try:
    obj.rmd(12345)  # dirname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ftplib/FTP__sanitize__s_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_ftplib_FTP__sanitize__s_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ftplib"
# dimension = "type"
# case = "FTP__sanitize__s_as_str_wrong"
# subject = "ftplib.FTP.sanitize(s: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ftplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ftplib.FTP.sanitize(s: str); call it with the wrong type.

typeshed contract: s is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ftplib import FTP
obj = object.__new__(FTP)
try:
    obj.sanitize(12345)  # s: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ftplib/FTP__sendcmd__cmd_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_ftplib_FTP__sendcmd__cmd_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ftplib"
# dimension = "type"
# case = "FTP__sendcmd__cmd_as_str_wrong"
# subject = "ftplib.FTP.sendcmd(cmd: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ftplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ftplib.FTP.sendcmd(cmd: str); call it with the wrong type.

typeshed contract: cmd is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ftplib import FTP
obj = object.__new__(FTP)
try:
    obj.sendcmd(12345)  # cmd: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ftplib/FTP__sendeprt__host_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_ftplib_FTP__sendeprt__host_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ftplib"
# dimension = "type"
# case = "FTP__sendeprt__host_as_str_wrong"
# subject = "ftplib.FTP.sendeprt(host: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ftplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ftplib.FTP.sendeprt(host: str); call it with the wrong type.

typeshed contract: host is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ftplib import FTP
obj = object.__new__(FTP)
try:
    obj.sendeprt(12345, 0)  # host: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ftplib/FTP__sendport__host_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_ftplib_FTP__sendport__host_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ftplib"
# dimension = "type"
# case = "FTP__sendport__host_as_str_wrong"
# subject = "ftplib.FTP.sendport(host: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ftplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ftplib.FTP.sendport(host: str); call it with the wrong type.

typeshed contract: host is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ftplib import FTP
obj = object.__new__(FTP)
try:
    obj.sendport(12345, 0)  # host: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ftplib/FTP__set_debuglevel__level_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_ftplib_FTP__set_debuglevel__level_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ftplib"
# dimension = "type"
# case = "FTP__set_debuglevel__level_as_int_wrong"
# subject = "ftplib.FTP.set_debuglevel(level: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ftplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ftplib.FTP.set_debuglevel(level: int); call it with the wrong type.

typeshed contract: level is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ftplib import FTP
obj = object.__new__(FTP)
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

/// Ported from `tests/cpython/type/std-libs/ftplib/FTP__set_pasv__val_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_ftplib_FTP__set_pasv__val_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ftplib"
# dimension = "type"
# case = "FTP__set_pasv__val_as_typed_wrong"
# subject = "ftplib.FTP.set_pasv(val: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ftplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ftplib.FTP.set_pasv(val: typed); call it with the wrong type.

typeshed contract: val is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ftplib import FTP
obj = object.__new__(FTP)
try:
    obj.set_pasv(_W())  # val: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ftplib/FTP__size__filename_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_ftplib_FTP__size__filename_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ftplib"
# dimension = "type"
# case = "FTP__size__filename_as_str_wrong"
# subject = "ftplib.FTP.size(filename: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ftplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ftplib.FTP.size(filename: str); call it with the wrong type.

typeshed contract: filename is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ftplib import FTP
obj = object.__new__(FTP)
try:
    obj.size(12345)  # filename: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ftplib/FTP__storbinary__cmd_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_ftplib_FTP__storbinary__cmd_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ftplib"
# dimension = "type"
# case = "FTP__storbinary__cmd_as_str_wrong"
# subject = "ftplib.FTP.storbinary(cmd: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ftplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ftplib.FTP.storbinary(cmd: str); call it with the wrong type.

typeshed contract: cmd is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ftplib import FTP
obj = object.__new__(FTP)
try:
    obj.storbinary(12345, None)  # cmd: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ftplib/FTP__storlines__cmd_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_ftplib_FTP__storlines__cmd_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ftplib"
# dimension = "type"
# case = "FTP__storlines__cmd_as_str_wrong"
# subject = "ftplib.FTP.storlines(cmd: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ftplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ftplib.FTP.storlines(cmd: str); call it with the wrong type.

typeshed contract: cmd is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ftplib import FTP
obj = object.__new__(FTP)
try:
    obj.storlines(12345, None)  # cmd: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ftplib/FTP__transfercmd__cmd_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_ftplib_FTP__transfercmd__cmd_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ftplib"
# dimension = "type"
# case = "FTP__transfercmd__cmd_as_str_wrong"
# subject = "ftplib.FTP.transfercmd(cmd: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ftplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ftplib.FTP.transfercmd(cmd: str); call it with the wrong type.

typeshed contract: cmd is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ftplib import FTP
obj = object.__new__(FTP)
try:
    obj.transfercmd(12345)  # cmd: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ftplib/FTP__voidcmd__cmd_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_ftplib_FTP__voidcmd__cmd_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ftplib"
# dimension = "type"
# case = "FTP__voidcmd__cmd_as_str_wrong"
# subject = "ftplib.FTP.voidcmd(cmd: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ftplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ftplib.FTP.voidcmd(cmd: str); call it with the wrong type.

typeshed contract: cmd is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ftplib import FTP
obj = object.__new__(FTP)
try:
    obj.voidcmd(12345)  # cmd: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ftplib/ftpcp__source_as_FTP_wrong.py`.
#[test]
fn test_gen_type_std_libs_ftplib_ftpcp__source_as_FTP_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ftplib"
# dimension = "type"
# case = "ftpcp__source_as_FTP_wrong"
# subject = "ftplib.ftpcp(source: FTP)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ftplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ftplib.ftpcp(source: FTP); call it with the wrong type.

typeshed contract: source is FTP. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ftplib import ftpcp
try:
    ftpcp(_W(), "", None)  # source: FTP <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ftplib/parse150__resp_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_ftplib_parse150__resp_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ftplib"
# dimension = "type"
# case = "parse150__resp_as_str_wrong"
# subject = "ftplib.parse150(resp: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ftplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ftplib.parse150(resp: str); call it with the wrong type.

typeshed contract: resp is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ftplib import parse150
try:
    parse150(12345)  # resp: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ftplib/parse227__resp_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_ftplib_parse227__resp_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ftplib"
# dimension = "type"
# case = "parse227__resp_as_str_wrong"
# subject = "ftplib.parse227(resp: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ftplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ftplib.parse227(resp: str); call it with the wrong type.

typeshed contract: resp is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ftplib import parse227
try:
    parse227(12345)  # resp: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ftplib/parse229__resp_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_ftplib_parse229__resp_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ftplib"
# dimension = "type"
# case = "parse229__resp_as_str_wrong"
# subject = "ftplib.parse229(resp: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ftplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ftplib.parse229(resp: str); call it with the wrong type.

typeshed contract: resp is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ftplib import parse229
try:
    parse229(12345, None)  # resp: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ftplib/parse257__resp_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_ftplib_parse257__resp_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ftplib"
# dimension = "type"
# case = "parse257__resp_as_str_wrong"
# subject = "ftplib.parse257(resp: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ftplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ftplib.parse257(resp: str); call it with the wrong type.

typeshed contract: resp is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ftplib import parse257
try:
    parse257(12345)  # resp: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
