# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "behavior"
# case = "container_roundtrip"
# subject = "pickle.loads"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pickle.py"
# status = "filled"
# ///
"""pickle.loads: list/tuple/dict (incl. nested) round-trip through dumps+loads equal to the original, including a list-of-lists with mixed depth"""
import pickle

containers = [
    [1, 2, 3],
    (1, 2, 3),
    {"a": 1, "b": [2, 3]},
    [[1, 2], [3, [4, 5]]],
]
for c in containers:
    rt = pickle.loads(pickle.dumps(c))
    assert rt == c, f"container round-trip {type(c).__name__}: {rt!r}"

print("container_roundtrip OK")
