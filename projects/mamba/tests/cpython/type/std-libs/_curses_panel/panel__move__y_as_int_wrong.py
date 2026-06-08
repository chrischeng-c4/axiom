# /// script
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
# xfail = ""
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
