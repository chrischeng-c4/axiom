# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "glob_matches_suffix"
# subject = "pathlib.Path"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.Path: in a TemporaryDirectory holding a.txt/b.txt/c.py, glob('*.txt') returns exactly the two .txt entries and every result has suffix '.txt'"""
import pathlib

import tempfile
Path = pathlib.Path

with tempfile.TemporaryDirectory() as _tmpdir:
    _base_p = Path(_tmpdir)
    (_base_p / "a.txt").write_text("a")
    (_base_p / "b.txt").write_text("b")
    (_base_p / "c.py").write_text("c")
    _txts = sorted(_base_p.glob("*.txt"))
    assert len(_txts) == 2, f"glob *.txt count = {len(_txts)!r}"
    assert all(p.suffix == ".txt" for p in _txts), "all .txt"
print("glob_matches_suffix OK")
