# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "accepts_keyword_arguments"
# subject = "bisect.bisect_left"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bisect.py"
# status = "filled"
# ///
"""bisect.bisect_left: all functions accept a / x / lo / hi as keyword arguments"""
import bisect

data = [10, 20, 30, 40, 50]
assert bisect.bisect_left(a=data, x=25, lo=1, hi=3) == 2
assert bisect.bisect_right(a=data, x=25, lo=1, hi=3) == 2
assert bisect.bisect(a=data, x=25, lo=1, hi=3) == 2
bisect.insort_left(a=data, x=25, lo=1, hi=3)
bisect.insort_right(a=data, x=25, lo=1, hi=3)
bisect.insort(a=data, x=25, lo=1, hi=3)
assert data == [10, 20, 25, 25, 25, 30, 40, 50], f"keyword insort = {data!r}"

print("accepts_keyword_arguments OK")
