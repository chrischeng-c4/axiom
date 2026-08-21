# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "type"
# case = "str__val_as_float_wrong"
# subject = "locale.str(val: float)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/locale.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: locale.str(val: float); call it with the wrong type.

typeshed contract: val is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from locale import str
try:
    str("not_a_float")  # val: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
