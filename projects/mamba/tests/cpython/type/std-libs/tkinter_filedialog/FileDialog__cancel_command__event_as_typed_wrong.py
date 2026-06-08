# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tkinter_filedialog"
# dimension = "type"
# case = "FileDialog__cancel_command__event_as_typed_wrong"
# subject = "tkinter.filedialog.FileDialog.cancel_command(event: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tkinter/filedialog.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tkinter.filedialog.FileDialog.cancel_command(event: typed); call it with the wrong type.

typeshed contract: event is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tkinter.filedialog import FileDialog
obj = object.__new__(FileDialog)
try:
    obj.cancel_command(_W())  # event: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
