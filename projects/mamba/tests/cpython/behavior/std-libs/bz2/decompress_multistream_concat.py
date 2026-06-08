# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "behavior"
# case = "decompress_multistream_concat"
# subject = "bz2.decompress"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.decompress: bz2.decompress fully consumes two concatenated bzip2 streams into the joined payload"""
import bz2

two = bz2.compress(b"AAAA") + bz2.compress(b"BBBB")
assert bz2.decompress(two) == b"AAAABBBB", "multi-stream decompress"
print("decompress_multistream_concat OK")
