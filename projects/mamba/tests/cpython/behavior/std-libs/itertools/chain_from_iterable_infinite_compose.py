# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "chain_from_iterable_infinite_compose"
# subject = "itertools.chain.from_iterable"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.chain.from_iterable: chain.from_iterable over an infinite repeat composes lazily; islice bounds it without diverging"""
import itertools

endless = itertools.chain.from_iterable(itertools.repeat(range(5)))
assert list(itertools.islice(endless, 7)) == [0, 1, 2, 3, 4, 0, 1], "endless prefix"

empties = itertools.chain.from_iterable(() for _ in range(10000))
raised = False
try:
    next(empties)
except StopIteration:
    raised = True
assert raised, "long run of empty sub-iterables terminates cleanly"

print("chain_from_iterable_infinite_compose OK")
