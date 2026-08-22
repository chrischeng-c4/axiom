use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/textwrap/TextWrapper__fill__text_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_textwrap_TextWrapper__fill__text_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "type"
# case = "TextWrapper__fill__text_as_str_wrong"
# subject = "textwrap.TextWrapper.fill(text: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/textwrap.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: textwrap.TextWrapper.fill(text: str); call it with the wrong type.

typeshed contract: text is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from textwrap import TextWrapper
obj = object.__new__(TextWrapper)
try:
    obj.fill(12345)  # text: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/textwrap/TextWrapper__init__width_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_textwrap_TextWrapper__init__width_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "type"
# case = "TextWrapper__init__width_as_int_wrong"
# subject = "textwrap.TextWrapper.__init__(width: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/textwrap.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: textwrap.TextWrapper.__init__(width: int); call it with the wrong type.

typeshed contract: width is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from textwrap import TextWrapper
try:
    TextWrapper("not_an_int")  # width: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/textwrap/TextWrapper__wrap__text_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_textwrap_TextWrapper__wrap__text_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "type"
# case = "TextWrapper__wrap__text_as_str_wrong"
# subject = "textwrap.TextWrapper.wrap(text: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/textwrap.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: textwrap.TextWrapper.wrap(text: str); call it with the wrong type.

typeshed contract: text is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from textwrap import TextWrapper
obj = object.__new__(TextWrapper)
try:
    obj.wrap(12345)  # text: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/textwrap/dedent__text_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_textwrap_dedent__text_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "type"
# case = "dedent__text_as_str_wrong"
# subject = "textwrap.dedent(text: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/textwrap.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: textwrap.dedent(text: str); call it with the wrong type.

typeshed contract: text is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from textwrap import dedent
try:
    dedent(12345)  # text: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/textwrap/fill__text_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_textwrap_fill__text_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "type"
# case = "fill__text_as_str_wrong"
# subject = "textwrap.fill(text: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/textwrap.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: textwrap.fill(text: str); call it with the wrong type.

typeshed contract: text is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from textwrap import fill
try:
    fill(12345)  # text: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/textwrap/indent__text_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_textwrap_indent__text_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "type"
# case = "indent__text_as_str_wrong"
# subject = "textwrap.indent(text: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/textwrap.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: textwrap.indent(text: str); call it with the wrong type.

typeshed contract: text is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from textwrap import indent
try:
    indent(12345, "")  # text: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/textwrap/shorten__text_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_textwrap_shorten__text_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "type"
# case = "shorten__text_as_str_wrong"
# subject = "textwrap.shorten(text: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/textwrap.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: textwrap.shorten(text: str); call it with the wrong type.

typeshed contract: text is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from textwrap import shorten
try:
    shorten(12345, 0)  # text: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/textwrap/wrap__text_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_textwrap_wrap__text_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "type"
# case = "wrap__text_as_str_wrong"
# subject = "textwrap.wrap(text: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/textwrap.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: textwrap.wrap(text: str); call it with the wrong type.

typeshed contract: text is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from textwrap import wrap
try:
    wrap(12345)  # text: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
