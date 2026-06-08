# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_tkinter"
# dimension = "type"
# case = "setbusywaitinterval__new_val_as_int_wrong"
# subject = "_tkinter.setbusywaitinterval(new_val: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_tkinter.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _tkinter.setbusywaitinterval(new_val: int); call it with the wrong type.

typeshed contract: new_val is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _tkinter import setbusywaitinterval
try:
    setbusywaitinterval("not_an_int")  # new_val: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
