# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "behavior"
# case = "numeric_digit_decimal_values"
# subject = "unicodedata.numeric"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicodedata.py"
# status = "filled"
# ///
"""unicodedata.numeric: digit('5')==5, decimal('7')==7, numeric vulgar-half (U+00BD)==0.5"""
import unicodedata

assert unicodedata.digit("5") == 5, f"digit '5' = {unicodedata.digit('5')!r}"
assert unicodedata.decimal("7") == 7, f"decimal '7' = {unicodedata.decimal('7')!r}"
assert unicodedata.numeric("½") == 0.5, f"numeric half = {unicodedata.numeric('½')!r}"

print("numeric_digit_decimal_values OK")
