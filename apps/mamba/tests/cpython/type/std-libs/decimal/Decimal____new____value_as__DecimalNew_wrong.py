# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal____new____value_as__DecimalNew_wrong"
# subject = "decimal.Decimal.__new__(value: _DecimalNew)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.__new__(value: _DecimalNew); call it with the wrong type.

typeshed contract: value is _DecimalNew. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.__new__(_W())  # value: _DecimalNew <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
