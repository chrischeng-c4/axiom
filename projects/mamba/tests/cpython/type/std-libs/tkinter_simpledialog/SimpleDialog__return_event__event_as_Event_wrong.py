# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tkinter_simpledialog"
# dimension = "type"
# case = "SimpleDialog__return_event__event_as_Event_wrong"
# subject = "tkinter.simpledialog.SimpleDialog.return_event(event: Event)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed event"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tkinter/simpledialog.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed event
# mamba-strict-type: TypeError
"""Type wall: tkinter.simpledialog.SimpleDialog.return_event(event: Event); call it with the wrong type.

typeshed contract: event is Event. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tkinter.simpledialog import SimpleDialog
obj = object.__new__(SimpleDialog)
try:
    obj.return_event(_W())  # event: Event <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
