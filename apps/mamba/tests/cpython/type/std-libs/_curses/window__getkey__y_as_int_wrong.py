# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_curses"
# dimension = "type"
# case = "window__getkey__y_as_int_wrong"
# subject = "_curses.window.getkey(y: int)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed y"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_curses.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed y
# mamba-strict-type: TypeError
"""Type wall: _curses.window.getkey(y: int); call it with the wrong type.

typeshed contract: y is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _curses import window
obj = object.__new__(window)
try:
    obj.getkey("not_an_int", 0)  # y: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
