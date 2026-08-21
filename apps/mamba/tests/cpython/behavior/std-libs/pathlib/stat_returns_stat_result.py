# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "stat_returns_stat_result"
# subject = "pathlib.Path"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.Path: Path('.').stat() returns a stat result exposing st_size and st_mtime attributes"""
import pathlib

Path = pathlib.Path

_stat = Path(".").stat()
assert hasattr(_stat, "st_size"), "stat has st_size"
assert hasattr(_stat, "st_mtime"), "stat has st_mtime"
print("stat_returns_stat_result OK")
