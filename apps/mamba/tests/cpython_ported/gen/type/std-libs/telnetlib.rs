use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/telnetlib/Telnet__init__host_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_telnetlib_Telnet__init__host_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "telnetlib"
# dimension = "type"
# case = "Telnet__init__host_as_typed_wrong"
# subject = "telnetlib.Telnet.__init__(host: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/telnetlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: telnetlib.Telnet.__init__(host: typed); call it with the wrong type.

typeshed contract: host is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from telnetlib import Telnet
try:
    Telnet(_W())  # host: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/telnetlib/Telnet__open__host_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_telnetlib_Telnet__open__host_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "telnetlib"
# dimension = "type"
# case = "Telnet__open__host_as_str_wrong"
# subject = "telnetlib.Telnet.open(host: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/telnetlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: telnetlib.Telnet.open(host: str); call it with the wrong type.

typeshed contract: host is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from telnetlib import Telnet
obj = object.__new__(Telnet)
try:
    obj.open(12345)  # host: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/telnetlib/Telnet__set_debuglevel__debuglevel_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_telnetlib_Telnet__set_debuglevel__debuglevel_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "telnetlib"
# dimension = "type"
# case = "Telnet__set_debuglevel__debuglevel_as_int_wrong"
# subject = "telnetlib.Telnet.set_debuglevel(debuglevel: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/telnetlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: telnetlib.Telnet.set_debuglevel(debuglevel: int); call it with the wrong type.

typeshed contract: debuglevel is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from telnetlib import Telnet
obj = object.__new__(Telnet)
try:
    obj.set_debuglevel("not_an_int")  # debuglevel: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/telnetlib/Telnet__write__buffer_as_bytes_wrong.py`.
#[test]
fn test_gen_type_std_libs_telnetlib_Telnet__write__buffer_as_bytes_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "telnetlib"
# dimension = "type"
# case = "Telnet__write__buffer_as_bytes_wrong"
# subject = "telnetlib.Telnet.write(buffer: bytes)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/telnetlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: telnetlib.Telnet.write(buffer: bytes); call it with the wrong type.

typeshed contract: buffer is bytes. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from telnetlib import Telnet
obj = object.__new__(Telnet)
try:
    obj.write(12345)  # buffer: bytes <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
