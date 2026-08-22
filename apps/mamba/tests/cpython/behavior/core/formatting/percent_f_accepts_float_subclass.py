# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "formatting"
# dimension = "behavior"
# case = "percent_f_accepts_float_subclass"
# subject = "str.percent_format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.percent_format: percent f formatting accepts a float subclass"""
class FloatSub(float):
    pass


assert "%.3f" % FloatSub(2.5) == "2.500"

print("percent_f_accepts_float_subclass OK")
