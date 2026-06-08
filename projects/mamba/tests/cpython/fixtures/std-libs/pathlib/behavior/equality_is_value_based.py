# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "equality_is_value_based"
# subject = "pathlib.Path"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.Path: Path equality is value-based: Path('/tmp/a')==Path('/tmp/a') and Path('/tmp/a')!=Path('/tmp/b')"""
import pathlib

Path = pathlib.Path

assert Path("/tmp/a") == Path("/tmp/a"), "path equality"
assert Path("/tmp/a") != Path("/tmp/b"), "path inequality"
print("equality_is_value_based OK")
