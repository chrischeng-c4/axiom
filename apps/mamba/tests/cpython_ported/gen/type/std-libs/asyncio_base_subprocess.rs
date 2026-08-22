use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/asyncio_base_subprocess/BaseSubprocessTransport__get_pipe_transport__fd_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_base_subprocess_BaseSubprocessTransport__get_pipe_transport__fd_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_base_subprocess"
# dimension = "type"
# case = "BaseSubprocessTransport__get_pipe_transport__fd_as_int_wrong"
# subject = "asyncio.base_subprocess.BaseSubprocessTransport.get_pipe_transport(fd: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/base_subprocess.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.base_subprocess.BaseSubprocessTransport.get_pipe_transport(fd: int); call it with the wrong type.

typeshed contract: fd is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from asyncio.base_subprocess import BaseSubprocessTransport
obj = object.__new__(BaseSubprocessTransport)
try:
    obj.get_pipe_transport("not_an_int")  # fd: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_base_subprocess/BaseSubprocessTransport__init__loop_as_AbstractEventLoop_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_base_subprocess_BaseSubprocessTransport__init__loop_as_AbstractEventLoop_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_base_subprocess"
# dimension = "type"
# case = "BaseSubprocessTransport__init__loop_as_AbstractEventLoop_wrong"
# subject = "asyncio.base_subprocess.BaseSubprocessTransport.__init__(loop: AbstractEventLoop)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/base_subprocess.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.base_subprocess.BaseSubprocessTransport.__init__(loop: AbstractEventLoop); call it with the wrong type.

typeshed contract: loop is AbstractEventLoop. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.base_subprocess import BaseSubprocessTransport
try:
    BaseSubprocessTransport(_W(), None, None, True, None, None, None, 0)  # loop: AbstractEventLoop <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_base_subprocess/BaseSubprocessTransport__send_signal__signal_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_base_subprocess_BaseSubprocessTransport__send_signal__signal_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_base_subprocess"
# dimension = "type"
# case = "BaseSubprocessTransport__send_signal__signal_as_int_wrong"
# subject = "asyncio.base_subprocess.BaseSubprocessTransport.send_signal(signal: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/base_subprocess.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.base_subprocess.BaseSubprocessTransport.send_signal(signal: int); call it with the wrong type.

typeshed contract: signal is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from asyncio.base_subprocess import BaseSubprocessTransport
obj = object.__new__(BaseSubprocessTransport)
try:
    obj.send_signal("not_an_int")  # signal: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_base_subprocess/WriteSubprocessPipeProto__init__proc_as_BaseSubprocessTransport_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_base_subprocess_WriteSubprocessPipeProto__init__proc_as_BaseSubprocessTransport_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_base_subprocess"
# dimension = "type"
# case = "WriteSubprocessPipeProto__init__proc_as_BaseSubprocessTransport_wrong"
# subject = "asyncio.base_subprocess.WriteSubprocessPipeProto.__init__(proc: BaseSubprocessTransport)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/base_subprocess.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.base_subprocess.WriteSubprocessPipeProto.__init__(proc: BaseSubprocessTransport); call it with the wrong type.

typeshed contract: proc is BaseSubprocessTransport. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.base_subprocess import WriteSubprocessPipeProto
try:
    WriteSubprocessPipeProto(_W(), 0)  # proc: BaseSubprocessTransport <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
