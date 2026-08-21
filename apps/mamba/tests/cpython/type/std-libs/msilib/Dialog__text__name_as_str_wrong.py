# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "msilib"
# dimension = "type"
# case = "Dialog__text__name_as_str_wrong"
# subject = "msilib.Dialog.text(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/msilib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: msilib.Dialog.text(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from msilib import Dialog
obj = object.__new__(Dialog)
try:
    obj.text(12345, 0, 0, 0, 0, 0, None)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
