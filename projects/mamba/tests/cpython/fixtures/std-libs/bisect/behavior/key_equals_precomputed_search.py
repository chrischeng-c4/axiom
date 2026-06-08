# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "behavior"
# case = "key_equals_precomputed_search"
# subject = "bisect.bisect_left"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bisect.py"
# status = "filled"
# ///
"""bisect.bisect_left: bisect_*(arr, x, key=f) matches searching the precomputed [f(v) for v in arr] with x"""
import bisect

# key= search over a list sorted by abs() equals searching the precomputed
# key list with the bare value.
keyfunc = abs
arr = sorted([2, -4, 6, 8, -10], key=keyfunc)   # [2, -4, 6, 8, -10]
precomputed = [keyfunc(v) for v in arr]         # [2, 4, 6, 8, 10]
for x in precomputed:
    assert bisect.bisect_left(arr, x, key=keyfunc) == bisect.bisect_left(precomputed, x)
    assert bisect.bisect_right(arr, x, key=keyfunc) == bisect.bisect_right(precomputed, x)

# Same equivalence with a string-casefold key over mixed-case letters.
kf = str.casefold
letters = sorted("aBcDeEfg", key=kf)
pre = [kf(v) for v in letters]
for x in pre:
    assert bisect.bisect_left(letters, x, key=kf) == bisect.bisect_left(pre, x)
    assert bisect.bisect_right(letters, x, key=kf) == bisect.bisect_right(pre, x)

print("key_equals_precomputed_search OK")
