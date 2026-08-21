# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal__from_float__f_as_float_wrong"
# subject = "decimal.Decimal.from_float(f: float)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.from_float(f: float); call it with the wrong type.

typeshed contract: f is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from decimal import Decimal
try:
    Decimal.from_float("not_a_float")  # f: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
