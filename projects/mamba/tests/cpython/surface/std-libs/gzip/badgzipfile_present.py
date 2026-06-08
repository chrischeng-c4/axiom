# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "surface"
# case = "badgzipfile_present"
# subject = "gzip"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""gzip: badgzipfile_present (surface)."""
import gzip

assert hasattr(gzip, "BadGzipFile")
print("badgzipfile_present OK")
