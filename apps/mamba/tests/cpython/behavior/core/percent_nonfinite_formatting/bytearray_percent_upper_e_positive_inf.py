# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "percent_nonfinite_formatting"
# dimension = "behavior"
# case = "bytearray_percent_upper_e_positive_inf"
# subject = "bytearray.percent_format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bytearray.percent_format: bytearray percent E formatting renders positive inf without a bogus exponent"""
value = float("inf")
result = bytearray(b"%.2E") % value
assert result == bytearray(b"INF")
assert type(result) is bytearray

print("bytearray_percent_upper_e_positive_inf OK")
