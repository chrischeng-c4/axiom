# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tkinter"
# dimension = "type"
# case = "Wm__wm_colormapwindows__first_wlist_item_as_Misc_wrong"
# subject = "tkinter.Wm.wm_colormapwindows(first_wlist_item: Misc)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed first_wlist_item"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tkinter.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed first_wlist_item
# mamba-strict-type: TypeError
"""Type wall: tkinter.Wm.wm_colormapwindows(first_wlist_item: Misc); call it with the wrong type.

typeshed contract: first_wlist_item is Misc. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tkinter import Wm
obj = object.__new__(Wm)
try:
    obj.wm_colormapwindows(_W())  # first_wlist_item: Misc <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
