use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/tkinter_filedialog/FileDialog__cancel_command__event_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_tkinter_filedialog_FileDialog__cancel_command__event_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tkinter_filedialog"
# dimension = "type"
# case = "FileDialog__cancel_command__event_as_typed_wrong"
# subject = "tkinter.filedialog.FileDialog.cancel_command(event: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tkinter/filedialog.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tkinter.filedialog.FileDialog.cancel_command(event: typed); call it with the wrong type.

typeshed contract: event is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tkinter.filedialog import FileDialog
obj = object.__new__(FileDialog)
try:
    obj.cancel_command(_W())  # event: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tkinter_filedialog/FileDialog__dirs_double_event__event_as_Event_wrong.py`.
#[test]
fn test_gen_type_std_libs_tkinter_filedialog_FileDialog__dirs_double_event__event_as_Event_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tkinter_filedialog"
# dimension = "type"
# case = "FileDialog__dirs_double_event__event_as_Event_wrong"
# subject = "tkinter.filedialog.FileDialog.dirs_double_event(event: Event)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tkinter/filedialog.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tkinter.filedialog.FileDialog.dirs_double_event(event: Event); call it with the wrong type.

typeshed contract: event is Event. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tkinter.filedialog import FileDialog
obj = object.__new__(FileDialog)
try:
    obj.dirs_double_event(_W())  # event: Event <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tkinter_filedialog/FileDialog__dirs_select_event__event_as_Event_wrong.py`.
#[test]
fn test_gen_type_std_libs_tkinter_filedialog_FileDialog__dirs_select_event__event_as_Event_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tkinter_filedialog"
# dimension = "type"
# case = "FileDialog__dirs_select_event__event_as_Event_wrong"
# subject = "tkinter.filedialog.FileDialog.dirs_select_event(event: Event)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tkinter/filedialog.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tkinter.filedialog.FileDialog.dirs_select_event(event: Event); call it with the wrong type.

typeshed contract: event is Event. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tkinter.filedialog import FileDialog
obj = object.__new__(FileDialog)
try:
    obj.dirs_select_event(_W())  # event: Event <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tkinter_filedialog/FileDialog__files_double_event__event_as_Event_wrong.py`.
#[test]
fn test_gen_type_std_libs_tkinter_filedialog_FileDialog__files_double_event__event_as_Event_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tkinter_filedialog"
# dimension = "type"
# case = "FileDialog__files_double_event__event_as_Event_wrong"
# subject = "tkinter.filedialog.FileDialog.files_double_event(event: Event)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tkinter/filedialog.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tkinter.filedialog.FileDialog.files_double_event(event: Event); call it with the wrong type.

typeshed contract: event is Event. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tkinter.filedialog import FileDialog
obj = object.__new__(FileDialog)
try:
    obj.files_double_event(_W())  # event: Event <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tkinter_filedialog/FileDialog__files_select_event__event_as_Event_wrong.py`.
#[test]
fn test_gen_type_std_libs_tkinter_filedialog_FileDialog__files_select_event__event_as_Event_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tkinter_filedialog"
# dimension = "type"
# case = "FileDialog__files_select_event__event_as_Event_wrong"
# subject = "tkinter.filedialog.FileDialog.files_select_event(event: Event)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tkinter/filedialog.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tkinter.filedialog.FileDialog.files_select_event(event: Event); call it with the wrong type.

typeshed contract: event is Event. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tkinter.filedialog import FileDialog
obj = object.__new__(FileDialog)
try:
    obj.files_select_event(_W())  # event: Event <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tkinter_filedialog/FileDialog__filter_command__event_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_tkinter_filedialog_FileDialog__filter_command__event_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tkinter_filedialog"
# dimension = "type"
# case = "FileDialog__filter_command__event_as_typed_wrong"
# subject = "tkinter.filedialog.FileDialog.filter_command(event: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tkinter/filedialog.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tkinter.filedialog.FileDialog.filter_command(event: typed); call it with the wrong type.

typeshed contract: event is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tkinter.filedialog import FileDialog
obj = object.__new__(FileDialog)
try:
    obj.filter_command(_W())  # event: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tkinter_filedialog/FileDialog__go__dir_or_file_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_tkinter_filedialog_FileDialog__go__dir_or_file_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tkinter_filedialog"
# dimension = "type"
# case = "FileDialog__go__dir_or_file_as_StrPath_wrong"
# subject = "tkinter.filedialog.FileDialog.go(dir_or_file: StrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tkinter/filedialog.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tkinter.filedialog.FileDialog.go(dir_or_file: StrPath); call it with the wrong type.

