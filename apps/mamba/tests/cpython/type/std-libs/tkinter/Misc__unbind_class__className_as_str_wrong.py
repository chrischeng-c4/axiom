# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tkinter"
# dimension = "type"
# case = "Misc__unbind_class__className_as_str_wrong"
# subject = "tkinter.Misc.unbind_class(className: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tkinter.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tkinter.Misc.unbind_class(className: str); call it with the wrong type.

typeshed contract: className is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from tkinter import Misc
obj = object.__new__(Misc)
try:
    obj.unbind_class(12345, "")  # className: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
