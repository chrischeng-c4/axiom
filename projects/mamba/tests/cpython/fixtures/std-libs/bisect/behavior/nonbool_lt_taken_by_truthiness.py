# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "nonbool_lt_taken_by_truthiness"
# subject = "bisect.bisect_left"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bisect.py"
# status = "filled"
# ///
"""bisect.bisect_left: a __lt__ returning a non-bool is interpreted by truthiness during the search"""
import bisect


# __lt__ returns a non-bool (truthy/falsy str); bisect takes it by truthiness.
class NonBool:
    def __init__(self, val):
        self.val = val

    def __lt__(self, other):
        return "nonempty" if self.val < other.val else ""


data = [NonBool(i) for i in range(100)]
assert bisect.bisect_left(data, NonBool(33)) == 33, "non-bool __lt__ left"
assert bisect.bisect_right(data, NonBool(33)) == 34, "non-bool __lt__ right"

print("nonbool_lt_taken_by_truthiness OK")
