# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "behavior"
# case = "get_terminal_size_namedtuple_shape"
# subject = "shutil.get_terminal_size"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
"""shutil.get_terminal_size: get_terminal_size() returns a 2-field named tuple with integer .columns and .lines attributes; tuple(size) round-trips to (columns, lines)"""
import shutil

size = shutil.get_terminal_size()
assert hasattr(size, "columns"), "has columns"
assert hasattr(size, "lines"), "has lines"
assert isinstance(size.columns, int), f"columns type = {type(size.columns)!r}"
assert isinstance(size.lines, int), f"lines type = {type(size.lines)!r}"
# Named tuple of length 2, round-tripping to (columns, lines).
assert len(size) == 2, f"len = {len(size)}"
assert tuple(size) == (size.columns, size.lines), f"tuple = {tuple(size)!r}"

print("get_terminal_size_namedtuple_shape OK")
