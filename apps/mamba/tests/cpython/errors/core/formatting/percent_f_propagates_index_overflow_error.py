# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "formatting"
# dimension = "errors"
# case = "percent_f_propagates_index_overflow_error"
# subject = "str.percent_format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.percent_format: percent f preserves an OverflowError raised by __index__"""
class IndexRaises:
    def __index__(self):
        raise OverflowError("index sentinel")


try:
    "%.3f" % IndexRaises()
except OverflowError as exc:
    assert str(exc) == "index sentinel"
else:
    raise AssertionError("expected OverflowError")

print("percent_f_propagates_index_overflow_error OK")
