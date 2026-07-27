use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/sunau/Au_read__init__f_as__File_wrong.py`.
#[test]
fn test_gen_type_std_libs_sunau_Au_read__init__f_as__File_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sunau"
# dimension = "type"
# case = "Au_read__init__f_as__File_wrong"
# subject = "sunau.Au_read.__init__(f: _File)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sunau.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: sunau.Au_read.__init__(f: _File); call it with the wrong type.

typeshed contract: f is _File. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from sunau import Au_read
try:
    Au_read(_W())  # f: _File <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/sunau/Au_read__readframes__nframes_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_sunau_Au_read__readframes__nframes_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sunau"
# dimension = "type"
# case = "Au_read__readframes__nframes_as_int_wrong"
# subject = "sunau.Au_read.readframes(nframes: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sunau.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: sunau.Au_read.readframes(nframes: int); call it with the wrong type.

typeshed contract: nframes is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from sunau import Au_read
obj = object.__new__(Au_read)
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

/// Ported from `tests/cpython/type/std-libs/sunau/Au_read__setpos__pos_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_sunau_Au_read__setpos__pos_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sunau"
# dimension = "type"
# case = "Au_read__setpos__pos_as_int_wrong"
# subject = "sunau.Au_read.setpos(pos: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sunau.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: sunau.Au_read.setpos(pos: int); call it with the wrong type.

typeshed contract: pos is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from sunau import Au_read
obj = object.__new__(Au_read)
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

/// Ported from `tests/cpython/type/std-libs/sunau/Au_write__init__f_as__File_wrong.py`.
#[test]
fn test_gen_type_std_libs_sunau_Au_write__init__f_as__File_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sunau"
# dimension = "type"
# case = "Au_write__init__f_as__File_wrong"
# subject = "sunau.Au_write.__init__(f: _File)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sunau.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: sunau.Au_write.__init__(f: _File); call it with the wrong type.

typeshed contract: f is _File. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from sunau import Au_write
try:
    Au_write(_W())  # f: _File <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/sunau/Au_write__setcomptype__type_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_sunau_Au_write__setcomptype__type_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sunau"
# dimension = "type"
# case = "Au_write__setcomptype__type_as_str_wrong"
# subject = "sunau.Au_write.setcomptype(type: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sunau.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: sunau.Au_write.setcomptype(type: str); call it with the wrong type.

typeshed contract: type is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from sunau import Au_write
obj = object.__new__(Au_write)
try:
    obj.setcomptype(12345, "")  # type: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/sunau/Au_write__setframerate__framerate_as_float_wrong.py`.
#[test]
fn test_gen_type_std_libs_sunau_Au_write__setframerate__framerate_as_float_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sunau"
# dimension = "type"
# case = "Au_write__setframerate__framerate_as_float_wrong"
# subject = "sunau.Au_write.setframerate(framerate: float)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sunau.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: sunau.Au_write.setframerate(framerate: float); call it with the wrong type.

typeshed contract: framerate is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from sunau import Au_write
obj = object.__new__(Au_write)
try:
    obj.setframerate("not_a_float")  # framerate: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/sunau/Au_write__setnchannels__nchannels_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_sunau_Au_write__setnchannels__nchannels_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sunau"
# dimension = "type"
# case = "Au_write__setnchannels__nchannels_as_int_wrong"
# subject = "sunau.Au_write.setnchannels(nchannels: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sunau.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: sunau.Au_write.setnchannels(nchannels: int); call it with the wrong type.

typeshed contract: nchannels is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from sunau import Au_write
obj = object.__new__(Au_write)
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

/// Ported from `tests/cpython/type/std-libs/sunau/Au_write__setnframes__nframes_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_sunau_Au_write__setnframes__nframes_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sunau"
# dimension = "type"
# case = "Au_write__setnframes__nframes_as_int_wrong"
# subject = "sunau.Au_write.setnframes(nframes: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sunau.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: sunau.Au_write.setnframes(nframes: int); call it with the wrong type.

typeshed contract: nframes is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from sunau import Au_write
obj = object.__new__(Au_write)
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

/// Ported from `tests/cpython/type/std-libs/sunau/Au_write__setparams__params_as__sunau_params_wrong.py`.
#[test]
fn test_gen_type_std_libs_sunau_Au_write__setparams__params_as__sunau_params_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sunau"
# dimension = "type"
# case = "Au_write__setparams__params_as__sunau_params_wrong"
# subject = "sunau.Au_write.setparams(params: _sunau_params)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sunau.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: sunau.Au_write.setparams(params: _sunau_params); call it with the wrong type.

typeshed contract: params is _sunau_params. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from sunau import Au_write
obj = object.__new__(Au_write)
try:
    obj.setparams(_W())  # params: _sunau_params <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/sunau/Au_write__setsampwidth__sampwidth_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_sunau_Au_write__setsampwidth__sampwidth_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sunau"
# dimension = "type"
# case = "Au_write__setsampwidth__sampwidth_as_int_wrong"
# subject = "sunau.Au_write.setsampwidth(sampwidth: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sunau.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: sunau.Au_write.setsampwidth(sampwidth: int); call it with the wrong type.

typeshed contract: sampwidth is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from sunau import Au_write
obj = object.__new__(Au_write)
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

/// Ported from `tests/cpython/type/std-libs/sunau/open__f_as__File_wrong.py`.
#[test]
fn test_gen_type_std_libs_sunau_open__f_as__File_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sunau"
# dimension = "type"
# case = "open__f_as__File_wrong"
# subject = "sunau.open(f: _File)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sunau.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: sunau.open(f: _File); call it with the wrong type.

typeshed contract: f is _File. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from sunau import open
try:
    open(_W(), None)  # f: _File <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
