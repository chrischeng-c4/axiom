# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "bytes_percent_float_formatting"
# dimension = "behavior"
# case = "bytes_percent_upper_g_accepts_float"
# subject = "bytes.percent_format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bytes.percent_format: bytes percent upper G formatting accepts a primitive float"""
assert b"%.3G" % 2500.0 == b"2.5E+03"

print("bytes_percent_upper_g_accepts_float OK")
