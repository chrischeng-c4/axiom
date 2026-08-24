use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/_multibytecodec/MultibyteIncrementalDecoder__decode__input_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs__multibytecodec_MultibyteIncrementalDecoder__decode__input_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_multibytecodec"
# dimension = "type"
# case = "MultibyteIncrementalDecoder__decode__input_as_ReadableBuffer_wrong"
# subject = "_multibytecodec.MultibyteIncrementalDecoder.decode(input: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_multibytecodec.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _multibytecodec.MultibyteIncrementalDecoder.decode(input: ReadableBuffer); call it with the wrong type.

typeshed contract: input is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _multibytecodec import MultibyteIncrementalDecoder
obj = object.__new__(MultibyteIncrementalDecoder)
try:
    obj.decode(_W())  # input: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_multibytecodec/MultibyteIncrementalDecoder__init__errors_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__multibytecodec_MultibyteIncrementalDecoder__init__errors_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_multibytecodec"
# dimension = "type"
# case = "MultibyteIncrementalDecoder__init__errors_as_str_wrong"
# subject = "_multibytecodec.MultibyteIncrementalDecoder.__init__(errors: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_multibytecodec.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _multibytecodec.MultibyteIncrementalDecoder.__init__(errors: str); call it with the wrong type.

typeshed contract: errors is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _multibytecodec import MultibyteIncrementalDecoder
try:
    MultibyteIncrementalDecoder(12345)  # errors: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_multibytecodec/MultibyteIncrementalDecoder__setstate__state_as_tuple_wrong.py`.
#[test]
fn test_gen_type_std_libs__multibytecodec_MultibyteIncrementalDecoder__setstate__state_as_tuple_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_multibytecodec"
# dimension = "type"
# case = "MultibyteIncrementalDecoder__setstate__state_as_tuple_wrong"
# subject = "_multibytecodec.MultibyteIncrementalDecoder.setstate(state: tuple)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_multibytecodec.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _multibytecodec.MultibyteIncrementalDecoder.setstate(state: tuple); call it with the wrong type.

typeshed contract: state is tuple. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _multibytecodec import MultibyteIncrementalDecoder
obj = object.__new__(MultibyteIncrementalDecoder)
try:
    obj.setstate(12345)  # state: tuple <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_multibytecodec/MultibyteIncrementalEncoder__encode__input_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__multibytecodec_MultibyteIncrementalEncoder__encode__input_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_multibytecodec"
# dimension = "type"
# case = "MultibyteIncrementalEncoder__encode__input_as_str_wrong"
# subject = "_multibytecodec.MultibyteIncrementalEncoder.encode(input: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_multibytecodec.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _multibytecodec.MultibyteIncrementalEncoder.encode(input: str); call it with the wrong type.

typeshed contract: input is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _multibytecodec import MultibyteIncrementalEncoder
obj = object.__new__(MultibyteIncrementalEncoder)
try:
    obj.encode(12345)  # input: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_multibytecodec/MultibyteIncrementalEncoder__init__errors_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__multibytecodec_MultibyteIncrementalEncoder__init__errors_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_multibytecodec"
# dimension = "type"
# case = "MultibyteIncrementalEncoder__init__errors_as_str_wrong"
# subject = "_multibytecodec.MultibyteIncrementalEncoder.__init__(errors: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_multibytecodec.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _multibytecodec.MultibyteIncrementalEncoder.__init__(errors: str); call it with the wrong type.

typeshed contract: errors is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _multibytecodec import MultibyteIncrementalEncoder
try:
    MultibyteIncrementalEncoder(12345)  # errors: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_multibytecodec/MultibyteIncrementalEncoder__setstate__state_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__multibytecodec_MultibyteIncrementalEncoder__setstate__state_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_multibytecodec"
# dimension = "type"
# case = "MultibyteIncrementalEncoder__setstate__state_as_int_wrong"
# subject = "_multibytecodec.MultibyteIncrementalEncoder.setstate(state: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_multibytecodec.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _multibytecodec.MultibyteIncrementalEncoder.setstate(state: int); call it with the wrong type.

typeshed contract: state is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _multibytecodec import MultibyteIncrementalEncoder
obj = object.__new__(MultibyteIncrementalEncoder)
try:
    obj.setstate("not_an_int")  # state: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_multibytecodec/MultibyteStreamReader__init__stream_as__ReadableStream_wrong.py`.
