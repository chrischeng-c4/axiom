# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tkinter_filedialog"
# dimension = "type"
# case = "FileDialog__set_selection__file_as_StrPath_wrong"
# subject = "tkinter.filedialog.FileDialog.set_selection(file: StrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tkinter/filedialog.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tkinter.filedialog.FileDialog.set_selection(file: StrPath); call it with the wrong type.

typeshed contract: file is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tkinter.filedialog import FileDialog
obj = object.__new__(FileDialog)
try:
    obj.set_selection(_W())  # file: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
