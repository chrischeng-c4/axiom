# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "type"
# case = "MH__get_folder__folder_as_StrPath_wrong"
# subject = "mailbox.MH.get_folder(folder: StrPath)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailbox.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mailbox.MH.get_folder(folder: StrPath); call it with the wrong type.

typeshed contract: folder is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from mailbox import MH
obj = object.__new__(MH)
try:
    obj.get_folder(_W())  # folder: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
