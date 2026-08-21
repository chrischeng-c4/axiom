# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "memory_bio_eof_after_drain"
# subject = "ssl.MemoryBIO.eof"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.MemoryBIO.eof: MemoryBIO.eof flips True only once the buffer is fully drained AND write_eof() was called; it stays False while bytes remain pending"""
import ssl

_e = ssl.MemoryBIO()
assert _e.eof is False, "fresh BIO not eof"
_e.write(b"fo")
_e.write_eof()
assert _e.eof is False, "eof stays False while bytes pending"
assert _e.read(1) == b"f", "read before eof"
assert _e.eof is False, "still bytes left"
assert _e.read(1) == b"o", "read last byte"
assert _e.eof is True, "eof True once drained after write_eof"
assert _e.read() == b"", "reading past eof yields empty"

print("memory_bio_eof_after_drain OK")
