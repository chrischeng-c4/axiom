# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "behavior"
# case = "decompressor_unused_data_trailing"
# subject = "bz2.BZ2Decompressor"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.BZ2Decompressor: trailing bytes after a complete stream are surfaced via unused_data, not decompressed"""
import bz2

data = bz2.compress(b"payload") + b"extra_trailing_bytes"
decomp = bz2.BZ2Decompressor()
result = decomp.decompress(data)
assert result == b"payload", f"payload = {result!r}"
assert decomp.unused_data == b"extra_trailing_bytes", f"unused = {decomp.unused_data!r}"
print("decompressor_unused_data_trailing OK")
