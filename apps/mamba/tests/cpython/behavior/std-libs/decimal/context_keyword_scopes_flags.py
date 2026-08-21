# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "context_keyword_scopes_flags"
# subject = "decimal.Context"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Context: the context= keyword scopes precision/flags to a private Context: an overflow inside it sets that context's flag, leaving the active context untouched"""
from decimal import Decimal, Context, localcontext, Overflow

D = Decimal
# Decimal methods accept a context= keyword that scopes precision/flags
# without touching the active context.
xc = Context(prec=1, Emax=1, Emin=-1)
with localcontext() as active:
    active.clear_flags()
    assert D(9, context=xc) == 9, "constructor context= keyword"
    assert D("9.73").normalize(context=xc) == D("1E+1"), "normalize context= keyword"
    assert D("0.0625").sqrt(context=xc) == D("0.2"), "sqrt context= keyword"
    # An error inside xc sets xc's flag, not the active context's.
    xc.clear_flags()
    try:
        D(8).exp(context=xc)
        raise AssertionError("exp overflow should raise")
    except Overflow:
        pass
    assert xc.flags[Overflow], "overflow flag set on xc"
    assert not active.flags[Overflow], "active context flag untouched"

print("context_keyword_scopes_flags OK")
