# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tkinter"
# dimension = "type"
# case = "Misc__winfo_visualsavailable__includeids_as_bool_wrong"
# subject = "tkinter.Misc.winfo_visualsavailable(includeids: bool)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed includeids"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tkinter.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed includeids
# mamba-strict-type: TypeError
"""Type wall: tkinter.Misc.winfo_visualsavailable(includeids: bool); call it with the wrong type.

typeshed contract: includeids is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from tkinter import Misc
obj = object.__new__(Misc)
try:
    obj.winfo_visualsavailable("not_a_bool")  # includeids: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
