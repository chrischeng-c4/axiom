# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "behavior"
# case = "escape_neutralizes_metachars"
# subject = "glob.escape"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_glob.py"
# status = "filled"
# ///
"""glob.escape: escape() makes a name with literal glob metachars matchable: a real file 'file[1].txt' is found by glob(escape(path)) and nothing else"""
import glob
import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    fname = "file[1].txt"
    with open(os.path.join(d, fname), "w") as fh:
        fh.write("")
    escaped = glob.escape(os.path.join(d, fname))
    results = glob.glob(escaped)
    assert len(results) == 1, f"escaped match count = {len(results)!r}"
    assert os.path.basename(results[0]) == fname, f"exact match = {results!r}"

print("escape_neutralizes_metachars OK")
