use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/curses_textpad/Textbox__do_command__ch_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_curses_textpad_Textbox__do_command__ch_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "curses_textpad"
# dimension = "type"
# case = "Textbox__do_command__ch_as_typed_wrong"
# subject = "curses.textpad.Textbox.do_command(ch: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/curses/textpad.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: curses.textpad.Textbox.do_command(ch: typed); call it with the wrong type.

typeshed contract: ch is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from curses.textpad import Textbox
obj = object.__new__(Textbox)
try:
    obj.do_command(_W())  # ch: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/curses_textpad/Textbox__edit__validate_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_curses_textpad_Textbox__edit__validate_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "curses_textpad"
# dimension = "type"
# case = "Textbox__edit__validate_as_typed_wrong"
# subject = "curses.textpad.Textbox.edit(validate: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/curses/textpad.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: curses.textpad.Textbox.edit(validate: typed); call it with the wrong type.

typeshed contract: validate is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from curses.textpad import Textbox
obj = object.__new__(Textbox)
try:
    obj.edit(_W())  # validate: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/curses_textpad/Textbox__init__win_as_window_wrong.py`.
#[test]
fn test_gen_type_std_libs_curses_textpad_Textbox__init__win_as_window_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "curses_textpad"
# dimension = "type"
# case = "Textbox__init__win_as_window_wrong"
# subject = "curses.textpad.Textbox.__init__(win: window)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/curses/textpad.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: curses.textpad.Textbox.__init__(win: window); call it with the wrong type.

typeshed contract: win is window. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from curses.textpad import Textbox
try:
    Textbox(_W())  # win: window <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/curses_textpad/rectangle__win_as_window_wrong.py`.
#[test]
fn test_gen_type_std_libs_curses_textpad_rectangle__win_as_window_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "curses_textpad"
# dimension = "type"
# case = "rectangle__win_as_window_wrong"
# subject = "curses.textpad.rectangle(win: window)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/curses/textpad.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: curses.textpad.rectangle(win: window); call it with the wrong type.

typeshed contract: win is window. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from curses.textpad import rectangle
try:
    rectangle(_W(), 0, 0, 0, 0)  # win: window <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
