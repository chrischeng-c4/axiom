# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tkinter_tix"
# dimension = "type"
# case = "PopupMenu__post_widget__widget_as_Widget_wrong"
# subject = "tkinter.tix.PopupMenu.post_widget(widget: Widget)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tkinter/tix.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tkinter.tix.PopupMenu.post_widget(widget: Widget); call it with the wrong type.

typeshed contract: widget is Widget. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tkinter.tix import PopupMenu
obj = object.__new__(PopupMenu)
try:
    obj.post_widget(_W(), 0, 0)  # widget: Widget <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
