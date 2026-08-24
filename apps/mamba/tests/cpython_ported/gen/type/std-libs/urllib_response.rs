use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/urllib_response/addbase__init__fp_as_IO_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_response_addbase__init__fp_as_IO_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_response"
# dimension = "type"
# case = "addbase__init__fp_as_IO_wrong"
# subject = "urllib.response.addbase.__init__(fp: IO)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/response.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.response.addbase.__init__(fp: IO); call it with the wrong type.

typeshed contract: fp is IO. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.response import addbase
try:
    addbase(_W())  # fp: IO <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_response/addbase__write__s_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_response_addbase__write__s_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_response"
# dimension = "type"
# case = "addbase__write__s_as_ReadableBuffer_wrong"
# subject = "urllib.response.addbase.write(s: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/response.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.response.addbase.write(s: ReadableBuffer); call it with the wrong type.

typeshed contract: s is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.response import addbase
obj = object.__new__(addbase)
try:
    obj.write(_W())  # s: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_response/addbase__writelines__lines_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_response_addbase__writelines__lines_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_response"
# dimension = "type"
# case = "addbase__writelines__lines_as_Iterable_wrong"
# subject = "urllib.response.addbase.writelines(lines: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/response.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.response.addbase.writelines(lines: Iterable); call it with the wrong type.

typeshed contract: lines is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.response import addbase
obj = object.__new__(addbase)
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

/// Ported from `tests/cpython/type/std-libs/urllib_response/addinfo__init__fp_as_IO_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_response_addinfo__init__fp_as_IO_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_response"
# dimension = "type"
# case = "addinfo__init__fp_as_IO_wrong"
# subject = "urllib.response.addinfo.__init__(fp: IO)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/response.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.response.addinfo.__init__(fp: IO); call it with the wrong type.

typeshed contract: fp is IO. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.response import addinfo
try:
    addinfo(_W(), None)  # fp: IO <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_response/addinfourl__init__fp_as_IO_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_response_addinfourl__init__fp_as_IO_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_response"
# dimension = "type"
# case = "addinfourl__init__fp_as_IO_wrong"
# subject = "urllib.response.addinfourl.__init__(fp: IO)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/response.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.response.addinfourl.__init__(fp: IO); call it with the wrong type.

typeshed contract: fp is IO. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.response import addinfourl
try:
    addinfourl(_W(), None, "")  # fp: IO <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
