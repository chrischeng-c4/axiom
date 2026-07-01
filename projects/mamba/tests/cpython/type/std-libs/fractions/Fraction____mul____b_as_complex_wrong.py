# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "type"
# case = "Fraction____mul____b_as_complex_wrong"
# subject = "fractions.Fraction.__mul__(b: complex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/fractions.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: fractions.Fraction.__mul__(b: complex); call it with the wrong type.

typeshed contract: b is complex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from fractions import Fraction
obj = object.__new__(Fraction)
try:
    obj.__mul__("not_a_complex")  # b: complex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
