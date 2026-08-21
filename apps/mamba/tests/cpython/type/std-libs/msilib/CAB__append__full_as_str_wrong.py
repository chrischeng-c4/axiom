# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "msilib"
# dimension = "type"
# case = "CAB__append__full_as_str_wrong"
# subject = "msilib.CAB.append(full: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/msilib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: msilib.CAB.append(full: str); call it with the wrong type.

typeshed contract: full is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from msilib import CAB
obj = object.__new__(CAB)
try:
    obj.append(12345, "", "")  # full: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
