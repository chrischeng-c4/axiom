# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cgitb"
# dimension = "type"
# case = "enable__display_as_int_wrong"
# subject = "cgitb.enable(display: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cgitb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cgitb.enable(display: int); call it with the wrong type.

typeshed contract: display is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from cgitb import enable
try:
    enable("not_an_int")  # display: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
