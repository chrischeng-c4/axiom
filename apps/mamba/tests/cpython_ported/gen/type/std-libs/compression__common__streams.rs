use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/compression__common__streams/DecompressReader__init__fp_as__Reader_wrong.py`.
#[test]
fn test_gen_type_std_libs_compression__common__streams_DecompressReader__init__fp_as__Reader_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compression__common__streams"
# dimension = "type"
# case = "DecompressReader__init__fp_as__Reader_wrong"
# subject = "compression._common._streams.DecompressReader.__init__(fp: _Reader)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/compression/_common/_streams.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: compression._common._streams.DecompressReader.__init__(fp: _Reader); call it with the wrong type.

typeshed contract: fp is _Reader. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from compression._common._streams import DecompressReader
try:
    DecompressReader(_W(), None)  # fp: _Reader <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/compression__common__streams/DecompressReader__read__size_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_compression__common__streams_DecompressReader__read__size_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compression__common__streams"
# dimension = "type"
# case = "DecompressReader__read__size_as_int_wrong"
# subject = "compression._common._streams.DecompressReader.read(size: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/compression/_common/_streams.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: compression._common._streams.DecompressReader.read(size: int); call it with the wrong type.

typeshed contract: size is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from compression._common._streams import DecompressReader
obj = object.__new__(DecompressReader)
try:
    obj.read("not_an_int")  # size: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/compression__common__streams/DecompressReader__readinto__b_as_WriteableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_compression__common__streams_DecompressReader__readinto__b_as_WriteableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compression__common__streams"
# dimension = "type"
# case = "DecompressReader__readinto__b_as_WriteableBuffer_wrong"
# subject = "compression._common._streams.DecompressReader.readinto(b: WriteableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/compression/_common/_streams.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: compression._common._streams.DecompressReader.readinto(b: WriteableBuffer); call it with the wrong type.

typeshed contract: b is WriteableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from compression._common._streams import DecompressReader
obj = object.__new__(DecompressReader)
try:
    obj.readinto(_W())  # b: WriteableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/compression__common__streams/DecompressReader__seek__offset_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_compression__common__streams_DecompressReader__seek__offset_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compression__common__streams"
# dimension = "type"
# case = "DecompressReader__seek__offset_as_int_wrong"
# subject = "compression._common._streams.DecompressReader.seek(offset: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/compression/_common/_streams.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: compression._common._streams.DecompressReader.seek(offset: int); call it with the wrong type.

typeshed contract: offset is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from compression._common._streams import DecompressReader
obj = object.__new__(DecompressReader)
try:
    obj.seek("not_an_int")  # offset: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
