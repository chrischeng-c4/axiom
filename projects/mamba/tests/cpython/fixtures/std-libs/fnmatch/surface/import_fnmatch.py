# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "surface"
# case = "import_fnmatch"
# subject = "fnmatch"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fnmatch: import_fnmatch (surface)."""
import fnmatch

assert hasattr(fnmatch, "fnmatch")
print("import_fnmatch OK")
