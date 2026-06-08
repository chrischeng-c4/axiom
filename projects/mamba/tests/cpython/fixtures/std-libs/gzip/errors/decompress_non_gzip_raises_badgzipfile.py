# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "errors"
# case = "decompress_non_gzip_raises_badgzipfile"
# subject = "gzip.decompress"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_gzip.py"
# status = "filled"
# ///
"""gzip.decompress: decompress_non_gzip_raises_badgzipfile (errors)."""
import gzip

_raised = False
try:
    gzip.decompress(b'not a gzip stream')
except gzip.BadGzipFile:
    _raised = True
assert _raised, "decompress_non_gzip_raises_badgzipfile: expected gzip.BadGzipFile"
print("decompress_non_gzip_raises_badgzipfile OK")
