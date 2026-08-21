# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_curses"
# dimension = "type"
# case = "window__overlay__destwin_as_window_wrong"
# subject = "_curses.window.overlay(destwin: window)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed destwin"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_curses.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed destwin
# mamba-strict-type: TypeError
"""Type wall: _curses.window.overlay(destwin: window); call it with the wrong type.

typeshed contract: destwin is window. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _curses import window
obj = object.__new__(window)
try:
    obj.overlay(_W())  # destwin: window <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
