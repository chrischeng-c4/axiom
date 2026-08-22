use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/tkinter_simpledialog/Dialog__body__master_as_Frame_wrong.py`.
#[test]
fn test_gen_type_std_libs_tkinter_simpledialog_Dialog__body__master_as_Frame_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tkinter_simpledialog"
# dimension = "type"
# case = "Dialog__body__master_as_Frame_wrong"
# subject = "tkinter.simpledialog.Dialog.body(master: Frame)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tkinter/simpledialog.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tkinter.simpledialog.Dialog.body(master: Frame); call it with the wrong type.

typeshed contract: master is Frame. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tkinter.simpledialog import Dialog
obj = object.__new__(Dialog)
try:
    obj.body(_W())  # master: Frame <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tkinter_simpledialog/Dialog__init__parent_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_tkinter_simpledialog_Dialog__init__parent_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tkinter_simpledialog"
# dimension = "type"
# case = "Dialog__init__parent_as_typed_wrong"
# subject = "tkinter.simpledialog.Dialog.__init__(parent: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tkinter/simpledialog.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tkinter.simpledialog.Dialog.__init__(parent: typed); call it with the wrong type.

typeshed contract: parent is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tkinter.simpledialog import Dialog
try:
    Dialog(_W())  # parent: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tkinter_simpledialog/SimpleDialog__done__num_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_tkinter_simpledialog_SimpleDialog__done__num_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tkinter_simpledialog"
# dimension = "type"
# case = "SimpleDialog__done__num_as_int_wrong"
# subject = "tkinter.simpledialog.SimpleDialog.done(num: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tkinter/simpledialog.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tkinter.simpledialog.SimpleDialog.done(num: int); call it with the wrong type.

typeshed contract: num is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from tkinter.simpledialog import SimpleDialog
obj = object.__new__(SimpleDialog)
try:
    obj.done("not_an_int")  # num: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tkinter_simpledialog/SimpleDialog__init__master_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_tkinter_simpledialog_SimpleDialog__init__master_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tkinter_simpledialog"
# dimension = "type"
# case = "SimpleDialog__init__master_as_typed_wrong"
# subject = "tkinter.simpledialog.SimpleDialog.__init__(master: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tkinter/simpledialog.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tkinter.simpledialog.SimpleDialog.__init__(master: typed); call it with the wrong type.

typeshed contract: master is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tkinter.simpledialog import SimpleDialog
try:
    SimpleDialog(_W())  # master: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tkinter_simpledialog/askfloat__title_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_tkinter_simpledialog_askfloat__title_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tkinter_simpledialog"
# dimension = "type"
# case = "askfloat__title_as_typed_wrong"
# subject = "tkinter.simpledialog.askfloat(title: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tkinter/simpledialog.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tkinter.simpledialog.askfloat(title: typed); call it with the wrong type.

typeshed contract: title is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tkinter.simpledialog import askfloat
try:
    askfloat(_W(), "")  # title: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tkinter_simpledialog/askinteger__title_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_tkinter_simpledialog_askinteger__title_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tkinter_simpledialog"
# dimension = "type"
# case = "askinteger__title_as_typed_wrong"
# subject = "tkinter.simpledialog.askinteger(title: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tkinter/simpledialog.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tkinter.simpledialog.askinteger(title: typed); call it with the wrong type.

typeshed contract: title is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tkinter.simpledialog import askinteger
try:
    askinteger(_W(), "")  # title: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tkinter_simpledialog/askstring__title_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_tkinter_simpledialog_askstring__title_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tkinter_simpledialog"
# dimension = "type"
# case = "askstring__title_as_typed_wrong"
# subject = "tkinter.simpledialog.askstring(title: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tkinter/simpledialog.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tkinter.simpledialog.askstring(title: typed); call it with the wrong type.

typeshed contract: title is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tkinter.simpledialog import askstring
try:
    askstring(_W(), "")  # title: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
