# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "surface"
# case = "purewindowspath_exists"
# subject = "pathlib"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""pathlib: purewindowspath_exists (surface)."""
import pathlib

assert hasattr(pathlib, "PureWindowsPath")
print("purewindowspath_exists OK")
