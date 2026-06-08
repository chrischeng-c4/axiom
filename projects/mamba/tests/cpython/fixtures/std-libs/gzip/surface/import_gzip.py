# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "surface"
# case = "import_gzip"
# subject = "gzip"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""gzip: import_gzip (surface)."""
import gzip

assert hasattr(gzip, "compress")
print("import_gzip OK")
