# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "percent_nonfinite_formatting"
# dimension = "behavior"
# case = "bytes_percent_upper_e_negative_inf"
# subject = "bytes.percent_format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bytes.percent_format: bytes percent E formatting renders negative inf without a bogus exponent"""
value = float("-inf")
result = b"%.2E" % value
assert result == b"-INF"
assert type(result) is bytes

print("bytes_percent_upper_e_negative_inf OK")
