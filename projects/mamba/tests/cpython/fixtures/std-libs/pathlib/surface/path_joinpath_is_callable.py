# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "surface"
# case = "path_joinpath_is_callable"
# subject = "pathlib.Path.joinpath"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""pathlib.Path.joinpath: path_joinpath_is_callable (surface)."""
import pathlib

assert callable(pathlib.Path.joinpath)
print("path_joinpath_is_callable OK")
