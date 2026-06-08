# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "surface"
# case = "path_cwd_is_callable"
# subject = "pathlib.Path.cwd"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""pathlib.Path.cwd: path_cwd_is_callable (surface)."""
import pathlib

assert callable(pathlib.Path.cwd)
print("path_cwd_is_callable OK")
