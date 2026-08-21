# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "surface"
# case = "import_linecache"
# subject = "linecache"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""linecache: import_linecache (surface)."""
import linecache

assert hasattr(linecache, "getline")
print("import_linecache OK")
