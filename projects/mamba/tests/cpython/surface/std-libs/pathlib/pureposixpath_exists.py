# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "surface"
# case = "pureposixpath_exists"
# subject = "pathlib"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""pathlib: pureposixpath_exists (surface)."""
import pathlib

assert hasattr(pathlib, "PurePosixPath")
print("pureposixpath_exists OK")
