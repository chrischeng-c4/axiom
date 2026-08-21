# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "surface"
# case = "path_home_is_callable"
# subject = "pathlib.Path.home"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""pathlib.Path.home: path_home_is_callable (surface)."""
import pathlib

assert callable(pathlib.Path.home)
print("path_home_is_callable OK")
