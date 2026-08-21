# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "behavior"
# case = "compress_mtime_zero_is_deterministic"
# subject = "gzip.compress"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""gzip.compress: two compress calls on the same data with mtime=0 produce byte-identical streams (no embedded wall-clock timestamp)"""
import gzip

_d3 = b"deterministic test"
_a = gzip.compress(_d3, mtime=0)
_b = gzip.compress(_d3, mtime=0)
assert _a == _b, "mtime=0 deterministic"

print("compress_mtime_zero_is_deterministic OK")
