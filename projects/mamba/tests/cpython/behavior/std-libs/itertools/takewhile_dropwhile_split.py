# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "takewhile_dropwhile_split"
# subject = "itertools.takewhile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.takewhile: takewhile yields the leading run where pred is true; dropwhile yields the rest once pred first fails"""
import itertools

assert list(itertools.takewhile(lambda x: x < 4, [1, 2, 3, 4, 5])) == [1, 2, 3], "takewhile"
assert list(itertools.dropwhile(lambda x: x < 4, [1, 2, 3, 4, 5])) == [4, 5], "dropwhile"
assert list(itertools.takewhile(lambda x: x < 0, [1, 2, 3])) == [], "takewhile none"
assert list(itertools.dropwhile(lambda x: x < 0, [1, 2, 3])) == [1, 2, 3], "dropwhile none"
assert list(itertools.dropwhile(lambda x: x < 100, [1, 2, 3])) == [], "dropwhile all"

print("takewhile_dropwhile_split OK")