#[test]
fn test_gen_type_std_libs__multibytecodec_MultibyteStreamReader__init__stream_as__ReadableStream_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_multibytecodec"
# dimension = "type"
# case = "MultibyteStreamReader__init__stream_as__ReadableStream_wrong"
# subject = "_multibytecodec.MultibyteStreamReader.__init__(stream: _ReadableStream)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_multibytecodec.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _multibytecodec.MultibyteStreamReader.__init__(stream: _ReadableStream); call it with the wrong type.

typeshed contract: stream is _ReadableStream. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _multibytecodec import MultibyteStreamReader
try:
    MultibyteStreamReader(_W())  # stream: _ReadableStream <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_multibytecodec/MultibyteStreamReader__read__sizeobj_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs__multibytecodec_MultibyteStreamReader__read__sizeobj_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_multibytecodec"
# dimension = "type"
# case = "MultibyteStreamReader__read__sizeobj_as_typed_wrong"
# subject = "_multibytecodec.MultibyteStreamReader.read(sizeobj: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_multibytecodec.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _multibytecodec.MultibyteStreamReader.read(sizeobj: typed); call it with the wrong type.

typeshed contract: sizeobj is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _multibytecodec import MultibyteStreamReader
obj = object.__new__(MultibyteStreamReader)
try:
    obj.read(_W())  # sizeobj: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_multibytecodec/MultibyteStreamReader__readline__sizeobj_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs__multibytecodec_MultibyteStreamReader__readline__sizeobj_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_multibytecodec"
# dimension = "type"
# case = "MultibyteStreamReader__readline__sizeobj_as_typed_wrong"
# subject = "_multibytecodec.MultibyteStreamReader.readline(sizeobj: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_multibytecodec.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _multibytecodec.MultibyteStreamReader.readline(sizeobj: typed); call it with the wrong type.

typeshed contract: sizeobj is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _multibytecodec import MultibyteStreamReader
obj = object.__new__(MultibyteStreamReader)
try:
    obj.readline(_W())  # sizeobj: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_multibytecodec/MultibyteStreamReader__readlines__sizehintobj_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs__multibytecodec_MultibyteStreamReader__readlines__sizehintobj_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_multibytecodec"
# dimension = "type"
# case = "MultibyteStreamReader__readlines__sizehintobj_as_typed_wrong"
# subject = "_multibytecodec.MultibyteStreamReader.readlines(sizehintobj: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_multibytecodec.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _multibytecodec.MultibyteStreamReader.readlines(sizehintobj: typed); call it with the wrong type.

typeshed contract: sizehintobj is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _multibytecodec import MultibyteStreamReader
obj = object.__new__(MultibyteStreamReader)
try:
    obj.readlines(_W())  # sizehintobj: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_multibytecodec/MultibyteStreamWriter__init__stream_as__WritableStream_wrong.py`.
#[test]
fn test_gen_type_std_libs__multibytecodec_MultibyteStreamWriter__init__stream_as__WritableStream_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_multibytecodec"
# dimension = "type"
# case = "MultibyteStreamWriter__init__stream_as__WritableStream_wrong"
# subject = "_multibytecodec.MultibyteStreamWriter.__init__(stream: _WritableStream)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_multibytecodec.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _multibytecodec.MultibyteStreamWriter.__init__(stream: _WritableStream); call it with the wrong type.

typeshed contract: stream is _WritableStream. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _multibytecodec import MultibyteStreamWriter
try:
    MultibyteStreamWriter(_W())  # stream: _WritableStream <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_multibytecodec/MultibyteStreamWriter__write__strobj_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__multibytecodec_MultibyteStreamWriter__write__strobj_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_multibytecodec"
# dimension = "type"
# case = "MultibyteStreamWriter__write__strobj_as_str_wrong"
# subject = "_multibytecodec.MultibyteStreamWriter.write(strobj: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_multibytecodec.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _multibytecodec.MultibyteStreamWriter.write(strobj: str); call it with the wrong type.

typeshed contract: strobj is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _multibytecodec import MultibyteStreamWriter
obj = object.__new__(MultibyteStreamWriter)
try:
    obj.write(12345)  # strobj: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_multibytecodec/MultibyteStreamWriter__writelines__lines_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs__multibytecodec_MultibyteStreamWriter__writelines__lines_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_multibytecodec"
# dimension = "type"
# case = "MultibyteStreamWriter__writelines__lines_as_Iterable_wrong"
# subject = "_multibytecodec.MultibyteStreamWriter.writelines(lines: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_multibytecodec.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _multibytecodec.MultibyteStreamWriter.writelines(lines: Iterable); call it with the wrong type.

typeshed contract: lines is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _multibytecodec import MultibyteStreamWriter
obj = object.__new__(MultibyteStreamWriter)
try:
    obj.writelines(_W())  # lines: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
