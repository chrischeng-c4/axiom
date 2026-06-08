# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "localcontext_keyword_overrides"
# subject = "decimal.localcontext"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.localcontext: localcontext accepts keyword overrides (prec/rounding/Emin/Emax) and applies them to the yielded context"""
from decimal import localcontext, ROUND_HALF_DOWN

# localcontext accepts keyword overrides and applies them to the yielded ctx.
with localcontext(prec=10, rounding=ROUND_HALF_DOWN, Emin=-20, Emax=20) as ctx:
    assert ctx.prec == 10 and ctx.rounding == ROUND_HALF_DOWN, "kw overrides applied"

print("localcontext_keyword_overrides OK")
