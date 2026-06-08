# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "accumulate_initial_seed"
# subject = "itertools.accumulate"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.accumulate: accumulate(it, initial=v) prepends the seed before folding the rest"""
import itertools

acc = list(itertools.accumulate([1, 2, 3, 4], initial=0))
assert acc == [0, 1, 3, 6, 10], f"accumulate initial = {acc!r}"

print("accumulate_initial_seed OK")
