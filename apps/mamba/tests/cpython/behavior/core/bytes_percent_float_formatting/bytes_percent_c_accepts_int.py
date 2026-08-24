# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "bytes_percent_float_formatting"
# dimension = "behavior"
# case = "bytes_percent_c_accepts_int"
# subject = "bytes.percent_format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bytes.percent_format: existing bytes percent c conversion remains unchanged"""
assert b"%c" % 65 == b"A"

print("bytes_percent_c_accepts_int OK")
