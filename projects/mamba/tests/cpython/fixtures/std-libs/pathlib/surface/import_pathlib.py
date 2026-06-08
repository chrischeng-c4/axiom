# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "surface"
# case = "import_pathlib"
# subject = "pathlib"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""pathlib: import_pathlib (surface)."""
import pathlib

assert hasattr(pathlib, "Path")
print("import_pathlib OK")
