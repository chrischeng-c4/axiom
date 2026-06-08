# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "behavior"
# case = "decompressor_eof_transitions"
# subject = "lzma.LZMADecompressor"
# kind = "semantic"
# xfail = "LZMADecompressor is a sentinel-string stub (src/runtime/stdlib/lzma_mod.rs:87-88)"
# mem_carveout = ""
# source = "Lib/test/test_lzma.py"
# status = "filled"
# ///
"""lzma.LZMADecompressor: LZMADecompressor.eof starts False and becomes True once a complete stream is decompressed"""
import lzma


decomp = lzma.LZMADecompressor()
assert decomp.eof is False, "eof False initially"
decomp.decompress(lzma.compress(b"small data"))
assert decomp.eof is True, "eof True after decompression"
print("decompressor_eof_transitions OK")
