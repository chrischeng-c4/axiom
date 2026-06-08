# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tkinter_filedialog"
# dimension = "type"
# case = "FileDialog__files_double_event__event_as_Event_wrong"
# subject = "tkinter.filedialog.FileDialog.files_double_event(event: Event)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tkinter/filedialog.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tkinter.filedialog.FileDialog.files_double_event(event: Event); call it with the wrong type.

typeshed contract: event is Event. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tkinter.filedialog import FileDialog
obj = object.__new__(FileDialog)
try:
    obj.files_double_event(_W())  # event: Event <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
