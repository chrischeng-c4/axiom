# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "type"
# case = "Fraction____mod____b_as_float_wrong"
# subject = "fractions.Fraction.__mod__(b: float)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed b"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/fractions.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed b
# mamba-strict-type: TypeError
"""Type wall: fractions.Fraction.__mod__(b: float); call it with the wrong type.

typeshed contract: b is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from fractions import Fraction
obj = object.__new__(Fraction)
try:
    obj.__mod__("not_a_float")  # b: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
