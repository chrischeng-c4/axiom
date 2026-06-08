# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "groupby_nested_roundtrip"
# subject = "itertools.groupby"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.groupby: nested groupby (by first field, then second) re-assembles exactly the original rows"""
import itertools

rows = [(0, 10), (0, 10), (0, 11), (1, 11), (1, 12)]
rebuilt = []
for k, g in itertools.groupby(rows, key=lambda r: r[0]):
    for ik, ig in itertools.groupby(g, key=lambda r: r[1]):
        for elem in ig:
            assert k == elem[0] and ik == elem[1], f"nested key {elem!r}"
            rebuilt.append(elem)
assert rebuilt == rows, f"rebuilt = {rebuilt!r}"

print("groupby_nested_roundtrip OK")
