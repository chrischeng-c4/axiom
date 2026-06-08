# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "type"
# case = "Fraction__limit_denominator__max_denominator_as_int_wrong"
# subject = "fractions.Fraction.limit_denominator(max_denominator: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/fractions.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: fractions.Fraction.limit_denominator(max_denominator: int); call it with the wrong type.

typeshed contract: max_denominator is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from fractions import Fraction
obj = object.__new__(Fraction)
try:
    obj.limit_denominator("not_an_int")  # max_denominator: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
