# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "iterdir_lists_children"
# subject = "pathlib.Path"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.Path: in a TemporaryDirectory holding two files, list(iterdir()) yields exactly two entries"""
import pathlib

import tempfile
Path = pathlib.Path

with tempfile.TemporaryDirectory() as _tmpdir:
    _d = Path(_tmpdir)
    (_d / "x").write_text("x")
    (_d / "y").write_text("y")
    _items = list(_d.iterdir())
    assert len(_items) == 2, f"iterdir count = {len(_items)!r}"
print("iterdir_lists_children OK")
