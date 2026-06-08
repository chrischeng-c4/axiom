# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "resolve_dot_is_cwd"
# subject = "pathlib.Path"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.Path: Path('.').resolve() returns an absolute path equal to Path.cwd()"""
import pathlib

Path = pathlib.Path

_resolved = Path(".").resolve()
assert _resolved.is_absolute(), f"resolve is absolute: {_resolved!r}"
assert _resolved == Path.cwd(), "resolve('.') == cwd"
print("resolve_dot_is_cwd OK")
