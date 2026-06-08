# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "groupby_shared_iterator_invalidation"
# subject = "itertools.groupby"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.groupby: groupby groups share one underlying iterator; advancing the outer iterator empties earlier un-consumed groups"""
import itertools

data = list(zip("AABBBAAAA", range(9)))
it = itertools.groupby(data, key=lambda r: r[0])
_, g1 = next(it)
_, g2 = next(it)
_, g3 = next(it)
assert list(g1) == [], "stale g1"
assert list(g2) == [], "stale g2"
assert next(g3) == ("A", 5), "live g3 first"
list(it)  # exhaust outer -> g3 also goes stale
assert list(g3) == [], "g3 stale after outer exhausted"

print("groupby_shared_iterator_invalidation OK")
