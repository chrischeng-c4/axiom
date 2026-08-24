# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "bytes_percent_float_formatting"
# dimension = "behavior"
# case = "bytes_percent_f_accepts_float_control"
# subject = "bytes.percent_format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bytes.percent_format: ordinary bytes percent f formatting remains pinned with a primitive float"""
assert b"%.3f" % 2.5 == b"2.500"

print("bytes_percent_f_accepts_float_control OK")
