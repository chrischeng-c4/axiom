# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tkinter_ttk"
# dimension = "type"
# case = "Style__theme_use__themename_as_str_wrong"
# subject = "tkinter.ttk.Style.theme_use(themename: str)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed themename"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tkinter/ttk.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed themename
# mamba-strict-type: TypeError
"""Type wall: tkinter.ttk.Style.theme_use(themename: str); call it with the wrong type.

typeshed contract: themename is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from tkinter.ttk import Style
obj = object.__new__(Style)
try:
    obj.theme_use(12345)  # themename: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
