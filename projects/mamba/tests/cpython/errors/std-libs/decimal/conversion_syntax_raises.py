# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "errors"
# case = "conversion_syntax_raises"
# subject = "decimal.Decimal"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: conversion_syntax_raises (errors)."""
import decimal

_raised = False
try:
    decimal.Decimal('not_a_number')
except decimal.InvalidOperation:
    _raised = True
assert _raised, "conversion_syntax_raises: expected decimal.InvalidOperation"
print("conversion_syntax_raises OK")
