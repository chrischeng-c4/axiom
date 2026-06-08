# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_tkinter"
# dimension = "type"
# case = "TkappType__settrace__func_as_typed_wrong"
# subject = "_tkinter.TkappType.settrace(func: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_tkinter.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _tkinter.TkappType.settrace(func: typed); call it with the wrong type.

typeshed contract: func is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _tkinter import TkappType
obj = object.__new__(TkappType)
try:
    obj.settrace(_W())  # func: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
