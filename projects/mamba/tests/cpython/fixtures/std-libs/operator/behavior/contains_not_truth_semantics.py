# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "contains_not_truth_semantics"
# subject = "operator.contains"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.contains: contains tests membership (b in a), not_ returns the boolean negation, and truth maps any object to its bool — all returning the bool singletons"""
import operator

assert operator.contains("hello", "ell") is True, "contains substring"
assert operator.contains([1, 2, 3], 2) is True, "contains element"
assert operator.contains([1, 2, 3], 4) is False, "not contains"
assert operator.not_(0) is True, "not_(0)"
assert operator.not_("non-empty") is False, "not_(non-empty)"
assert operator.not_(False) is True, "not_(False)"
assert operator.truth(0) is False, "truth(0)"
assert operator.truth(1) is True, "truth(1)"
assert operator.truth("") is False, "truth empty str"
assert operator.truth([1]) is True, "truth non-empty"

print("contains_not_truth_semantics OK")
