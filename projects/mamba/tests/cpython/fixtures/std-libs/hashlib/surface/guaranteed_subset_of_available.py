# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "surface"
# case = "guaranteed_subset_of_available"
# subject = "hashlib.algorithms_available"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""hashlib.algorithms_available: algorithms_available is a set/frozenset, guaranteed is a subset of it, and there is no placeholder 'undefined' algorithm"""
import hashlib

_alg = hashlib.algorithms_guaranteed
_avail = hashlib.algorithms_available
assert isinstance(_avail, (set, frozenset)), f"algorithms_available type = {type(_avail)!r}"
assert _alg <= _avail, "guaranteed subset of available"
assert "undefined" not in _avail, "no placeholder 'undefined' algorithm"

print("guaranteed_subset_of_available OK")
