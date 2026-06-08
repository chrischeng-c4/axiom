# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "behavior"
# case = "decompressor_max_length_bounded"
# subject = "bz2.BZ2Decompressor"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.BZ2Decompressor: max_length caps per-call output and needs_input/eof track progress while draining the rest"""
import bz2

blob = bz2.compress(b"x" * 1000)
decomp = bz2.BZ2Decompressor()
part = decomp.decompress(blob, max_length=10)
assert len(part) == 10, f"max_length cap = {len(part)}"
assert decomp.needs_input is False, "needs_input False while output pending"
assert decomp.eof is False, "not eof mid-stream"
rest = decomp.decompress(b"", max_length=-1)
assert part + rest == b"x" * 1000, "bounded reassembly"
assert decomp.eof is True, "eof after draining"
print("decompressor_max_length_bounded OK")
