# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "groupby_consecutive_runs"
# subject = "itertools.groupby"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.groupby: groupby groups consecutive equal elements (no global sort), with and without a key function"""
import itertools

data = [1, 1, 2, 3, 3, 1]
groups = [(k, list(g)) for k, g in itertools.groupby(data)]
assert groups == [(1, [1, 1]), (2, [2]), (3, [3, 3]), (1, [1])], f"groupby = {groups!r}"

words = ["ant", "bear", "cat", "dog", "eagle"]
by_len = [(k, list(g)) for k, g in itertools.groupby(words, key=len)]
assert by_len[0] == (3, ["ant"]), f"by_len[0] = {by_len[0]!r}"

print("groupby_consecutive_runs OK")
