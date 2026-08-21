# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "surface"
# case = "path_is_callable"
# subject = "pathlib.Path"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""pathlib.Path: path_is_callable (surface)."""
import pathlib

assert callable(pathlib.Path)
print("path_is_callable OK")
