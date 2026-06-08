# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "anchor_absolute_vs_relative"
# subject = "pathlib.Path"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.Path: Path('/a/b').anchor == '/' (absolute root) while Path('a/b').anchor == '' (relative has no anchor)"""
import pathlib

Path = pathlib.Path

assert Path("/a/b").anchor == "/", f"anchor = {Path('/a/b').anchor!r}"
assert Path("a/b").anchor == "", f"relative anchor = {Path('a/b').anchor!r}"
print("anchor_absolute_vs_relative OK")
