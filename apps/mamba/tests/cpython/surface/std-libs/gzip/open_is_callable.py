# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "surface"
# case = "open_is_callable"
# subject = "gzip.open"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""gzip.open: open_is_callable (surface)."""
import gzip

assert callable(gzip.open)
print("open_is_callable OK")
