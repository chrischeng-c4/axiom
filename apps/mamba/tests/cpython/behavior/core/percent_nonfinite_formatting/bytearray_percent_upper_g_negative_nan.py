# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "percent_nonfinite_formatting"
# dimension = "behavior"
# case = "bytearray_percent_upper_g_negative_nan"
# subject = "bytearray.percent_format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bytearray.percent_format: bytearray percent G formatting discards a negative NaN sign"""
value = -float("nan")
result = bytearray(b"%.2G") % value
assert result == bytearray(b"NAN")
assert type(result) is bytearray

print("bytearray_percent_upper_g_negative_nan OK")
