# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "errors"
# case = "length_hint_negative_valueerror"
# subject = "operator.length_hint"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.length_hint: a __length_hint__ returning a negative count makes operator.length_hint raise ValueError"""
import operator


class Hinted:
    def __length_hint__(self):
        return -2


_raised = False
try:
    operator.length_hint(Hinted())
except ValueError:
    _raised = True
assert _raised, "expected ValueError for negative __length_hint__"
print("length_hint_negative_valueerror OK")
