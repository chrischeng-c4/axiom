# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "groupby_empty_input"
# subject = "itertools.groupby"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.groupby: groupby over empty input yields no groups, with and without a key function"""
import itertools

assert list(itertools.groupby([])) == [], "empty"
assert list(itertools.groupby([], key=id)) == [], "empty keyed"

print("groupby_empty_input OK")
