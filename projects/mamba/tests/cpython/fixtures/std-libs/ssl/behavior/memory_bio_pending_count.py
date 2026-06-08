# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "memory_bio_pending_count"
# subject = "ssl.MemoryBIO.pending"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.MemoryBIO.pending: MemoryBIO.pending tracks unread bytes exactly: 0 when empty, 3 after writing 3 bytes, 2 after reading 1"""
import ssl

_p = ssl.MemoryBIO()
assert _p.pending == 0, "empty BIO pending 0"
_p.write(b"foo")
assert _p.pending == 3, "pending counts written bytes"
_p.read(1)
assert _p.pending == 2, "pending drops as read"

print("memory_bio_pending_count OK")
