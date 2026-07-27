use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/Binary__decode__data_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_Binary__decode__data_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "Binary__decode__data_as_ReadableBuffer_wrong"
# subject = "xmlrpc.client.Binary.decode(data: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.Binary.decode(data: ReadableBuffer); call it with the wrong type.

typeshed contract: data is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xmlrpc.client import Binary
obj = object.__new__(Binary)
try:
    obj.decode(_W())  # data: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/Binary__encode__out_as_SupportsWrite_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_Binary__encode__out_as_SupportsWrite_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "Binary__encode__out_as_SupportsWrite_wrong"
# subject = "xmlrpc.client.Binary.encode(out: SupportsWrite)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.Binary.encode(out: SupportsWrite); call it with the wrong type.

typeshed contract: out is SupportsWrite. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xmlrpc.client import Binary
obj = object.__new__(Binary)
try:
    obj.encode(_W())  # out: SupportsWrite <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/Binary__init__data_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_Binary__init__data_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "Binary__init__data_as_typed_wrong"
# subject = "xmlrpc.client.Binary.__init__(data: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.Binary.__init__(data: typed); call it with the wrong type.

typeshed contract: data is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xmlrpc.client import Binary
try:
    Binary(_W())  # data: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/DateTime____eq____other_as__DateTimeComparable_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_DateTime____eq____other_as__DateTimeComparable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "DateTime____eq____other_as__DateTimeComparable_wrong"
# subject = "xmlrpc.client.DateTime.__eq__(other: _DateTimeComparable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.DateTime.__eq__(other: _DateTimeComparable); call it with the wrong type.

typeshed contract: other is _DateTimeComparable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xmlrpc.client import DateTime
obj = object.__new__(DateTime)
try:
    obj.__eq__(_W())  # other: _DateTimeComparable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/DateTime____ge____other_as__DateTimeComparable_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_DateTime____ge____other_as__DateTimeComparable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "DateTime____ge____other_as__DateTimeComparable_wrong"
# subject = "xmlrpc.client.DateTime.__ge__(other: _DateTimeComparable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.DateTime.__ge__(other: _DateTimeComparable); call it with the wrong type.

typeshed contract: other is _DateTimeComparable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xmlrpc.client import DateTime
obj = object.__new__(DateTime)
try:
    obj.__ge__(_W())  # other: _DateTimeComparable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/DateTime____gt____other_as__DateTimeComparable_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_DateTime____gt____other_as__DateTimeComparable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "DateTime____gt____other_as__DateTimeComparable_wrong"
# subject = "xmlrpc.client.DateTime.__gt__(other: _DateTimeComparable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.DateTime.__gt__(other: _DateTimeComparable); call it with the wrong type.

typeshed contract: other is _DateTimeComparable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xmlrpc.client import DateTime
obj = object.__new__(DateTime)
try:
    obj.__gt__(_W())  # other: _DateTimeComparable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/DateTime____le____other_as__DateTimeComparable_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_DateTime____le____other_as__DateTimeComparable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "DateTime____le____other_as__DateTimeComparable_wrong"
# subject = "xmlrpc.client.DateTime.__le__(other: _DateTimeComparable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.DateTime.__le__(other: _DateTimeComparable); call it with the wrong type.

typeshed contract: other is _DateTimeComparable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xmlrpc.client import DateTime
obj = object.__new__(DateTime)
try:
    obj.__le__(_W())  # other: _DateTimeComparable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/DateTime____lt____other_as__DateTimeComparable_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_DateTime____lt____other_as__DateTimeComparable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "DateTime____lt____other_as__DateTimeComparable_wrong"
# subject = "xmlrpc.client.DateTime.__lt__(other: _DateTimeComparable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.DateTime.__lt__(other: _DateTimeComparable); call it with the wrong type.

typeshed contract: other is _DateTimeComparable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xmlrpc.client import DateTime
obj = object.__new__(DateTime)
try:
    obj.__lt__(_W())  # other: _DateTimeComparable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/DateTime__encode__out_as_SupportsWrite_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_DateTime__encode__out_as_SupportsWrite_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "DateTime__encode__out_as_SupportsWrite_wrong"
