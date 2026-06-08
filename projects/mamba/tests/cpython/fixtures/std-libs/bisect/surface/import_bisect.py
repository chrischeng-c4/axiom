# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bisect"
# dimension = "surface"
# case = "import_bisect"
# subject = "bisect"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bisect: import_bisect (surface)."""
import bisect

assert hasattr(bisect, "bisect_left")
print("import_bisect OK")
