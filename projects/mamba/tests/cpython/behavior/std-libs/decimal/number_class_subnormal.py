# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "number_class_subnormal"
# subject = "decimal.Context.number_class"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Context.number_class: number_class names the IEEE category: under a tight context Decimal('0.01').number_class() == '+Subnormal'"""
from decimal import Decimal, Context

D = Decimal
# number_class names the IEEE category of a value under a tight context.
xc = Context(prec=1, Emax=1, Emin=-1)
assert D("0.01").number_class(context=xc) == "+Subnormal", "number_class subnormal"

print("number_class_subnormal OK")
