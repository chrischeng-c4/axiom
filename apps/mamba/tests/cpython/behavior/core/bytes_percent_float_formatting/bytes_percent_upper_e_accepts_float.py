# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "bytes_percent_float_formatting"
# dimension = "behavior"
# case = "bytes_percent_upper_e_accepts_float"
# subject = "bytes.percent_format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bytes.percent_format: bytes percent upper E formatting accepts a primitive float"""
assert b"%.3E" % 2.5 == b"2.500E+00"

print("bytes_percent_upper_e_accepts_float OK")
