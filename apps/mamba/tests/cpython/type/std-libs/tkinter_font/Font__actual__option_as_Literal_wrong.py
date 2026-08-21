# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tkinter_font"
# dimension = "type"
# case = "Font__actual__option_as_Literal_wrong"
# subject = "tkinter.font.Font.actual(option: Literal)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed option"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tkinter/font.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed option
# mamba-strict-type: TypeError
"""Type wall: tkinter.font.Font.actual(option: Literal); call it with the wrong type.

typeshed contract: option is Literal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tkinter.font import Font
obj = object.__new__(Font)
try:
    obj.actual(_W())  # option: Literal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
