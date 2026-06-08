# /// script
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
# xfail = ""
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
