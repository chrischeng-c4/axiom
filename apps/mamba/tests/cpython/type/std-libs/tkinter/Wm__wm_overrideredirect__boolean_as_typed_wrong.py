# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tkinter"
# dimension = "type"
# case = "Wm__wm_overrideredirect__boolean_as_typed_wrong"
# subject = "tkinter.Wm.wm_overrideredirect(boolean: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed boolean"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tkinter.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed boolean
# mamba-strict-type: TypeError
"""Type wall: tkinter.Wm.wm_overrideredirect(boolean: typed); call it with the wrong type.

typeshed contract: boolean is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tkinter import Wm
obj = object.__new__(Wm)
try:
    obj.wm_overrideredirect(_W())  # boolean: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
