# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "surface"
# case = "posixpath_exists"
# subject = "pathlib"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""pathlib: posixpath_exists (surface)."""
import pathlib

assert hasattr(pathlib, "PosixPath")
print("posixpath_exists OK")
