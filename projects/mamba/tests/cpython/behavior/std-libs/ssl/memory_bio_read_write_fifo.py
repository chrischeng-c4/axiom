# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "memory_bio_read_write_fifo"
# subject = "ssl.MemoryBIO"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.MemoryBIO: MemoryBIO is a byte FIFO: successive writes concatenate, a full read drains it, and sized reads take prefixes leaving the remainder"""
import ssl

_bio = ssl.MemoryBIO()
_bio.write(b"foo")
_bio.write(b"bar")
assert _bio.read() == b"foobar", "FIFO concatenates writes"
assert _bio.read() == b"", "drained BIO reads empty"
_bio.write(b"baz")
assert _bio.read(2) == b"ba", "sized read takes prefix"
assert _bio.read(1) == b"z", "sized read takes remainder"
assert _bio.read(1) == b"", "empty after drain"

print("memory_bio_read_write_fifo OK")
