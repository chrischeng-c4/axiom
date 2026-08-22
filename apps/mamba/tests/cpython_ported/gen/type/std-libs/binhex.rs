use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/binhex/BinHex__init__name_finfo_dlen_rlen_as__FileInfoTuple_wrong.py`.
#[test]
fn test_gen_type_std_libs_binhex_BinHex__init__name_finfo_dlen_rlen_as__FileInfoTuple_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binhex"
# dimension = "type"
# case = "BinHex__init__name_finfo_dlen_rlen_as__FileInfoTuple_wrong"
# subject = "binhex.BinHex.__init__(name_finfo_dlen_rlen: _FileInfoTuple)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/binhex.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: binhex.BinHex.__init__(name_finfo_dlen_rlen: _FileInfoTuple); call it with the wrong type.

typeshed contract: name_finfo_dlen_rlen is _FileInfoTuple. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from binhex import BinHex
try:
    BinHex(_W(), None)  # name_finfo_dlen_rlen: _FileInfoTuple <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/binhex/BinHex__write__data_as_SizedBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_binhex_BinHex__write__data_as_SizedBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binhex"
# dimension = "type"
# case = "BinHex__write__data_as_SizedBuffer_wrong"
# subject = "binhex.BinHex.write(data: SizedBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/binhex.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: binhex.BinHex.write(data: SizedBuffer); call it with the wrong type.

typeshed contract: data is SizedBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from binhex import BinHex
obj = object.__new__(BinHex)
try:
    obj.write(_W())  # data: SizedBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/binhex/BinHex__write_rsrc__data_as_SizedBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_binhex_BinHex__write_rsrc__data_as_SizedBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binhex"
# dimension = "type"
# case = "BinHex__write_rsrc__data_as_SizedBuffer_wrong"
# subject = "binhex.BinHex.write_rsrc(data: SizedBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/binhex.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: binhex.BinHex.write_rsrc(data: SizedBuffer); call it with the wrong type.

typeshed contract: data is SizedBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from binhex import BinHex
obj = object.__new__(BinHex)
try:
    obj.write_rsrc(_W())  # data: SizedBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/binhex/HexBin__init__ifp_as__FileHandleUnion_wrong.py`.
#[test]
fn test_gen_type_std_libs_binhex_HexBin__init__ifp_as__FileHandleUnion_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binhex"
# dimension = "type"
# case = "HexBin__init__ifp_as__FileHandleUnion_wrong"
# subject = "binhex.HexBin.__init__(ifp: _FileHandleUnion)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/binhex.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: binhex.HexBin.__init__(ifp: _FileHandleUnion); call it with the wrong type.

typeshed contract: ifp is _FileHandleUnion. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from binhex import HexBin
try:
    HexBin(_W())  # ifp: _FileHandleUnion <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/binhex/binhex__inp_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_binhex_binhex__inp_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binhex"
# dimension = "type"
# case = "binhex__inp_as_str_wrong"
# subject = "binhex.binhex(inp: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/binhex.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: binhex.binhex(inp: str); call it with the wrong type.

typeshed contract: inp is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from binhex import binhex
try:
    binhex(12345, "")  # inp: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/binhex/getfileinfo__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_binhex_getfileinfo__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binhex"
# dimension = "type"
# case = "getfileinfo__name_as_str_wrong"
# subject = "binhex.getfileinfo(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/binhex.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: binhex.getfileinfo(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from binhex import getfileinfo
try:
    getfileinfo(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/binhex/hexbin__inp_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_binhex_hexbin__inp_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binhex"
# dimension = "type"
# case = "hexbin__inp_as_str_wrong"
# subject = "binhex.hexbin(inp: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/binhex.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: binhex.hexbin(inp: str); call it with the wrong type.

typeshed contract: inp is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from binhex import hexbin
try:
    hexbin(12345, "")  # inp: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
