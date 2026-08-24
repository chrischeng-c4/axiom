use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/aifc/Aifc_read__getmark__id_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_aifc_Aifc_read__getmark__id_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "aifc"
# dimension = "type"
# case = "Aifc_read__getmark__id_as_int_wrong"
# subject = "aifc.Aifc_read.getmark(id: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/aifc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: aifc.Aifc_read.getmark(id: int); call it with the wrong type.

typeshed contract: id is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from aifc import Aifc_read
obj = object.__new__(Aifc_read)
try:
    obj.getmark("not_an_int")  # id: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/aifc/Aifc_read__init__f_as__File_wrong.py`.
#[test]
fn test_gen_type_std_libs_aifc_Aifc_read__init__f_as__File_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "aifc"
# dimension = "type"
# case = "Aifc_read__init__f_as__File_wrong"
# subject = "aifc.Aifc_read.__init__(f: _File)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/aifc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: aifc.Aifc_read.__init__(f: _File); call it with the wrong type.

typeshed contract: f is _File. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from aifc import Aifc_read
try:
    Aifc_read(_W())  # f: _File <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/aifc/Aifc_read__initfp__file_as_IO_wrong.py`.
#[test]
fn test_gen_type_std_libs_aifc_Aifc_read__initfp__file_as_IO_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "aifc"
# dimension = "type"
# case = "Aifc_read__initfp__file_as_IO_wrong"
# subject = "aifc.Aifc_read.initfp(file: IO)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/aifc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: aifc.Aifc_read.initfp(file: IO); call it with the wrong type.

typeshed contract: file is IO. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from aifc import Aifc_read
obj = object.__new__(Aifc_read)
try:
    obj.initfp(_W())  # file: IO <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/aifc/Aifc_read__readframes__nframes_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_aifc_Aifc_read__readframes__nframes_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "aifc"
# dimension = "type"
# case = "Aifc_read__readframes__nframes_as_int_wrong"
# subject = "aifc.Aifc_read.readframes(nframes: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/aifc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: aifc.Aifc_read.readframes(nframes: int); call it with the wrong type.

typeshed contract: nframes is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from aifc import Aifc_read
obj = object.__new__(Aifc_read)
try:
    obj.readframes("not_an_int")  # nframes: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/aifc/Aifc_read__setpos__pos_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_aifc_Aifc_read__setpos__pos_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "aifc"
# dimension = "type"
# case = "Aifc_read__setpos__pos_as_int_wrong"
# subject = "aifc.Aifc_read.setpos(pos: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/aifc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: aifc.Aifc_read.setpos(pos: int); call it with the wrong type.

typeshed contract: pos is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from aifc import Aifc_read
obj = object.__new__(Aifc_read)
try:
    obj.setpos("not_an_int")  # pos: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/aifc/Aifc_write__getmark__id_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_aifc_Aifc_write__getmark__id_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "aifc"
# dimension = "type"
# case = "Aifc_write__getmark__id_as_int_wrong"
# subject = "aifc.Aifc_write.getmark(id: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/aifc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: aifc.Aifc_write.getmark(id: int); call it with the wrong type.

typeshed contract: id is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from aifc import Aifc_write
obj = object.__new__(Aifc_write)
try:
    obj.getmark("not_an_int")  # id: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/aifc/Aifc_write__init__f_as__File_wrong.py`.
#[test]
fn test_gen_type_std_libs_aifc_Aifc_write__init__f_as__File_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "aifc"
# dimension = "type"
# case = "Aifc_write__init__f_as__File_wrong"
# subject = "aifc.Aifc_write.__init__(f: _File)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/aifc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: aifc.Aifc_write.__init__(f: _File); call it with the wrong type.

typeshed contract: f is _File. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from aifc import Aifc_write
try:
    Aifc_write(_W())  # f: _File <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/aifc/Aifc_write__initfp__file_as_IO_wrong.py`.
#[test]
fn test_gen_type_std_libs_aifc_Aifc_write__initfp__file_as_IO_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "aifc"
# dimension = "type"
# case = "Aifc_write__initfp__file_as_IO_wrong"
# subject = "aifc.Aifc_write.initfp(file: IO)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/aifc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: aifc.Aifc_write.initfp(file: IO); call it with the wrong type.

typeshed contract: file is IO. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from aifc import Aifc_write
obj = object.__new__(Aifc_write)
try:
    obj.initfp(_W())  # file: IO <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/aifc/Aifc_write__setframerate__framerate_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_aifc_Aifc_write__setframerate__framerate_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "aifc"
# dimension = "type"
# case = "Aifc_write__setframerate__framerate_as_int_wrong"
# subject = "aifc.Aifc_write.setframerate(framerate: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/aifc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: aifc.Aifc_write.setframerate(framerate: int); call it with the wrong type.

typeshed contract: framerate is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from aifc import Aifc_write
obj = object.__new__(Aifc_write)
try:
    obj.setframerate("not_an_int")  # framerate: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/aifc/Aifc_write__setmark__id_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_aifc_Aifc_write__setmark__id_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "aifc"
# dimension = "type"
# case = "Aifc_write__setmark__id_as_int_wrong"
# subject = "aifc.Aifc_write.setmark(id: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/aifc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: aifc.Aifc_write.setmark(id: int); call it with the wrong type.

typeshed contract: id is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from aifc import Aifc_write
obj = object.__new__(Aifc_write)
try:
    obj.setmark("not_an_int", 0, b"")  # id: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/aifc/Aifc_write__setnchannels__nchannels_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_aifc_Aifc_write__setnchannels__nchannels_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "aifc"
# dimension = "type"
# case = "Aifc_write__setnchannels__nchannels_as_int_wrong"
# subject = "aifc.Aifc_write.setnchannels(nchannels: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/aifc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: aifc.Aifc_write.setnchannels(nchannels: int); call it with the wrong type.

typeshed contract: nchannels is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from aifc import Aifc_write
obj = object.__new__(Aifc_write)
try:
    obj.setnchannels("not_an_int")  # nchannels: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/aifc/Aifc_write__setnframes__nframes_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_aifc_Aifc_write__setnframes__nframes_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "aifc"
# dimension = "type"
# case = "Aifc_write__setnframes__nframes_as_int_wrong"
# subject = "aifc.Aifc_write.setnframes(nframes: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/aifc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: aifc.Aifc_write.setnframes(nframes: int); call it with the wrong type.

typeshed contract: nframes is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from aifc import Aifc_write
obj = object.__new__(Aifc_write)
try:
    obj.setnframes("not_an_int")  # nframes: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/aifc/Aifc_write__setsampwidth__sampwidth_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_aifc_Aifc_write__setsampwidth__sampwidth_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "aifc"
# dimension = "type"
# case = "Aifc_write__setsampwidth__sampwidth_as_int_wrong"
# subject = "aifc.Aifc_write.setsampwidth(sampwidth: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/aifc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: aifc.Aifc_write.setsampwidth(sampwidth: int); call it with the wrong type.

typeshed contract: sampwidth is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from aifc import Aifc_write
obj = object.__new__(Aifc_write)
try:
    obj.setsampwidth("not_an_int")  # sampwidth: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
