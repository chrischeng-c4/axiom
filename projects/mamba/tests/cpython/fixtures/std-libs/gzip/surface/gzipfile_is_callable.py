# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "surface"
# case = "gzipfile_is_callable"
# subject = "gzip.GzipFile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""gzip.GzipFile: gzipfile_is_callable (surface)."""
import gzip

assert callable(gzip.GzipFile)
print("gzipfile_is_callable OK")
