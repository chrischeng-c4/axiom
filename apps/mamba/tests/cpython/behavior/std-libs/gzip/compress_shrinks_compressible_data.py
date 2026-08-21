# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "behavior"
# case = "compress_shrinks_compressible_data"
# subject = "gzip.compress"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""gzip.compress: compressing highly repetitive data at level 6 yields a stream strictly smaller than the uncompressed input"""
import gzip

_compressible = b"hello " * 500
_comp = gzip.compress(_compressible, compresslevel=6)
assert len(_comp) < len(_compressible), "compressed < uncompressed"

print("compress_shrinks_compressible_data OK")
