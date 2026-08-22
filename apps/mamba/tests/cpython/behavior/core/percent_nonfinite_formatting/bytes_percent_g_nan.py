# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "percent_nonfinite_formatting"
# dimension = "behavior"
# case = "bytes_percent_g_nan"
# subject = "bytes.percent_format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bytes.percent_format: bytes percent g formatting renders nan without a bogus exponent"""
value = float("nan")
result = b"%.2g" % value
assert result == b"nan"
assert type(result) is bytes

print("bytes_percent_g_nan OK")
