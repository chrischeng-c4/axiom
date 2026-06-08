# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "atexit"
# dimension = "surface"
# case = "import_atexit"
# subject = "atexit"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""atexit: import_atexit (surface)."""
import atexit

assert hasattr(atexit, "register")
print("import_atexit OK")
