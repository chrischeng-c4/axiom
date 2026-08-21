# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "islice_bounds_infinite_source"
# subject = "itertools.islice"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.islice: islice bounds an infinite count(): islice(count(), 5) yields exactly the first five values"""
import itertools

limited = list(itertools.islice(itertools.count(), 5))
assert limited == [0, 1, 2, 3, 4], f"islice count = {limited!r}"

stepped = list(itertools.islice(itertools.count(100, 5), 4))
assert stepped == [100, 105, 110, 115], f"islice count step = {stepped!r}"

print("islice_bounds_infinite_source OK")
