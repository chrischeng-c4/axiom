# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tkinter"
# dimension = "type"
# case = "Canvas__create_line__xy_pair_0_as_tuple_wrong"
# subject = "tkinter.Canvas.create_line(xy_pair_0: tuple)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed xy_pair_0"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tkinter.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed xy_pair_0
# mamba-strict-type: TypeError
"""Type wall: tkinter.Canvas.create_line(xy_pair_0: tuple); call it with the wrong type.

typeshed contract: xy_pair_0 is tuple. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from tkinter import Canvas
obj = object.__new__(Canvas)
try:
    obj.create_line(12345, None)  # xy_pair_0: tuple <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
