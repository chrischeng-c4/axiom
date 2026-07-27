use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/_pickle/Pickler__init__file_as_SupportsWrite_wrong.py`.
#[test]
fn test_gen_type_std_libs__pickle_Pickler__init__file_as_SupportsWrite_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_pickle"
# dimension = "type"
# case = "Pickler__init__file_as_SupportsWrite_wrong"
# subject = "_pickle.Pickler.__init__(file: SupportsWrite)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_pickle.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _pickle.Pickler.__init__(file: SupportsWrite); call it with the wrong type.

typeshed contract: file is SupportsWrite. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _pickle import Pickler
try:
    Pickler(_W())  # file: SupportsWrite <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_pickle/Unpickler__find_class__module_name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__pickle_Unpickler__find_class__module_name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_pickle"
# dimension = "type"
# case = "Unpickler__find_class__module_name_as_str_wrong"
# subject = "_pickle.Unpickler.find_class(module_name: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_pickle.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _pickle.Unpickler.find_class(module_name: str); call it with the wrong type.

typeshed contract: module_name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _pickle import Unpickler
obj = object.__new__(Unpickler)
try:
    obj.find_class(12345, "")  # module_name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_pickle/Unpickler__init__file_as__ReadableFileobj_wrong.py`.
#[test]
fn test_gen_type_std_libs__pickle_Unpickler__init__file_as__ReadableFileobj_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_pickle"
# dimension = "type"
# case = "Unpickler__init__file_as__ReadableFileobj_wrong"
# subject = "_pickle.Unpickler.__init__(file: _ReadableFileobj)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_pickle.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _pickle.Unpickler.__init__(file: _ReadableFileobj); call it with the wrong type.

typeshed contract: file is _ReadableFileobj. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _pickle import Unpickler
try:
    Unpickler(_W())  # file: _ReadableFileobj <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_pickle/dump__file_as_SupportsWrite_wrong.py`.
#[test]
fn test_gen_type_std_libs__pickle_dump__file_as_SupportsWrite_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_pickle"
# dimension = "type"
# case = "dump__file_as_SupportsWrite_wrong"
# subject = "_pickle.dump(file: SupportsWrite)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_pickle.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _pickle.dump(file: SupportsWrite); call it with the wrong type.

typeshed contract: file is SupportsWrite. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _pickle import dump
try:
    dump(None, _W())  # file: SupportsWrite <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_pickle/dumps__protocol_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs__pickle_dumps__protocol_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_pickle"
# dimension = "type"
# case = "dumps__protocol_as_typed_wrong"
# subject = "_pickle.dumps(protocol: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_pickle.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _pickle.dumps(protocol: typed); call it with the wrong type.

typeshed contract: protocol is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _pickle import dumps
try:
    dumps(None, _W())  # protocol: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_pickle/load__file_as__ReadableFileobj_wrong.py`.
#[test]
fn test_gen_type_std_libs__pickle_load__file_as__ReadableFileobj_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_pickle"
# dimension = "type"
# case = "load__file_as__ReadableFileobj_wrong"
# subject = "_pickle.load(file: _ReadableFileobj)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_pickle.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _pickle.load(file: _ReadableFileobj); call it with the wrong type.

typeshed contract: file is _ReadableFileobj. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _pickle import load
try:
    load(_W())  # file: _ReadableFileobj <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_pickle/loads__data_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs__pickle_loads__data_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_pickle"
# dimension = "type"
# case = "loads__data_as_ReadableBuffer_wrong"
# subject = "_pickle.loads(data: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_pickle.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _pickle.loads(data: ReadableBuffer); call it with the wrong type.

typeshed contract: data is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _pickle import loads
try:
    loads(_W())  # data: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pickle/PickleBuffer____buffer____flags_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_pickle_PickleBuffer____buffer____flags_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "type"
# case = "PickleBuffer____buffer____flags_as_int_wrong"
# subject = "pickle.PickleBuffer.__buffer__(flags: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pickle.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pickle.PickleBuffer.__buffer__(flags: int); call it with the wrong type.

typeshed contract: flags is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pickle import PickleBuffer
obj = object.__new__(PickleBuffer)
try:
    obj.__buffer__("not_an_int")  # flags: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pickle/PickleBuffer____new____buffer_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_pickle_PickleBuffer____new____buffer_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "type"
# case = "PickleBuffer____new____buffer_as_ReadableBuffer_wrong"
# subject = "pickle.PickleBuffer.__new__(buffer: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pickle.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pickle.PickleBuffer.__new__(buffer: ReadableBuffer); call it with the wrong type.

typeshed contract: buffer is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pickle import PickleBuffer
obj = object.__new__(PickleBuffer)
try:
    obj.__new__(_W())  # buffer: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pickle/PickleBuffer____release_buffer____buffer_as_memoryview_wrong.py`.
#[test]
fn test_gen_type_std_libs_pickle_PickleBuffer____release_buffer____buffer_as_memoryview_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "type"
# case = "PickleBuffer____release_buffer____buffer_as_memoryview_wrong"
# subject = "pickle.PickleBuffer.__release_buffer__(buffer: memoryview)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pickle.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pickle.PickleBuffer.__release_buffer__(buffer: memoryview); call it with the wrong type.

typeshed contract: buffer is memoryview. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pickle import PickleBuffer
obj = object.__new__(PickleBuffer)
try:
    obj.__release_buffer__(12345)  # buffer: memoryview <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pickle/decode_long__data_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_pickle_decode_long__data_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "type"
# case = "decode_long__data_as_typed_wrong"
# subject = "pickle.decode_long(data: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pickle.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pickle.decode_long(data: typed); call it with the wrong type.

typeshed contract: data is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pickle import decode_long
try:
    decode_long(_W())  # data: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pickle/encode_long__x_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_pickle_encode_long__x_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "type"
# case = "encode_long__x_as_int_wrong"
# subject = "pickle.encode_long(x: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pickle.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pickle.encode_long(x: int); call it with the wrong type.

typeshed contract: x is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pickle import encode_long
try:
    encode_long("not_an_int")  # x: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
