# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "surface"
# case = "windowspath_exists"
# subject = "pathlib"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""pathlib: windowspath_exists (surface)."""
import pathlib

assert hasattr(pathlib, "WindowsPath")
print("windowspath_exists OK")
