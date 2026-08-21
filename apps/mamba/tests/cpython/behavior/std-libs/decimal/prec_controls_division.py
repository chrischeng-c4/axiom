# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "prec_controls_division"
# subject = "decimal.localcontext"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.localcontext: a localcontext with prec=4 rounds 1/3 to '0.3333'"""
from decimal import Decimal, localcontext

# A context's prec controls default precision of inexact results.
with localcontext() as _ctx:
    _ctx.prec = 4
    _result = Decimal("1") / Decimal("3")
    assert str(_result) == "0.3333", f"prec=4 division = {str(_result)!r}"

print("prec_controls_division OK")
