# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "bytes_percent_float_formatting"
# dimension = "behavior"
# case = "bytes_percent_f_accepts_dunder_float"
# subject = "bytes.percent_format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bytes.percent_format: bytes percent f formatting accepts a class-level __float__ protocol"""
class FloatLike:
    def __float__(self):
        return 2.5


assert b"%.3f" % FloatLike() == b"2.500"

print("bytes_percent_f_accepts_dunder_float OK")
