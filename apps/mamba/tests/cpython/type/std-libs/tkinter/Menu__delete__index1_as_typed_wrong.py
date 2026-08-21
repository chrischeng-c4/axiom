# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tkinter"
# dimension = "type"
# case = "Menu__delete__index1_as_typed_wrong"
# subject = "tkinter.Menu.delete(index1: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tkinter.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tkinter.Menu.delete(index1: typed); call it with the wrong type.

typeshed contract: index1 is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tkinter import Menu
obj = object.__new__(Menu)
try:
    obj.delete(_W())  # index1: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
