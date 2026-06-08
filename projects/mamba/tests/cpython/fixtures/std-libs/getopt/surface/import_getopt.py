# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getopt"
# dimension = "surface"
# case = "import_getopt"
# subject = "getopt"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""getopt: import_getopt (surface)."""
import getopt

assert hasattr(getopt, "getopt")
print("import_getopt OK")