# subject = "xmlrpc.client.DateTime.encode(out: SupportsWrite)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.DateTime.encode(out: SupportsWrite); call it with the wrong type.

typeshed contract: out is SupportsWrite. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xmlrpc.client import DateTime
obj = object.__new__(DateTime)
try:
    obj.encode(_W())  # out: SupportsWrite <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/DateTime__make_comparable__other_as__DateTimeComparable_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_DateTime__make_comparable__other_as__DateTimeComparable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "DateTime__make_comparable__other_as__DateTimeComparable_wrong"
# subject = "xmlrpc.client.DateTime.make_comparable(other: _DateTimeComparable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.DateTime.make_comparable(other: _DateTimeComparable); call it with the wrong type.

typeshed contract: other is _DateTimeComparable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xmlrpc.client import DateTime
obj = object.__new__(DateTime)
try:
    obj.make_comparable(_W())  # other: _DateTimeComparable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/ExpatParser__feed__data_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_ExpatParser__feed__data_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "ExpatParser__feed__data_as_typed_wrong"
# subject = "xmlrpc.client.ExpatParser.feed(data: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.ExpatParser.feed(data: typed); call it with the wrong type.

typeshed contract: data is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xmlrpc.client import ExpatParser
obj = object.__new__(ExpatParser)
try:
    obj.feed(_W())  # data: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/ExpatParser__init__target_as_Unmarshaller_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_ExpatParser__init__target_as_Unmarshaller_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "ExpatParser__init__target_as_Unmarshaller_wrong"
# subject = "xmlrpc.client.ExpatParser.__init__(target: Unmarshaller)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.ExpatParser.__init__(target: Unmarshaller); call it with the wrong type.

typeshed contract: target is Unmarshaller. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xmlrpc.client import ExpatParser
try:
    ExpatParser(_W())  # target: Unmarshaller <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/Fault__init__faultCode_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_Fault__init__faultCode_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "Fault__init__faultCode_as_int_wrong"
# subject = "xmlrpc.client.Fault.__init__(faultCode: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.Fault.__init__(faultCode: int); call it with the wrong type.

typeshed contract: faultCode is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xmlrpc.client import Fault
try:
    Fault("not_an_int", "")  # faultCode: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/GzipDecodedResponse__init__response_as_SupportsRead_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_GzipDecodedResponse__init__response_as_SupportsRead_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "GzipDecodedResponse__init__response_as_SupportsRead_wrong"
# subject = "xmlrpc.client.GzipDecodedResponse.__init__(response: SupportsRead)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.GzipDecodedResponse.__init__(response: SupportsRead); call it with the wrong type.

typeshed contract: response is SupportsRead. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xmlrpc.client import GzipDecodedResponse
try:
    GzipDecodedResponse(_W())  # response: SupportsRead <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/Marshaller__dump_array__value_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_Marshaller__dump_array__value_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "Marshaller__dump_array__value_as_Iterable_wrong"
# subject = "xmlrpc.client.Marshaller.dump_array(value: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.Marshaller.dump_array(value: Iterable); call it with the wrong type.

typeshed contract: value is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xmlrpc.client import Marshaller
obj = object.__new__(Marshaller)
try:
    obj.dump_array(_W(), None)  # value: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/Marshaller__dump_bytes__value_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_Marshaller__dump_bytes__value_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "Marshaller__dump_bytes__value_as_ReadableBuffer_wrong"
# subject = "xmlrpc.client.Marshaller.dump_bytes(value: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.Marshaller.dump_bytes(value: ReadableBuffer); call it with the wrong type.

typeshed contract: value is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xmlrpc.client import Marshaller
obj = object.__new__(Marshaller)
try:
    obj.dump_bytes(_W(), None)  # value: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/Marshaller__dump_datetime__value_as__XMLDate_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_Marshaller__dump_datetime__value_as__XMLDate_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "Marshaller__dump_datetime__value_as__XMLDate_wrong"
