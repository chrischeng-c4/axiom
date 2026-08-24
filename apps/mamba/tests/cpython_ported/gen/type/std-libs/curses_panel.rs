use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/_curses_panel/new_panel__win_as_window_wrong.py`.
#[test]
fn test_gen_type_std_libs__curses_panel_new_panel__win_as_window_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_curses_panel"
# dimension = "type"
# case = "new_panel__win_as_window_wrong"
# subject = "_curses_panel.new_panel(win: window)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_curses_panel.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _curses_panel.new_panel(win: window); call it with the wrong type.

typeshed contract: win is window. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _curses_panel import new_panel
try:
    new_panel(_W())  # win: window <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_curses_panel/panel__move__y_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__curses_panel_panel__move__y_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_curses_panel"
# dimension = "type"
# case = "panel__move__y_as_int_wrong"
# subject = "_curses_panel.panel.move(y: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_curses_panel.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _curses_panel.panel.move(y: int); call it with the wrong type.

typeshed contract: y is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _curses_panel import panel
obj = object.__new__(panel)
try:
    obj.move("not_an_int", 0)  # y: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_curses_panel/panel__replace__win_as_window_wrong.py`.
#[test]
fn test_gen_type_std_libs__curses_panel_panel__replace__win_as_window_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_curses_panel"
# dimension = "type"
# case = "panel__replace__win_as_window_wrong"
# subject = "_curses_panel.panel.replace(win: window)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_curses_panel.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _curses_panel.panel.replace(win: window); call it with the wrong type.

typeshed contract: win is window. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _curses_panel import panel
obj = object.__new__(panel)
try:
    obj.replace(_W())  # win: window <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
