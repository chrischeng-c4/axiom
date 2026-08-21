# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tkinter"
# dimension = "type"
# case = "Wm__wm_aspect__minNumer_as_int_wrong"
# subject = "tkinter.Wm.wm_aspect(minNumer: int)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed minNumer"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tkinter.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed minNumer
# mamba-strict-type: TypeError
"""Type wall: tkinter.Wm.wm_aspect(minNumer: int); call it with the wrong type.

typeshed contract: minNumer is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from tkinter import Wm
obj = object.__new__(Wm)
try:
    obj.wm_aspect("not_an_int", 0, 0, 0)  # minNumer: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
