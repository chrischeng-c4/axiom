use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/subprocess/CalledProcessError__init__returncode_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_subprocess_CalledProcessError__init__returncode_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "type"
# case = "CalledProcessError__init__returncode_as_int_wrong"
# subject = "subprocess.CalledProcessError.__init__(returncode: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/subprocess.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: subprocess.CalledProcessError.__init__(returncode: int); call it with the wrong type.

typeshed contract: returncode is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from subprocess import CalledProcessError
try:
    CalledProcessError("not_an_int", None)  # returncode: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/subprocess/CompletedProcess__init__args_as__CMD_wrong.py`.
#[test]
fn test_gen_type_std_libs_subprocess_CompletedProcess__init__args_as__CMD_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "type"
# case = "CompletedProcess__init__args_as__CMD_wrong"
# subject = "subprocess.CompletedProcess.__init__(args: _CMD)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/subprocess.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: subprocess.CompletedProcess.__init__(args: _CMD); call it with the wrong type.

typeshed contract: args is _CMD. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from subprocess import CompletedProcess
try:
    CompletedProcess(_W(), 0)  # args: _CMD <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/subprocess/Popen__send_signal__sig_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_subprocess_Popen__send_signal__sig_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "type"
# case = "Popen__send_signal__sig_as_int_wrong"
# subject = "subprocess.Popen.send_signal(sig: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/subprocess.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: subprocess.Popen.send_signal(sig: int); call it with the wrong type.

typeshed contract: sig is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from subprocess import Popen
obj = object.__new__(Popen)
try:
    obj.send_signal("not_an_int")  # sig: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/subprocess/Popen__wait__timeout_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_subprocess_Popen__wait__timeout_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "type"
# case = "Popen__wait__timeout_as_typed_wrong"
# subject = "subprocess.Popen.wait(timeout: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/subprocess.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: subprocess.Popen.wait(timeout: typed); call it with the wrong type.

typeshed contract: timeout is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from subprocess import Popen
obj = object.__new__(Popen)
try:
    obj.wait(_W())  # timeout: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/subprocess/TimeoutExpired__init__cmd_as__CMD_wrong.py`.
#[test]
fn test_gen_type_std_libs_subprocess_TimeoutExpired__init__cmd_as__CMD_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "type"
# case = "TimeoutExpired__init__cmd_as__CMD_wrong"
# subject = "subprocess.TimeoutExpired.__init__(cmd: _CMD)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/subprocess.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: subprocess.TimeoutExpired.__init__(cmd: _CMD); call it with the wrong type.

typeshed contract: cmd is _CMD. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from subprocess import TimeoutExpired
try:
    TimeoutExpired(_W(), 0.0)  # cmd: _CMD <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/subprocess/call__args_as__CMD_wrong.py`.
#[test]
fn test_gen_type_std_libs_subprocess_call__args_as__CMD_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "type"
# case = "call__args_as__CMD_wrong"
# subject = "subprocess.call(args: _CMD)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/subprocess.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: subprocess.call(args: _CMD); call it with the wrong type.

typeshed contract: args is _CMD. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from subprocess import call
try:
    call(_W())  # args: _CMD <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/subprocess/check_call__args_as__CMD_wrong.py`.
#[test]
fn test_gen_type_std_libs_subprocess_check_call__args_as__CMD_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "type"
# case = "check_call__args_as__CMD_wrong"
# subject = "subprocess.check_call(args: _CMD)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/subprocess.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: subprocess.check_call(args: _CMD); call it with the wrong type.

typeshed contract: args is _CMD. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from subprocess import check_call
try:
    check_call(_W())  # args: _CMD <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/subprocess/getoutput__cmd_as__CMD_wrong.py`.
#[test]
fn test_gen_type_std_libs_subprocess_getoutput__cmd_as__CMD_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "type"
# case = "getoutput__cmd_as__CMD_wrong"
# subject = "subprocess.getoutput(cmd: _CMD)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/subprocess.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: subprocess.getoutput(cmd: _CMD); call it with the wrong type.

typeshed contract: cmd is _CMD. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from subprocess import getoutput
try:
    getoutput(_W())  # cmd: _CMD <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/subprocess/getstatusoutput__cmd_as__CMD_wrong.py`.
#[test]
fn test_gen_type_std_libs_subprocess_getstatusoutput__cmd_as__CMD_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "type"
# case = "getstatusoutput__cmd_as__CMD_wrong"
# subject = "subprocess.getstatusoutput(cmd: _CMD)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/subprocess.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: subprocess.getstatusoutput(cmd: _CMD); call it with the wrong type.

typeshed contract: cmd is _CMD. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from subprocess import getstatusoutput
try:
    getstatusoutput(_W())  # cmd: _CMD <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/subprocess/list2cmdline__seq_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_subprocess_list2cmdline__seq_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "type"
# case = "list2cmdline__seq_as_Iterable_wrong"
# subject = "subprocess.list2cmdline(seq: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/subprocess.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: subprocess.list2cmdline(seq: Iterable); call it with the wrong type.

typeshed contract: seq is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from subprocess import list2cmdline
try:
    list2cmdline(_W())  # seq: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
