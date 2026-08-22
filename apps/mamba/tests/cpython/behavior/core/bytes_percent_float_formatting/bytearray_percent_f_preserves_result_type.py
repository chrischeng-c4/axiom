# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "bytes_percent_float_formatting"
# dimension = "behavior"
# case = "bytearray_percent_f_preserves_result_type"
# subject = "bytearray.percent_format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bytearray.percent_format: bytearray percent f formatting returns an exact bytearray result"""
result = bytearray(b"%.3f") % 2.5
assert result == bytearray(b"2.500")
assert type(result) is bytearray

print("bytearray_percent_f_preserves_result_type OK")
