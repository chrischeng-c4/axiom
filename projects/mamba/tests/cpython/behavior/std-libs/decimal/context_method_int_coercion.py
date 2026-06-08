# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "behavior"
# case = "context_method_int_coercion"
# subject = "decimal.Context"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Context: Context arithmetic/utility methods coerce int operands to Decimal: to_eng_string/normalize/to_integral_value/copy_decimal/number_class agree on int vs Decimal input"""
from decimal import Decimal, Context

D = Decimal
c = Context()
# Context arithmetic/utility methods coerce int operands to Decimal.
assert c.to_eng_string(10) == c.to_eng_string(D(10)), "to_eng_string int"
assert c.normalize(10) == c.normalize(D(10)), "normalize int"
assert c.to_integral_value(10) == c.to_integral_value(D(10)), "to_integral_value int"
assert c.copy_decimal(-1) == c.copy_decimal(D(-1)), "copy_decimal int"
assert c.number_class(123) == c.number_class(D(123)), "number_class int"

print("context_method_int_coercion OK")
