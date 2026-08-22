# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "bytes_percent_float_formatting"
# dimension = "behavior"
# case = "bytes_percent_plus_sign_width_precision"
# subject = "bytes.percent_format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bytes.percent_format: bytes percent f formatting honors plus sign width and precision"""
assert b"%+8.2f" % 1.5 == b"   +1.50"

print("bytes_percent_plus_sign_width_precision OK")
