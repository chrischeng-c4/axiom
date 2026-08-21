# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "import_sys"
# subject = "sys"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys: import_sys (surface)."""
import sys

assert hasattr(sys, "version")
print("import_sys OK")
