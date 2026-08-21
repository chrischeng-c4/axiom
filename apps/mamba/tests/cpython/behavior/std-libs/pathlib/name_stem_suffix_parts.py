# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "name_stem_suffix_parts"
# subject = "pathlib.Path"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.Path: Path('/tmp/test/file.txt') exposes name='file.txt', stem='file', suffix='.txt', parent=Path('/tmp/test'), parts=('/','a','b','c'), and str() round-trips"""
import pathlib

Path = pathlib.Path

_p = Path("/tmp/test/file.txt")
assert isinstance(_p, Path), f"Path type = {type(_p)!r}"
assert _p.name == "file.txt", f"name = {_p.name!r}"
assert _p.stem == "file", f"stem = {_p.stem!r}"
assert _p.suffix == ".txt", f"suffix = {_p.suffix!r}"
assert _p.parent == Path("/tmp/test"), f"parent = {_p.parent!r}"

_parts = Path("/a/b/c").parts
assert _parts == ("/", "a", "b", "c"), f"parts = {_parts!r}"

assert str(Path("/tmp/foo")) == "/tmp/foo", f"str = {str(Path('/tmp/foo'))!r}"

print("name_stem_suffix_parts OK")
