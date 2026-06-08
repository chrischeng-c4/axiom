# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "slash_join_returns_path"
# subject = "pathlib.Path"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.Path: the / operator joins components producing a new Path: Path('/usr')/'local'/'bin' == Path('/usr/local/bin') and the result is a Path instance"""
import pathlib

Path = pathlib.Path

_base = Path("/usr")
_full = _base / "local" / "bin"
assert _full == Path("/usr/local/bin"), f"join = {_full!r}"
assert isinstance(_full, Path), "join returns Path"
print("slash_join_returns_path OK")
