# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "formatting"
# dimension = "errors"
# case = "percent_f_rejects_invalid_dunder_float_return"
# subject = "str.percent_format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.percent_format: percent f pins the TypeError taxonomy for an invalid __float__ return"""
class FloatBad:
    def __float__(self):
        return 3


try:
    "%.3f" % FloatBad()
except TypeError as exc:
    assert str(exc) == "FloatBad.__float__ returned non-float (type int)"
else:
    raise AssertionError("expected TypeError")

print("percent_f_rejects_invalid_dunder_float_return OK")
