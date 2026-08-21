# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tkinter"
# dimension = "type"
# case = "Canvas__create_line__coords_as_typed_wrong"
# subject = "tkinter.Canvas.create_line(coords: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed coords"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tkinter.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed coords
# mamba-strict-type: TypeError
"""Type wall: tkinter.Canvas.create_line(coords: typed); call it with the wrong type.

typeshed contract: coords is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tkinter import Canvas
obj = object.__new__(Canvas)
try:
    obj.create_line(_W())  # coords: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
