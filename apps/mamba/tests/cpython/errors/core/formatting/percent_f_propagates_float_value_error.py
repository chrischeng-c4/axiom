# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "formatting"
# dimension = "errors"
# case = "percent_f_propagates_float_value_error"
# subject = "str.percent_format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.percent_format: percent f preserves a ValueError raised by __float__"""
class FloatRaises:
    def __float__(self):
        raise ValueError("float sentinel")


try:
    "%.3f" % FloatRaises()
except ValueError as exc:
    assert str(exc) == "float sentinel"
else:
    raise AssertionError("expected ValueError")

print("percent_f_propagates_float_value_error OK")
