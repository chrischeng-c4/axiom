# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "normaldist_value_dunders"
# subject = "statistics.NormalDist"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.NormalDist: NormalDist is a value type: repr round-trips, equality is by (mu,sigma), it is hashable (set dedups equal instances), copy/deepcopy preserve equality, and __slots__ means no instance __dict__"""
import copy
from statistics import NormalDist

# repr round-trips the constructor arguments.
assert repr(NormalDist(37.5, 5.625)) == "NormalDist(mu=37.5, sigma=5.625)"
# Equality is by (mu, sigma).
assert NormalDist() == NormalDist() and NormalDist(2, 4) != NormalDist(2, 8)
# Hashable: a set dedups equal instances.
assert len({NormalDist(100, 15), NormalDist(100.0, 15.0), NormalDist(95, 15)}) == 2
# copy / deepcopy preserve equality.
nd = NormalDist(37.5, 5.625)
assert copy.copy(nd) == nd and copy.deepcopy(nd) == nd
# __slots__ means there is no instance __dict__.
_raised = False
try:
    vars(nd)
except TypeError:
    _raised = True
assert _raised, "NormalDist should have no __dict__"

print("normaldist_value_dunders OK")
