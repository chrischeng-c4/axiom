# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tkinter_ttk"
# dimension = "type"
# case = "Widget__state__statespec_as_typed_wrong"
# subject = "tkinter.ttk.Widget.state(statespec: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed statespec"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tkinter/ttk.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed statespec
# mamba-strict-type: TypeError
"""Type wall: tkinter.ttk.Widget.state(statespec: typed); call it with the wrong type.

typeshed contract: statespec is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tkinter.ttk import Widget
obj = object.__new__(Widget)
try:
    obj.state(_W())  # statespec: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
