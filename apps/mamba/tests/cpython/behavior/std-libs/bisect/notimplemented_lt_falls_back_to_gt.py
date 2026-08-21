# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "notimplemented_lt_falls_back_to_gt"
# subject = "bisect.bisect_left"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bisect.py"
# status = "filled"
# ///
"""bisect.bisect_left: when __lt__ returns NotImplemented the reflected __gt__ drives the comparison"""
import bisect


# __lt__ returns NotImplemented -> Python falls back to the reflected __gt__.
class FallBack:
    def __init__(self, val):
        self.val = val

    def __lt__(self, other):
        return NotImplemented

    def __gt__(self, other):
        return self.val > other.val


d2 = [FallBack(i) for i in range(100)]
assert bisect.bisect_left(d2, FallBack(40)) == 40, "notimplemented fallback left"
assert bisect.bisect_right(d2, FallBack(40)) == 41, "notimplemented fallback right"

print("notimplemented_lt_falls_back_to_gt OK")
