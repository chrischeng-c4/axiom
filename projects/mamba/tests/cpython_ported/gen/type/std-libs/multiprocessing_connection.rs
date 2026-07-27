use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/multiprocessing_connection/Client__address_as__Address_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_connection_Client__address_as__Address_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_connection"
# dimension = "type"
# case = "Client__address_as__Address_wrong"
# subject = "multiprocessing.connection.Client(address: _Address)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/connection.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.connection.Client(address: _Address); call it with the wrong type.

typeshed contract: address is _Address. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.connection import Client
try:
    Client(_W())  # address: _Address <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/multiprocessing_connection/Listener____exit____exc_type_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_connection_Listener____exit____exc_type_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_connection"
# dimension = "type"
# case = "Listener____exit____exc_type_as_typed_wrong"
# subject = "multiprocessing.connection.Listener.__exit__(exc_type: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/connection.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.connection.Listener.__exit__(exc_type: typed); call it with the wrong type.

typeshed contract: exc_type is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.connection import Listener
obj = object.__new__(Listener)
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

/// Ported from `tests/cpython/type/std-libs/multiprocessing_connection/Listener__init__address_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_connection_Listener__init__address_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_connection"
# dimension = "type"
# case = "Listener__init__address_as_typed_wrong"
# subject = "multiprocessing.connection.Listener.__init__(address: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/connection.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.connection.Listener.__init__(address: typed); call it with the wrong type.

typeshed contract: address is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.connection import Listener
try:
    Listener(_W())  # address: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/multiprocessing_connection/Pipe__duplex_as_bool_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_connection_Pipe__duplex_as_bool_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_connection"
# dimension = "type"
# case = "Pipe__duplex_as_bool_wrong"
# subject = "multiprocessing.connection.Pipe(duplex: bool)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/connection.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.connection.Pipe(duplex: bool); call it with the wrong type.

typeshed contract: duplex is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from multiprocessing.connection import Pipe
try:
    Pipe("not_a_bool")  # duplex: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/multiprocessing_connection/answer_challenge__connection_as_Connection_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_connection_answer_challenge__connection_as_Connection_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_connection"
# dimension = "type"
# case = "answer_challenge__connection_as_Connection_wrong"
# subject = "multiprocessing.connection.answer_challenge(connection: Connection)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/connection.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.connection.answer_challenge(connection: Connection); call it with the wrong type.

typeshed contract: connection is Connection. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.connection import answer_challenge
try:
    answer_challenge(_W(), b"")  # connection: Connection <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/multiprocessing_connection/deliver_challenge__connection_as_Connection_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_connection_deliver_challenge__connection_as_Connection_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_connection"
# dimension = "type"
# case = "deliver_challenge__connection_as_Connection_wrong"
# subject = "multiprocessing.connection.deliver_challenge(connection: Connection)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/connection.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.connection.deliver_challenge(connection: Connection); call it with the wrong type.

typeshed contract: connection is Connection. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.connection import deliver_challenge
try:
    deliver_challenge(_W(), b"")  # connection: Connection <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/multiprocessing_connection/wait__object_list_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_connection_wait__object_list_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_connection"
# dimension = "type"
# case = "wait__object_list_as_Iterable_wrong"
# subject = "multiprocessing.connection.wait(object_list: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/connection.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.connection.wait(object_list: Iterable); call it with the wrong type.

typeshed contract: object_list is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.connection import wait
try:
    wait(_W())  # object_list: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
