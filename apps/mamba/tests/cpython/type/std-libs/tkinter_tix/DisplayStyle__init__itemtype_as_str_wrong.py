# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tkinter_tix"
# dimension = "type"
# case = "DisplayStyle__init__itemtype_as_str_wrong"
# subject = "tkinter.tix.DisplayStyle.__init__(itemtype: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tkinter/tix.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tkinter.tix.DisplayStyle.__init__(itemtype: str); call it with the wrong type.

typeshed contract: itemtype is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from tkinter.tix import DisplayStyle
try:
    DisplayStyle(12345)  # itemtype: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
