# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "round_dunder_ndigits"
# subject = "decimal.Decimal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: __round__() rounds to an int-valued Decimal, __round__(ndigits) rescales (negative ndigits shifts left), and the round() builtin applies banker's rounding"""
from decimal import Decimal, localcontext

D = Decimal
# __round__ with no digits rounds to an int-valued Decimal; with ndigits it
# rescales (negative ndigits shifts left of the point).
with localcontext() as ctx:
    ctx.prec = 28
    assert str(D("9.99").__round__()) == "10", "round() to integer"
    assert str(D("1.23456789").__round__(5)) == "1.23457", "round(ndigits=5)"
    assert str(D("1.2345").__round__(-10)) == "0E+10", "round(ndigits=-10)"
# round() built-in returns an int and applies banker's rounding.
assert round(D("2.5")) == 2, "round() banker's rounding"
assert round(D("3.5")) == 4, "round() banker's rounding up"

print("round_dunder_ndigits OK")
