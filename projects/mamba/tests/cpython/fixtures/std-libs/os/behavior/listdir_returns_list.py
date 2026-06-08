# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "listdir_returns_list"
# subject = "os.listdir"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.listdir: os.listdir('.') returns a list of entry-name strings"""
import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    for name in ("a.txt", "b.txt", "c.txt"):
        with open(os.path.join(d, name), "w", encoding="utf-8") as f:
            f.write("x")
    entries = os.listdir(d)
    assert isinstance(entries, list), f"listdir type = {type(entries)!r}"
    assert all(isinstance(e, str) for e in entries), "entries are str"
    assert set(entries) == {"a.txt", "b.txt", "c.txt"}, f"entries = {set(entries)!r}"
print("listdir_returns_list OK")
