use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/lib2to3_pgen2_driver/Driver__init__grammar_as_Grammar_wrong.py`.
#[test]
fn test_gen_type_std_libs_lib2to3_pgen2_driver_Driver__init__grammar_as_Grammar_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_pgen2_driver"
# dimension = "type"
# case = "Driver__init__grammar_as_Grammar_wrong"
# subject = "lib2to3.pgen2.driver.Driver.__init__(grammar: Grammar)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/pgen2/driver.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.pgen2.driver.Driver.__init__(grammar: Grammar); call it with the wrong type.

typeshed contract: grammar is Grammar. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lib2to3.pgen2.driver import Driver
try:
    Driver(_W())  # grammar: Grammar <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lib2to3_pgen2_driver/Driver__parse_file__filename_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_lib2to3_pgen2_driver_Driver__parse_file__filename_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_pgen2_driver"
# dimension = "type"
# case = "Driver__parse_file__filename_as_StrPath_wrong"
# subject = "lib2to3.pgen2.driver.Driver.parse_file(filename: StrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/pgen2/driver.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.pgen2.driver.Driver.parse_file(filename: StrPath); call it with the wrong type.

typeshed contract: filename is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lib2to3.pgen2.driver import Driver
obj = object.__new__(Driver)
try:
    obj.parse_file(_W())  # filename: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lib2to3_pgen2_driver/Driver__parse_stream__stream_as_IO_wrong.py`.
#[test]
fn test_gen_type_std_libs_lib2to3_pgen2_driver_Driver__parse_stream__stream_as_IO_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_pgen2_driver"
# dimension = "type"
# case = "Driver__parse_stream__stream_as_IO_wrong"
# subject = "lib2to3.pgen2.driver.Driver.parse_stream(stream: IO)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/pgen2/driver.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.pgen2.driver.Driver.parse_stream(stream: IO); call it with the wrong type.

typeshed contract: stream is IO. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lib2to3.pgen2.driver import Driver
obj = object.__new__(Driver)
try:
    obj.parse_stream(_W())  # stream: IO <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lib2to3_pgen2_driver/Driver__parse_stream_raw__stream_as_IO_wrong.py`.
#[test]
fn test_gen_type_std_libs_lib2to3_pgen2_driver_Driver__parse_stream_raw__stream_as_IO_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_pgen2_driver"
# dimension = "type"
# case = "Driver__parse_stream_raw__stream_as_IO_wrong"
# subject = "lib2to3.pgen2.driver.Driver.parse_stream_raw(stream: IO)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/pgen2/driver.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.pgen2.driver.Driver.parse_stream_raw(stream: IO); call it with the wrong type.

typeshed contract: stream is IO. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lib2to3.pgen2.driver import Driver
obj = object.__new__(Driver)
try:
    obj.parse_stream_raw(_W())  # stream: IO <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lib2to3_pgen2_driver/Driver__parse_string__text_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_lib2to3_pgen2_driver_Driver__parse_string__text_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_pgen2_driver"
# dimension = "type"
# case = "Driver__parse_string__text_as_str_wrong"
# subject = "lib2to3.pgen2.driver.Driver.parse_string(text: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/pgen2/driver.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.pgen2.driver.Driver.parse_string(text: str); call it with the wrong type.

typeshed contract: text is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from lib2to3.pgen2.driver import Driver
obj = object.__new__(Driver)
try:
    obj.parse_string(12345)  # text: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lib2to3_pgen2_driver/Driver__parse_tokens__tokens_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_lib2to3_pgen2_driver_Driver__parse_tokens__tokens_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_pgen2_driver"
# dimension = "type"
# case = "Driver__parse_tokens__tokens_as_Iterable_wrong"
# subject = "lib2to3.pgen2.driver.Driver.parse_tokens(tokens: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/pgen2/driver.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.pgen2.driver.Driver.parse_tokens(tokens: Iterable); call it with the wrong type.

typeshed contract: tokens is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lib2to3.pgen2.driver import Driver
obj = object.__new__(Driver)
try:
    obj.parse_tokens(_W())  # tokens: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lib2to3_pgen2_driver/load_grammar__gt_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_lib2to3_pgen2_driver_load_grammar__gt_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_pgen2_driver"
# dimension = "type"
# case = "load_grammar__gt_as_str_wrong"
# subject = "lib2to3.pgen2.driver.load_grammar(gt: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/pgen2/driver.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.pgen2.driver.load_grammar(gt: str); call it with the wrong type.

typeshed contract: gt is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from lib2to3.pgen2.driver import load_grammar
try:
    load_grammar(12345)  # gt: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
