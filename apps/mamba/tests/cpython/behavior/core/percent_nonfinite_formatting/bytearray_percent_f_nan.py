# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "percent_nonfinite_formatting"
# dimension = "behavior"
# case = "bytearray_percent_f_nan"
# subject = "bytearray.percent_format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bytearray.percent_format: bytearray percent f formatting renders nan without a bogus exponent"""
value = float("nan")
result = bytearray(b"%.2f") % value
assert result == bytearray(b"nan")
assert type(result) is bytearray

print("bytearray_percent_f_nan OK")
