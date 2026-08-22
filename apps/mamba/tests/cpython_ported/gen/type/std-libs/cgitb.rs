use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/cgitb/Hook____call____etype_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_cgitb_Hook____call____etype_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cgitb"
# dimension = "type"
# case = "Hook____call____etype_as_typed_wrong"
# subject = "cgitb.Hook.__call__(etype: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cgitb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cgitb.Hook.__call__(etype: typed); call it with the wrong type.

typeshed contract: etype is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from cgitb import Hook
obj = object.__new__(Hook)
try:
    obj.__call__(_W(), None, None)  # etype: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cgitb/Hook__handle__info_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_cgitb_Hook__handle__info_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cgitb"
# dimension = "type"
# case = "Hook__handle__info_as_typed_wrong"
# subject = "cgitb.Hook.handle(info: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cgitb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cgitb.Hook.handle(info: typed); call it with the wrong type.

typeshed contract: info is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from cgitb import Hook
obj = object.__new__(Hook)
try:
    obj.handle(_W())  # info: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cgitb/Hook__init__display_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_cgitb_Hook__init__display_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cgitb"
# dimension = "type"
# case = "Hook__init__display_as_int_wrong"
# subject = "cgitb.Hook.__init__(display: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cgitb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cgitb.Hook.__init__(display: int); call it with the wrong type.

typeshed contract: display is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from cgitb import Hook
try:
    Hook("not_an_int")  # display: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cgitb/enable__display_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_cgitb_enable__display_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cgitb"
# dimension = "type"
# case = "enable__display_as_int_wrong"
# subject = "cgitb.enable(display: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cgitb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cgitb.enable(display: int); call it with the wrong type.

typeshed contract: display is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from cgitb import enable
try:
    enable("not_an_int")  # display: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cgitb/grey__text_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_cgitb_grey__text_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cgitb"
# dimension = "type"
# case = "grey__text_as_str_wrong"
# subject = "cgitb.grey(text: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cgitb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cgitb.grey(text: str); call it with the wrong type.

typeshed contract: text is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from cgitb import grey
try:
    grey(12345)  # text: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cgitb/handler__info_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_cgitb_handler__info_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cgitb"
# dimension = "type"
# case = "handler__info_as_typed_wrong"
# subject = "cgitb.handler(info: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cgitb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cgitb.handler(info: typed); call it with the wrong type.

typeshed contract: info is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from cgitb import handler
try:
    handler(_W())  # info: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cgitb/html__einfo_as_OptExcInfo_wrong.py`.
#[test]
fn test_gen_type_std_libs_cgitb_html__einfo_as_OptExcInfo_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cgitb"
# dimension = "type"
# case = "html__einfo_as_OptExcInfo_wrong"
# subject = "cgitb.html(einfo: OptExcInfo)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cgitb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cgitb.html(einfo: OptExcInfo); call it with the wrong type.

typeshed contract: einfo is OptExcInfo. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from cgitb import html
try:
    html(_W())  # einfo: OptExcInfo <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cgitb/lookup__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_cgitb_lookup__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cgitb"
# dimension = "type"
# case = "lookup__name_as_str_wrong"
# subject = "cgitb.lookup(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cgitb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cgitb.lookup(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from cgitb import lookup
try:
    lookup(12345, None, None)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cgitb/scanvars__reader_as_Callable_wrong.py`.
#[test]
fn test_gen_type_std_libs_cgitb_scanvars__reader_as_Callable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cgitb"
# dimension = "type"
# case = "scanvars__reader_as_Callable_wrong"
# subject = "cgitb.scanvars(reader: Callable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cgitb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cgitb.scanvars(reader: Callable); call it with the wrong type.

typeshed contract: reader is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from cgitb import scanvars
try:
    scanvars(_W(), None, None)  # reader: Callable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cgitb/small__text_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_cgitb_small__text_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cgitb"
# dimension = "type"
# case = "small__text_as_str_wrong"
# subject = "cgitb.small(text: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cgitb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cgitb.small(text: str); call it with the wrong type.

typeshed contract: text is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from cgitb import small
try:
    small(12345)  # text: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cgitb/strong__text_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_cgitb_strong__text_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cgitb"
# dimension = "type"
# case = "strong__text_as_str_wrong"
# subject = "cgitb.strong(text: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cgitb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cgitb.strong(text: str); call it with the wrong type.

typeshed contract: text is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from cgitb import strong
try:
    strong(12345)  # text: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cgitb/text__einfo_as_OptExcInfo_wrong.py`.
#[test]
fn test_gen_type_std_libs_cgitb_text__einfo_as_OptExcInfo_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cgitb"
# dimension = "type"
# case = "text__einfo_as_OptExcInfo_wrong"
# subject = "cgitb.text(einfo: OptExcInfo)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cgitb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cgitb.text(einfo: OptExcInfo); call it with the wrong type.

typeshed contract: einfo is OptExcInfo. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from cgitb import text
try:
    text(_W())  # einfo: OptExcInfo <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
