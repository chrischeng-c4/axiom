# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "errors"
# case = "context_method_str_operand_raises_typeerror"
# subject = "decimal.Context"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Context: context_method_str_operand_raises_typeerror (errors)."""
import decimal

_raised = False
try:
    decimal.Context().to_eng_string('10')
except TypeError:
    _raised = True
assert _raised, "context_method_str_operand_raises_typeerror: expected TypeError"
print("context_method_str_operand_raises_typeerror OK")
