# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "type"
# case = "Fraction__from_decimal__dec_as_Decimal_wrong"
# subject = "fractions.Fraction.from_decimal(dec: Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/fractions.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: fractions.Fraction.from_decimal(dec: Decimal); call it with the wrong type.

typeshed contract: dec is Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from fractions import Fraction
try:
    Fraction.from_decimal(_W())  # dec: Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
