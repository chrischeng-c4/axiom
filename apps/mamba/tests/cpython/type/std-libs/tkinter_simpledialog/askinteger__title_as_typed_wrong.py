# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tkinter_simpledialog"
# dimension = "type"
# case = "askinteger__title_as_typed_wrong"
# subject = "tkinter.simpledialog.askinteger(title: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tkinter/simpledialog.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tkinter.simpledialog.askinteger(title: typed); call it with the wrong type.

typeshed contract: title is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tkinter.simpledialog import askinteger
try:
    askinteger(_W(), "")  # title: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
