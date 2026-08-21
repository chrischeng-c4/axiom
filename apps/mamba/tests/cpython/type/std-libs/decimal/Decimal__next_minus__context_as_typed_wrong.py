# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal__next_minus__context_as_typed_wrong"
# subject = "decimal.Decimal.next_minus(context: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.next_minus(context: typed); call it with the wrong type.

typeshed contract: context is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.next_minus(_W())  # context: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
