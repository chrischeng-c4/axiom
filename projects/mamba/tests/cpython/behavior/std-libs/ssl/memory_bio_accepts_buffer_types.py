# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "memory_bio_accepts_buffer_types"
# subject = "ssl.MemoryBIO.write"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.MemoryBIO.write: MemoryBIO.write accepts bytes-like buffers (bytearray, memoryview) and reads them back as concatenated bytes"""
import ssl

_t = ssl.MemoryBIO()
_t.write(bytearray(b"ba"))
_t.write(memoryview(b"r"))
assert _t.read() == b"bar", "bytearray and memoryview accepted"

print("memory_bio_accepts_buffer_types OK")
