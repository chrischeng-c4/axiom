# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "relative_to_strips_prefix"
# subject = "pathlib.Path"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.Path: Path('/usr/local/bin/python').relative_to('/usr') == Path('local/bin/python')"""
import pathlib

Path = pathlib.Path

_abs = Path("/usr/local/bin/python")
_rel = _abs.relative_to("/usr")
assert _rel == Path("local/bin/python"), f"relative_to = {_rel!r}"
print("relative_to_strips_prefix OK")
