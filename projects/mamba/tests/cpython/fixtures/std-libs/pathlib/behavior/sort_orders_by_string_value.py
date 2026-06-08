# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "sort_orders_by_string_value"
# subject = "pathlib.Path"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.Path: sorting a list of Paths orders them by their string value: sorted([Path('/b'),Path('/a'),Path('/c')]) == [Path('/a'),Path('/b'),Path('/c')]"""
import pathlib

Path = pathlib.Path

_paths = [Path("/b"), Path("/a"), Path("/c")]
_sorted = sorted(_paths)
assert _sorted == [Path("/a"), Path("/b"), Path("/c")], f"sorted paths = {_sorted!r}"
print("sort_orders_by_string_value OK")
