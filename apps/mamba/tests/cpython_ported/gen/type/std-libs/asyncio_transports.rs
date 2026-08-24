use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/asyncio_transports/BaseTransport__get_extra_info__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_transports_BaseTransport__get_extra_info__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_transports"
# dimension = "type"
# case = "BaseTransport__get_extra_info__name_as_str_wrong"
# subject = "asyncio.transports.BaseTransport.get_extra_info(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/transports.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.transports.BaseTransport.get_extra_info(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from asyncio.transports import BaseTransport
obj = object.__new__(BaseTransport)
try:
    obj.get_extra_info(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_transports/BaseTransport__init__extra_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_transports_BaseTransport__init__extra_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_transports"
# dimension = "type"
# case = "BaseTransport__init__extra_as_typed_wrong"
# subject = "asyncio.transports.BaseTransport.__init__(extra: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/transports.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.transports.BaseTransport.__init__(extra: typed); call it with the wrong type.

typeshed contract: extra is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.transports import BaseTransport
try:
    BaseTransport(_W())  # extra: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_transports/BaseTransport__set_protocol__protocol_as_BaseProtocol_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_transports_BaseTransport__set_protocol__protocol_as_BaseProtocol_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_transports"
# dimension = "type"
# case = "BaseTransport__set_protocol__protocol_as_BaseProtocol_wrong"
# subject = "asyncio.transports.BaseTransport.set_protocol(protocol: BaseProtocol)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/transports.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.transports.BaseTransport.set_protocol(protocol: BaseProtocol); call it with the wrong type.

typeshed contract: protocol is BaseProtocol. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.transports import BaseTransport
obj = object.__new__(BaseTransport)
try:
    obj.set_protocol(_W())  # protocol: BaseProtocol <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_transports/DatagramTransport__sendto__data_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_transports_DatagramTransport__sendto__data_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_transports"
# dimension = "type"
# case = "DatagramTransport__sendto__data_as_typed_wrong"
# subject = "asyncio.transports.DatagramTransport.sendto(data: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/transports.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.transports.DatagramTransport.sendto(data: typed); call it with the wrong type.

typeshed contract: data is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.transports import DatagramTransport
obj = object.__new__(DatagramTransport)
try:
    obj.sendto(_W())  # data: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_transports/SubprocessTransport__get_pipe_transport__fd_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_transports_SubprocessTransport__get_pipe_transport__fd_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_transports"
# dimension = "type"
# case = "SubprocessTransport__get_pipe_transport__fd_as_int_wrong"
# subject = "asyncio.transports.SubprocessTransport.get_pipe_transport(fd: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/transports.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.transports.SubprocessTransport.get_pipe_transport(fd: int); call it with the wrong type.

typeshed contract: fd is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from asyncio.transports import SubprocessTransport
obj = object.__new__(SubprocessTransport)
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

/// Ported from `tests/cpython/type/std-libs/asyncio_transports/SubprocessTransport__send_signal__signal_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_transports_SubprocessTransport__send_signal__signal_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_transports"
# dimension = "type"
# case = "SubprocessTransport__send_signal__signal_as_int_wrong"
# subject = "asyncio.transports.SubprocessTransport.send_signal(signal: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/transports.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.transports.SubprocessTransport.send_signal(signal: int); call it with the wrong type.

typeshed contract: signal is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from asyncio.transports import SubprocessTransport
obj = object.__new__(SubprocessTransport)
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

/// Ported from `tests/cpython/type/std-libs/asyncio_transports/WriteTransport__set_write_buffer_limits__high_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_transports_WriteTransport__set_write_buffer_limits__high_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_transports"
# dimension = "type"
# case = "WriteTransport__set_write_buffer_limits__high_as_typed_wrong"
# subject = "asyncio.transports.WriteTransport.set_write_buffer_limits(high: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/transports.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.transports.WriteTransport.set_write_buffer_limits(high: typed); call it with the wrong type.

typeshed contract: high is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.transports import WriteTransport
obj = object.__new__(WriteTransport)
try:
    obj.set_write_buffer_limits(_W())  # high: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_transports/WriteTransport__write__data_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_transports_WriteTransport__write__data_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_transports"
# dimension = "type"
# case = "WriteTransport__write__data_as_typed_wrong"
# subject = "asyncio.transports.WriteTransport.write(data: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/transports.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.transports.WriteTransport.write(data: typed); call it with the wrong type.

typeshed contract: data is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.transports import WriteTransport
obj = object.__new__(WriteTransport)
try:
    obj.write(_W())  # data: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_transports/WriteTransport__writelines__list_of_data_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_transports_WriteTransport__writelines__list_of_data_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_transports"
# dimension = "type"
# case = "WriteTransport__writelines__list_of_data_as_Iterable_wrong"
# subject = "asyncio.transports.WriteTransport.writelines(list_of_data: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/transports.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.transports.WriteTransport.writelines(list_of_data: Iterable); call it with the wrong type.

typeshed contract: list_of_data is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.transports import WriteTransport
obj = object.__new__(WriteTransport)
try:
    obj.writelines(_W())  # list_of_data: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
