# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "behavior"
# case = "badgzipfile_is_oserror_subclass"
# subject = "gzip.BadGzipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_gzip.py"
# status = "filled"
# ///
"""gzip.BadGzipFile: gzip.BadGzipFile is a subclass of OSError (the exception hierarchy contract on Python 3.12+)"""
import gzip

assert issubclass(gzip.BadGzipFile, OSError), "BadGzipFile is an OSError subclass"

print("badgzipfile_is_oserror_subclass OK")
