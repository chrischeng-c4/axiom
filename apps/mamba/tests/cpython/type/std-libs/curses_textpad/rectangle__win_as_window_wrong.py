# /// script
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
