use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/asyncio_windows_utils/PipeHandle____exit____t_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_windows_utils_PipeHandle____exit____t_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_windows_utils"
# dimension = "type"
# case = "PipeHandle____exit____t_as_typed_wrong"
# subject = "asyncio.windows_utils.PipeHandle.__exit__(t: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/windows_utils.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.windows_utils.PipeHandle.__exit__(t: typed); call it with the wrong type.

typeshed contract: t is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.windows_utils import PipeHandle
obj = object.__new__(PipeHandle)
try:
    obj.__exit__(_W(), None, None)  # t: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_windows_utils/PipeHandle__init__handle_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_windows_utils_PipeHandle__init__handle_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_windows_utils"
# dimension = "type"
# case = "PipeHandle__init__handle_as_int_wrong"
# subject = "asyncio.windows_utils.PipeHandle.__init__(handle: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/windows_utils.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.windows_utils.PipeHandle.__init__(handle: int); call it with the wrong type.

typeshed contract: handle is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from asyncio.windows_utils import PipeHandle
try:
    PipeHandle("not_an_int")  # handle: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_windows_utils/Popen____new____args_as__CMD_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_windows_utils_Popen____new____args_as__CMD_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_windows_utils"
# dimension = "type"
# case = "Popen____new____args_as__CMD_wrong"
# subject = "asyncio.windows_utils.Popen.__new__(args: _CMD)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/windows_utils.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.windows_utils.Popen.__new__(args: _CMD); call it with the wrong type.

typeshed contract: args is _CMD. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.windows_utils import Popen
obj = object.__new__(Popen)
try:
    obj.__new__(_W())  # args: _CMD <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_windows_utils/Popen__init__args_as__CMD_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_windows_utils_Popen__init__args_as__CMD_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_windows_utils"
# dimension = "type"
# case = "Popen__init__args_as__CMD_wrong"
# subject = "asyncio.windows_utils.Popen.__init__(args: _CMD)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/windows_utils.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.windows_utils.Popen.__init__(args: _CMD); call it with the wrong type.

typeshed contract: args is _CMD. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.windows_utils import Popen
try:
    Popen(_W())  # args: _CMD <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