# subject = "xmlrpc.client.Marshaller.dump_datetime(value: _XMLDate)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.Marshaller.dump_datetime(value: _XMLDate); call it with the wrong type.

typeshed contract: value is _XMLDate. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xmlrpc.client import Marshaller
obj = object.__new__(Marshaller)
try:
    obj.dump_datetime(_W(), None)  # value: _XMLDate <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/Marshaller__dump_double__value_as_float_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_Marshaller__dump_double__value_as_float_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "Marshaller__dump_double__value_as_float_wrong"
# subject = "xmlrpc.client.Marshaller.dump_double(value: float)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.Marshaller.dump_double(value: float); call it with the wrong type.

typeshed contract: value is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xmlrpc.client import Marshaller
obj = object.__new__(Marshaller)
try:
    obj.dump_double("not_a_float", None)  # value: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/Marshaller__dump_instance__write_as__WriteCallback_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_Marshaller__dump_instance__write_as__WriteCallback_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "Marshaller__dump_instance__write_as__WriteCallback_wrong"
# subject = "xmlrpc.client.Marshaller.dump_instance(write: _WriteCallback)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.Marshaller.dump_instance(write: _WriteCallback); call it with the wrong type.

typeshed contract: write is _WriteCallback. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xmlrpc.client import Marshaller
obj = object.__new__(Marshaller)
try:
    obj.dump_instance(None, _W())  # write: _WriteCallback <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/Marshaller__dump_int__value_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_Marshaller__dump_int__value_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "Marshaller__dump_int__value_as_int_wrong"
# subject = "xmlrpc.client.Marshaller.dump_int(value: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.Marshaller.dump_int(value: int); call it with the wrong type.

typeshed contract: value is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xmlrpc.client import Marshaller
obj = object.__new__(Marshaller)
try:
    obj.dump_int("not_an_int", None)  # value: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/Marshaller__dump_long__value_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_Marshaller__dump_long__value_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "Marshaller__dump_long__value_as_int_wrong"
# subject = "xmlrpc.client.Marshaller.dump_long(value: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.Marshaller.dump_long(value: int); call it with the wrong type.

typeshed contract: value is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xmlrpc.client import Marshaller
obj = object.__new__(Marshaller)
try:
    obj.dump_long("not_an_int", None)  # value: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/Marshaller__dump_unicode__value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_Marshaller__dump_unicode__value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "Marshaller__dump_unicode__value_as_str_wrong"
# subject = "xmlrpc.client.Marshaller.dump_unicode(value: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.Marshaller.dump_unicode(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xmlrpc.client import Marshaller
obj = object.__new__(Marshaller)
try:
    obj.dump_unicode(12345, None)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/Marshaller__dumps__values_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_Marshaller__dumps__values_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "Marshaller__dumps__values_as_typed_wrong"
# subject = "xmlrpc.client.Marshaller.dumps(values: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.Marshaller.dumps(values: typed); call it with the wrong type.

typeshed contract: values is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xmlrpc.client import Marshaller
obj = object.__new__(Marshaller)
try:
    obj.dumps(_W())  # values: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/Marshaller__init__encoding_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_Marshaller__init__encoding_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "Marshaller__init__encoding_as_typed_wrong"
# subject = "xmlrpc.client.Marshaller.__init__(encoding: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.Marshaller.__init__(encoding: typed); call it with the wrong type.

typeshed contract: encoding is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xmlrpc.client import Marshaller
try:
    Marshaller(_W())  # encoding: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/MultiCallIterator____getitem____i_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_MultiCallIterator____getitem____i_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "MultiCallIterator____getitem____i_as_int_wrong"
# subject = "xmlrpc.client.MultiCallIterator.__getitem__(i: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.MultiCallIterator.__getitem__(i: int); call it with the wrong type.

typeshed contract: i is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xmlrpc.client import MultiCallIterator
obj = object.__new__(MultiCallIterator)
try:
    obj.__getitem__("not_an_int")  # i: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/MultiCall____getattr____name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_MultiCall____getattr____name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "MultiCall____getattr____name_as_str_wrong"