typeshed contract: dir_or_file is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tkinter.filedialog import FileDialog
obj = object.__new__(FileDialog)
try:
    obj.go(_W())  # dir_or_file: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tkinter_filedialog/FileDialog__init__master_as_Misc_wrong.py`.
#[test]
fn test_gen_type_std_libs_tkinter_filedialog_FileDialog__init__master_as_Misc_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tkinter_filedialog"
# dimension = "type"
# case = "FileDialog__init__master_as_Misc_wrong"
# subject = "tkinter.filedialog.FileDialog.__init__(master: Misc)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tkinter/filedialog.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tkinter.filedialog.FileDialog.__init__(master: Misc); call it with the wrong type.

typeshed contract: master is Misc. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tkinter.filedialog import FileDialog
try:
    FileDialog(_W())  # master: Misc <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tkinter_filedialog/FileDialog__ok_event__event_as_Event_wrong.py`.
#[test]
fn test_gen_type_std_libs_tkinter_filedialog_FileDialog__ok_event__event_as_Event_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tkinter_filedialog"
# dimension = "type"
# case = "FileDialog__ok_event__event_as_Event_wrong"
# subject = "tkinter.filedialog.FileDialog.ok_event(event: Event)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tkinter/filedialog.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tkinter.filedialog.FileDialog.ok_event(event: Event); call it with the wrong type.

typeshed contract: event is Event. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tkinter.filedialog import FileDialog
obj = object.__new__(FileDialog)
try:
    obj.ok_event(_W())  # event: Event <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tkinter_filedialog/FileDialog__quit__how_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_tkinter_filedialog_FileDialog__quit__how_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tkinter_filedialog"
# dimension = "type"
# case = "FileDialog__quit__how_as_typed_wrong"
# subject = "tkinter.filedialog.FileDialog.quit(how: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tkinter/filedialog.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tkinter.filedialog.FileDialog.quit(how: typed); call it with the wrong type.

typeshed contract: how is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tkinter.filedialog import FileDialog
obj = object.__new__(FileDialog)
try:
    obj.quit(_W())  # how: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tkinter_filedialog/FileDialog__set_filter__dir_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_tkinter_filedialog_FileDialog__set_filter__dir_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tkinter_filedialog"
# dimension = "type"
# case = "FileDialog__set_filter__dir_as_StrPath_wrong"
# subject = "tkinter.filedialog.FileDialog.set_filter(dir: StrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tkinter/filedialog.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tkinter.filedialog.FileDialog.set_filter(dir: StrPath); call it with the wrong type.

typeshed contract: dir is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tkinter.filedialog import FileDialog
obj = object.__new__(FileDialog)
try:
    obj.set_filter(_W(), None)  # dir: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tkinter_filedialog/FileDialog__set_selection__file_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_tkinter_filedialog_FileDialog__set_selection__file_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tkinter_filedialog"
# dimension = "type"
# case = "FileDialog__set_selection__file_as_StrPath_wrong"
# subject = "tkinter.filedialog.FileDialog.set_selection(file: StrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tkinter/filedialog.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tkinter.filedialog.FileDialog.set_selection(file: StrPath); call it with the wrong type.

typeshed contract: file is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tkinter.filedialog import FileDialog
obj = object.__new__(FileDialog)
try:
    obj.set_selection(_W())  # file: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tkinter_filedialog/askopenfile__mode_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_tkinter_filedialog_askopenfile__mode_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tkinter_filedialog"
# dimension = "type"
# case = "askopenfile__mode_as_str_wrong"
# subject = "tkinter.filedialog.askopenfile(mode: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tkinter/filedialog.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tkinter.filedialog.askopenfile(mode: str); call it with the wrong type.

typeshed contract: mode is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from tkinter.filedialog import askopenfile
try:
    askopenfile(12345)  # mode: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tkinter_filedialog/askopenfiles__mode_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_tkinter_filedialog_askopenfiles__mode_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tkinter_filedialog"
# dimension = "type"
# case = "askopenfiles__mode_as_str_wrong"
# subject = "tkinter.filedialog.askopenfiles(mode: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tkinter/filedialog.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tkinter.filedialog.askopenfiles(mode: str); call it with the wrong type.

typeshed contract: mode is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from tkinter.filedialog import askopenfiles
try:
    askopenfiles(12345)  # mode: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tkinter_filedialog/asksaveasfile__mode_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_tkinter_filedialog_asksaveasfile__mode_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tkinter_filedialog"
# dimension = "type"
# case = "asksaveasfile__mode_as_str_wrong"
# subject = "tkinter.filedialog.asksaveasfile(mode: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tkinter/filedialog.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tkinter.filedialog.asksaveasfile(mode: str); call it with the wrong type.

typeshed contract: mode is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from tkinter.filedialog import asksaveasfile
try:
    asksaveasfile(12345)  # mode: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
