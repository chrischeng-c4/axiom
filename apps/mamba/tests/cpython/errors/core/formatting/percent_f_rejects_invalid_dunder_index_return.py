# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "formatting"
# dimension = "errors"
# case = "percent_f_rejects_invalid_dunder_index_return"
# subject = "str.percent_format"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""str.percent_format: percent f pins the TypeError taxonomy for an invalid __index__ return"""
class IndexBad:
    def __index__(self):
        return 2.5


try:
    "%.3f" % IndexBad()
except TypeError as exc:
    assert str(exc) == "__index__ returned non-int (type float)"
else:
    raise AssertionError("expected TypeError")

print("percent_f_rejects_invalid_dunder_index_return OK")
