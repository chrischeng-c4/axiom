# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "surface"
# case = "api_bad_gzip_file_is_present"
# subject = "gzip.BadGzipFile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""gzip.BadGzipFile: api_bad_gzip_file_is_present (surface)."""
import gzip

assert hasattr(gzip, "BadGzipFile")
print("api_bad_gzip_file_is_present OK")