# subject = "xmlrpc.client.MultiCall.__getattr__(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.MultiCall.__getattr__(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xmlrpc.client import MultiCall
obj = object.__new__(MultiCall)
try:
    obj.__getattr__(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/MultiCall__init__server_as_ServerProxy_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_MultiCall__init__server_as_ServerProxy_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "MultiCall__init__server_as_ServerProxy_wrong"
# subject = "xmlrpc.client.MultiCall.__init__(server: ServerProxy)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.MultiCall.__init__(server: ServerProxy); call it with the wrong type.

typeshed contract: server is ServerProxy. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xmlrpc.client import MultiCall
try:
    MultiCall(_W())  # server: ServerProxy <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/ProtocolError__init__url_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_ProtocolError__init__url_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "ProtocolError__init__url_as_str_wrong"
# subject = "xmlrpc.client.ProtocolError.__init__(url: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.ProtocolError.__init__(url: str); call it with the wrong type.

typeshed contract: url is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xmlrpc.client import ProtocolError
try:
    ProtocolError(12345, 0, "", None)  # url: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/SafeTransport__make_connection__host_as__HostType_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_SafeTransport__make_connection__host_as__HostType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "SafeTransport__make_connection__host_as__HostType_wrong"
# subject = "xmlrpc.client.SafeTransport.make_connection(host: _HostType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.SafeTransport.make_connection(host: _HostType); call it with the wrong type.

typeshed contract: host is _HostType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xmlrpc.client import SafeTransport
obj = object.__new__(SafeTransport)
try:
    obj.make_connection(_W())  # host: _HostType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/ServerProxy____getattr____name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_ServerProxy____getattr____name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "ServerProxy____getattr____name_as_str_wrong"
# subject = "xmlrpc.client.ServerProxy.__getattr__(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.ServerProxy.__getattr__(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xmlrpc.client import ServerProxy
obj = object.__new__(ServerProxy)
try:
    obj.__getattr__(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/ServerProxy__init__uri_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_ServerProxy__init__uri_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "ServerProxy__init__uri_as_str_wrong"
# subject = "xmlrpc.client.ServerProxy.__init__(uri: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.ServerProxy.__init__(uri: str); call it with the wrong type.

typeshed contract: uri is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xmlrpc.client import ServerProxy
try:
    ServerProxy(12345)  # uri: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/Transport__get_host_info__host_as__HostType_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_Transport__get_host_info__host_as__HostType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "Transport__get_host_info__host_as__HostType_wrong"
# subject = "xmlrpc.client.Transport.get_host_info(host: _HostType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.Transport.get_host_info(host: _HostType); call it with the wrong type.

typeshed contract: host is _HostType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xmlrpc.client import Transport
obj = object.__new__(Transport)
try:
    obj.get_host_info(_W())  # host: _HostType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/Transport__make_connection__host_as__HostType_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_Transport__make_connection__host_as__HostType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "Transport__make_connection__host_as__HostType_wrong"
# subject = "xmlrpc.client.Transport.make_connection(host: _HostType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.Transport.make_connection(host: _HostType); call it with the wrong type.

typeshed contract: host is _HostType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xmlrpc.client import Transport
obj = object.__new__(Transport)
try:
    obj.make_connection(_W())  # host: _HostType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/Transport__parse_response__response_as_HTTPResponse_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_Transport__parse_response__response_as_HTTPResponse_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "Transport__parse_response__response_as_HTTPResponse_wrong"
# subject = "xmlrpc.client.Transport.parse_response(response: HTTPResponse)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.Transport.parse_response(response: HTTPResponse); call it with the wrong type.

typeshed contract: response is HTTPResponse. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xmlrpc.client import Transport
obj = object.__new__(Transport)
try:
    obj.parse_response(_W())  # response: HTTPResponse <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/Transport__request__host_as__HostType_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_Transport__request__host_as__HostType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "Transport__request__host_as__HostType_wrong"
# subject = "xmlrpc.client.Transport.request(host: _HostType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.Transport.request(host: _HostType); call it with the wrong type.

typeshed contract: host is _HostType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xmlrpc.client import Transport
obj = object.__new__(Transport)
try:
    obj.request(_W(), "", None)  # host: _HostType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/Transport__send_content__connection_as_HTTPConnection_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_Transport__send_content__connection_as_HTTPConnection_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "Transport__send_content__connection_as_HTTPConnection_wrong"
# subject = "xmlrpc.client.Transport.send_content(connection: HTTPConnection)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.Transport.send_content(connection: HTTPConnection); call it with the wrong type.

typeshed contract: connection is HTTPConnection. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xmlrpc.client import Transport
obj = object.__new__(Transport)
try:
    obj.send_content(_W(), None)  # connection: HTTPConnection <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/Transport__send_headers__connection_as_HTTPConnection_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_Transport__send_headers__connection_as_HTTPConnection_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "Transport__send_headers__connection_as_HTTPConnection_wrong"
# subject = "xmlrpc.client.Transport.send_headers(connection: HTTPConnection)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.Transport.send_headers(connection: HTTPConnection); call it with the wrong type.

typeshed contract: connection is HTTPConnection. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xmlrpc.client import Transport
obj = object.__new__(Transport)
try:
    obj.send_headers(_W(), None)  # connection: HTTPConnection <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/Transport__send_request__host_as__HostType_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_Transport__send_request__host_as__HostType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "Transport__send_request__host_as__HostType_wrong"
# subject = "xmlrpc.client.Transport.send_request(host: _HostType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.Transport.send_request(host: _HostType); call it with the wrong type.

typeshed contract: host is _HostType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xmlrpc.client import Transport
obj = object.__new__(Transport)
try:
    obj.send_request(_W(), "", None, True)  # host: _HostType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/Transport__single_request__host_as__HostType_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_Transport__single_request__host_as__HostType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "Transport__single_request__host_as__HostType_wrong"
# subject = "xmlrpc.client.Transport.single_request(host: _HostType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.Transport.single_request(host: _HostType); call it with the wrong type.

typeshed contract: host is _HostType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xmlrpc.client import Transport
obj = object.__new__(Transport)
try:
    obj.single_request(_W(), "", None)  # host: _HostType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/Unmarshaller__data__text_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_Unmarshaller__data__text_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "Unmarshaller__data__text_as_str_wrong"
# subject = "xmlrpc.client.Unmarshaller.data(text: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.Unmarshaller.data(text: str); call it with the wrong type.

typeshed contract: text is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xmlrpc.client import Unmarshaller
obj = object.__new__(Unmarshaller)
try:
    obj.data(12345)  # text: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/Unmarshaller__end__tag_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_Unmarshaller__end__tag_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "Unmarshaller__end__tag_as_str_wrong"
# subject = "xmlrpc.client.Unmarshaller.end(tag: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.Unmarshaller.end(tag: str); call it with the wrong type.

typeshed contract: tag is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xmlrpc.client import Unmarshaller
obj = object.__new__(Unmarshaller)
try:
    obj.end(12345)  # tag: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/Unmarshaller__end_array__data_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_Unmarshaller__end_array__data_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "Unmarshaller__end_array__data_as_str_wrong"
# subject = "xmlrpc.client.Unmarshaller.end_array(data: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.Unmarshaller.end_array(data: str); call it with the wrong type.

typeshed contract: data is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xmlrpc.client import Unmarshaller
obj = object.__new__(Unmarshaller)
try:
    obj.end_array(12345)  # data: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/Unmarshaller__end_base64__data_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_Unmarshaller__end_base64__data_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "Unmarshaller__end_base64__data_as_str_wrong"
# subject = "xmlrpc.client.Unmarshaller.end_base64(data: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.Unmarshaller.end_base64(data: str); call it with the wrong type.

typeshed contract: data is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xmlrpc.client import Unmarshaller
obj = object.__new__(Unmarshaller)
try:
    obj.end_base64(12345)  # data: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/Unmarshaller__end_bigdecimal__data_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_Unmarshaller__end_bigdecimal__data_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "Unmarshaller__end_bigdecimal__data_as_str_wrong"
# subject = "xmlrpc.client.Unmarshaller.end_bigdecimal(data: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.Unmarshaller.end_bigdecimal(data: str); call it with the wrong type.

typeshed contract: data is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xmlrpc.client import Unmarshaller
obj = object.__new__(Unmarshaller)
try:
    obj.end_bigdecimal(12345)  # data: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/Unmarshaller__end_boolean__data_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_Unmarshaller__end_boolean__data_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "Unmarshaller__end_boolean__data_as_str_wrong"
# subject = "xmlrpc.client.Unmarshaller.end_boolean(data: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.Unmarshaller.end_boolean(data: str); call it with the wrong type.

typeshed contract: data is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xmlrpc.client import Unmarshaller
obj = object.__new__(Unmarshaller)
try:
    obj.end_boolean(12345)  # data: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/Unmarshaller__end_dateTime__data_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_Unmarshaller__end_dateTime__data_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "Unmarshaller__end_dateTime__data_as_str_wrong"
# subject = "xmlrpc.client.Unmarshaller.end_dateTime(data: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.Unmarshaller.end_dateTime(data: str); call it with the wrong type.

typeshed contract: data is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xmlrpc.client import Unmarshaller
obj = object.__new__(Unmarshaller)
try:
    obj.end_dateTime(12345)  # data: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/Unmarshaller__end_dispatch__tag_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_Unmarshaller__end_dispatch__tag_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "Unmarshaller__end_dispatch__tag_as_str_wrong"
# subject = "xmlrpc.client.Unmarshaller.end_dispatch(tag: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.Unmarshaller.end_dispatch(tag: str); call it with the wrong type.

typeshed contract: tag is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xmlrpc.client import Unmarshaller
obj = object.__new__(Unmarshaller)
try:
    obj.end_dispatch(12345, "")  # tag: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/Unmarshaller__end_double__data_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_Unmarshaller__end_double__data_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "Unmarshaller__end_double__data_as_str_wrong"
# subject = "xmlrpc.client.Unmarshaller.end_double(data: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.Unmarshaller.end_double(data: str); call it with the wrong type.

typeshed contract: data is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xmlrpc.client import Unmarshaller
obj = object.__new__(Unmarshaller)
try:
    obj.end_double(12345)  # data: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/Unmarshaller__end_fault__data_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_Unmarshaller__end_fault__data_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "Unmarshaller__end_fault__data_as_str_wrong"
# subject = "xmlrpc.client.Unmarshaller.end_fault(data: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.Unmarshaller.end_fault(data: str); call it with the wrong type.

typeshed contract: data is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xmlrpc.client import Unmarshaller
obj = object.__new__(Unmarshaller)
try:
    obj.end_fault(12345)  # data: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/Unmarshaller__end_int__data_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_Unmarshaller__end_int__data_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "Unmarshaller__end_int__data_as_str_wrong"
# subject = "xmlrpc.client.Unmarshaller.end_int(data: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.Unmarshaller.end_int(data: str); call it with the wrong type.

typeshed contract: data is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xmlrpc.client import Unmarshaller
obj = object.__new__(Unmarshaller)
try:
    obj.end_int(12345)  # data: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/Unmarshaller__end_methodName__data_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_Unmarshaller__end_methodName__data_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "Unmarshaller__end_methodName__data_as_str_wrong"
# subject = "xmlrpc.client.Unmarshaller.end_methodName(data: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.Unmarshaller.end_methodName(data: str); call it with the wrong type.

typeshed contract: data is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xmlrpc.client import Unmarshaller
obj = object.__new__(Unmarshaller)
try:
    obj.end_methodName(12345)  # data: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/Unmarshaller__end_nil__data_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_Unmarshaller__end_nil__data_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "Unmarshaller__end_nil__data_as_str_wrong"
# subject = "xmlrpc.client.Unmarshaller.end_nil(data: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.Unmarshaller.end_nil(data: str); call it with the wrong type.

typeshed contract: data is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xmlrpc.client import Unmarshaller
obj = object.__new__(Unmarshaller)
try:
    obj.end_nil(12345)  # data: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/Unmarshaller__end_params__data_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_Unmarshaller__end_params__data_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "Unmarshaller__end_params__data_as_str_wrong"
# subject = "xmlrpc.client.Unmarshaller.end_params(data: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.Unmarshaller.end_params(data: str); call it with the wrong type.

typeshed contract: data is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xmlrpc.client import Unmarshaller
obj = object.__new__(Unmarshaller)
try:
    obj.end_params(12345)  # data: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/Unmarshaller__end_string__data_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_Unmarshaller__end_string__data_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "Unmarshaller__end_string__data_as_str_wrong"
# subject = "xmlrpc.client.Unmarshaller.end_string(data: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.Unmarshaller.end_string(data: str); call it with the wrong type.

typeshed contract: data is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xmlrpc.client import Unmarshaller
obj = object.__new__(Unmarshaller)
try:
    obj.end_string(12345)  # data: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/Unmarshaller__end_struct__data_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_Unmarshaller__end_struct__data_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "Unmarshaller__end_struct__data_as_str_wrong"
# subject = "xmlrpc.client.Unmarshaller.end_struct(data: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.Unmarshaller.end_struct(data: str); call it with the wrong type.

typeshed contract: data is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xmlrpc.client import Unmarshaller
obj = object.__new__(Unmarshaller)
try:
    obj.end_struct(12345)  # data: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/Unmarshaller__end_value__data_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_Unmarshaller__end_value__data_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "Unmarshaller__end_value__data_as_str_wrong"
# subject = "xmlrpc.client.Unmarshaller.end_value(data: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.Unmarshaller.end_value(data: str); call it with the wrong type.

typeshed contract: data is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xmlrpc.client import Unmarshaller
obj = object.__new__(Unmarshaller)
try:
    obj.end_value(12345)  # data: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/Unmarshaller__start__tag_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_Unmarshaller__start__tag_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "Unmarshaller__start__tag_as_str_wrong"
# subject = "xmlrpc.client.Unmarshaller.start(tag: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.Unmarshaller.start(tag: str); call it with the wrong type.

typeshed contract: tag is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xmlrpc.client import Unmarshaller
obj = object.__new__(Unmarshaller)
try:
    obj.start(12345, None)  # tag: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/Unmarshaller__xml__encoding_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_Unmarshaller__xml__encoding_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "Unmarshaller__xml__encoding_as_str_wrong"
# subject = "xmlrpc.client.Unmarshaller.xml(encoding: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.Unmarshaller.xml(encoding: str); call it with the wrong type.

typeshed contract: encoding is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xmlrpc.client import Unmarshaller
obj = object.__new__(Unmarshaller)
try:
    obj.xml(12345, None)  # encoding: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/escape__s_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_escape__s_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "escape__s_as_str_wrong"
# subject = "xmlrpc.client.escape(s: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.escape(s: str); call it with the wrong type.

typeshed contract: s is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xmlrpc.client import escape
try:
    escape(12345)  # s: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/gzip_decode__data_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_gzip_decode__data_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "gzip_decode__data_as_ReadableBuffer_wrong"
# subject = "xmlrpc.client.gzip_decode(data: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.gzip_decode(data: ReadableBuffer); call it with the wrong type.

typeshed contract: data is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xmlrpc.client import gzip_decode
try:
    gzip_decode(_W())  # data: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/gzip_encode__data_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_gzip_encode__data_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "gzip_encode__data_as_ReadableBuffer_wrong"
# subject = "xmlrpc.client.gzip_encode(data: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.gzip_encode(data: ReadableBuffer); call it with the wrong type.

typeshed contract: data is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xmlrpc.client import gzip_encode
try:
    gzip_encode(_W())  # data: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_client/loads__data_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_client_loads__data_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_client"
# dimension = "type"
# case = "loads__data_as_typed_wrong"
# subject = "xmlrpc.client.loads(data: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/client.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.client.loads(data: typed); call it with the wrong type.

typeshed contract: data is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xmlrpc.client import loads
try:
    loads(_W())  # data: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
