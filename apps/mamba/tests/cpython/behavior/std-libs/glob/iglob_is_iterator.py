# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "behavior"
# case = "iglob_is_iterator"
# subject = "glob.iglob"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_glob.py"
# status = "filled"
# ///
"""glob.iglob: iglob returns a lazy iterator (has __iter__ and __next__) whose materialized results match glob() for the same pattern"""
import glob
import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    for name in ("a.txt", "b.txt", "c.txt"):
        with open(os.path.join(d, name), "w") as fh:
            fh.write("")
    pattern = os.path.join(d, "*.txt")
    it = glob.iglob(pattern)
    assert hasattr(it, "__iter__"), "iglob iterable"
    assert hasattr(it, "__next__"), "iglob iterator"
    materialized = sorted(it)
    assert sorted(glob.glob(pattern)) == materialized, "iglob == glob results"
    assert len(materialized) == 3, f"iglob count = {len(materialized)!r}"

print("iglob_is_iterator OK")
