# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "bytes_percent_float_formatting"
# dimension = "behavior"
# case = "bytes_percent_f_accepts_dunder_index"
# subject = "bytes.percent_format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bytes.percent_format: bytes percent f formatting accepts a class-level __index__ protocol"""
class IndexLike:
    def __index__(self):
        return 7


assert b"%.3f" % IndexLike() == b"7.000"

print("bytes_percent_f_accepts_dunder_index OK")
