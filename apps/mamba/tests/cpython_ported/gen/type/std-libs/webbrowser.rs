use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/webbrowser/BaseBrowser__init__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_webbrowser_BaseBrowser__init__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "webbrowser"
# dimension = "type"
# case = "BaseBrowser__init__name_as_str_wrong"
# subject = "webbrowser.BaseBrowser.__init__(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/webbrowser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: webbrowser.BaseBrowser.__init__(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from webbrowser import BaseBrowser
try:
    BaseBrowser(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/webbrowser/BaseBrowser__open__url_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_webbrowser_BaseBrowser__open__url_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "webbrowser"
# dimension = "type"
# case = "BaseBrowser__open__url_as_str_wrong"
# subject = "webbrowser.BaseBrowser.open(url: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/webbrowser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: webbrowser.BaseBrowser.open(url: str); call it with the wrong type.

typeshed contract: url is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from webbrowser import BaseBrowser
obj = object.__new__(BaseBrowser)
try:
    obj.open(12345)  # url: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/webbrowser/BaseBrowser__open_new__url_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_webbrowser_BaseBrowser__open_new__url_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "webbrowser"
# dimension = "type"
# case = "BaseBrowser__open_new__url_as_str_wrong"
# subject = "webbrowser.BaseBrowser.open_new(url: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/webbrowser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: webbrowser.BaseBrowser.open_new(url: str); call it with the wrong type.

typeshed contract: url is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from webbrowser import BaseBrowser
obj = object.__new__(BaseBrowser)
try:
    obj.open_new(12345)  # url: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/webbrowser/BaseBrowser__open_new_tab__url_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_webbrowser_BaseBrowser__open_new_tab__url_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "webbrowser"
# dimension = "type"
# case = "BaseBrowser__open_new_tab__url_as_str_wrong"
# subject = "webbrowser.BaseBrowser.open_new_tab(url: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/webbrowser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: webbrowser.BaseBrowser.open_new_tab(url: str); call it with the wrong type.

typeshed contract: url is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from webbrowser import BaseBrowser
obj = object.__new__(BaseBrowser)
try:
    obj.open_new_tab(12345)  # url: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/webbrowser/GenericBrowser__init__name_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_webbrowser_GenericBrowser__init__name_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "webbrowser"
# dimension = "type"
# case = "GenericBrowser__init__name_as_typed_wrong"
# subject = "webbrowser.GenericBrowser.__init__(name: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/webbrowser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: webbrowser.GenericBrowser.__init__(name: typed); call it with the wrong type.

typeshed contract: name is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from webbrowser import GenericBrowser
try:
    GenericBrowser(_W())  # name: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/webbrowser/GenericBrowser__open__url_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_webbrowser_GenericBrowser__open__url_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "webbrowser"
# dimension = "type"
# case = "GenericBrowser__open__url_as_str_wrong"
# subject = "webbrowser.GenericBrowser.open(url: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/webbrowser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: webbrowser.GenericBrowser.open(url: str); call it with the wrong type.

typeshed contract: url is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from webbrowser import GenericBrowser
obj = object.__new__(GenericBrowser)
try:
    obj.open(12345)  # url: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/webbrowser/Grail__open__url_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_webbrowser_Grail__open__url_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "webbrowser"
# dimension = "type"
# case = "Grail__open__url_as_str_wrong"
# subject = "webbrowser.Grail.open(url: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/webbrowser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: webbrowser.Grail.open(url: str); call it with the wrong type.

typeshed contract: url is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from webbrowser import Grail
obj = object.__new__(Grail)
try:
    obj.open(12345)  # url: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/webbrowser/Konqueror__open__url_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_webbrowser_Konqueror__open__url_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "webbrowser"
# dimension = "type"
# case = "Konqueror__open__url_as_str_wrong"
# subject = "webbrowser.Konqueror.open(url: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/webbrowser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: webbrowser.Konqueror.open(url: str); call it with the wrong type.

typeshed contract: url is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from webbrowser import Konqueror
obj = object.__new__(Konqueror)
try:
    obj.open(12345)  # url: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/webbrowser/MacOSXOSAScript__init__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_webbrowser_MacOSXOSAScript__init__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "webbrowser"
# dimension = "type"
# case = "MacOSXOSAScript__init__name_as_str_wrong"
# subject = "webbrowser.MacOSXOSAScript.__init__(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/webbrowser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: webbrowser.MacOSXOSAScript.__init__(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from webbrowser import MacOSXOSAScript
try:
    MacOSXOSAScript(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/webbrowser/MacOSXOSAScript__open__url_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_webbrowser_MacOSXOSAScript__open__url_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "webbrowser"
# dimension = "type"
# case = "MacOSXOSAScript__open__url_as_str_wrong"
# subject = "webbrowser.MacOSXOSAScript.open(url: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/webbrowser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: webbrowser.MacOSXOSAScript.open(url: str); call it with the wrong type.

typeshed contract: url is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from webbrowser import MacOSXOSAScript
obj = object.__new__(MacOSXOSAScript)
try:
    obj.open(12345)  # url: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/webbrowser/MacOSX__init__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_webbrowser_MacOSX__init__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "webbrowser"
# dimension = "type"
# case = "MacOSX__init__name_as_str_wrong"
# subject = "webbrowser.MacOSX.__init__(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/webbrowser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: webbrowser.MacOSX.__init__(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from webbrowser import MacOSX
try:
    MacOSX(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/webbrowser/MacOSX__open__url_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_webbrowser_MacOSX__open__url_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "webbrowser"
# dimension = "type"
# case = "MacOSX__open__url_as_str_wrong"
# subject = "webbrowser.MacOSX.open(url: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/webbrowser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: webbrowser.MacOSX.open(url: str); call it with the wrong type.

typeshed contract: url is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from webbrowser import MacOSX
obj = object.__new__(MacOSX)
try:
    obj.open(12345)  # url: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/webbrowser/UnixBrowser__open__url_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_webbrowser_UnixBrowser__open__url_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "webbrowser"
# dimension = "type"
# case = "UnixBrowser__open__url_as_str_wrong"
# subject = "webbrowser.UnixBrowser.open(url: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/webbrowser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: webbrowser.UnixBrowser.open(url: str); call it with the wrong type.

typeshed contract: url is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from webbrowser import UnixBrowser
obj = object.__new__(UnixBrowser)
try:
    obj.open(12345)  # url: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/webbrowser/WindowsDefault__open__url_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_webbrowser_WindowsDefault__open__url_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "webbrowser"
# dimension = "type"
# case = "WindowsDefault__open__url_as_str_wrong"
# subject = "webbrowser.WindowsDefault.open(url: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/webbrowser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: webbrowser.WindowsDefault.open(url: str); call it with the wrong type.

typeshed contract: url is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from webbrowser import WindowsDefault
obj = object.__new__(WindowsDefault)
try:
    obj.open(12345)  # url: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/webbrowser/get__using_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_webbrowser_get__using_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "webbrowser"
# dimension = "type"
# case = "get__using_as_typed_wrong"
# subject = "webbrowser.get(using: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/webbrowser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: webbrowser.get(using: typed); call it with the wrong type.

typeshed contract: using is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from webbrowser import get
try:
    get(_W())  # using: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/webbrowser/open__url_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_webbrowser_open__url_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "webbrowser"
# dimension = "type"
# case = "open__url_as_str_wrong"
# subject = "webbrowser.open(url: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/webbrowser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: webbrowser.open(url: str); call it with the wrong type.

typeshed contract: url is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from webbrowser import open
try:
    open(12345)  # url: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/webbrowser/open_new__url_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_webbrowser_open_new__url_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "webbrowser"
# dimension = "type"
# case = "open_new__url_as_str_wrong"
# subject = "webbrowser.open_new(url: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/webbrowser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: webbrowser.open_new(url: str); call it with the wrong type.

typeshed contract: url is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from webbrowser import open_new
try:
    open_new(12345)  # url: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/webbrowser/open_new_tab__url_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_webbrowser_open_new_tab__url_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "webbrowser"
# dimension = "type"
# case = "open_new_tab__url_as_str_wrong"
# subject = "webbrowser.open_new_tab(url: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/webbrowser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: webbrowser.open_new_tab(url: str); call it with the wrong type.

typeshed contract: url is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from webbrowser import open_new_tab
try:
    open_new_tab(12345)  # url: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/webbrowser/register__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_webbrowser_register__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "webbrowser"
# dimension = "type"
# case = "register__name_as_str_wrong"
# subject = "webbrowser.register(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/webbrowser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: webbrowser.register(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from webbrowser import register
try:
    register(12345, None)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
