# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "behavior"
# case = "decompressor_eof_transitions"
# subject = "bz2.BZ2Decompressor"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.BZ2Decompressor: BZ2Decompressor.eof is False initially and True after a complete stream is decompressed"""
import bz2

decomp = bz2.BZ2Decompressor()
assert decomp.eof is False, "eof False initially"
small = bz2.compress(b"small")
decomp.decompress(small)
assert decomp.eof is True, "eof True after decompression"
print("decompressor_eof_transitions OK")
