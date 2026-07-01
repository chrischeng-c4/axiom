# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "type"
# case = "Fraction____new____numerator_as__ConvertibleToIntegerRatio_wrong"
# subject = "fractions.Fraction.__new__(numerator: _ConvertibleToIntegerRatio)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/fractions.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: fractions.Fraction.__new__(numerator: _ConvertibleToIntegerRatio); call it with the wrong type.

typeshed contract: numerator is _ConvertibleToIntegerRatio. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from fractions import Fraction
obj = object.__new__(Fraction)
try:
    obj.__new__(_W())  # numerator: _ConvertibleToIntegerRatio <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
