# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "errors"
# case = "memory_bio_write_rejects_non_buffer"
# subject = "ssl.MemoryBIO.write"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.MemoryBIO.write: MemoryBIO.write rejects non-bytes-like inputs (str, None, bool, int) with TypeError and a non-contiguous writable memoryview with BufferError"""
import ssl

_t = ssl.MemoryBIO()
for _bad in ("foo", None, True, 1):
    try:
        _t.write(_bad)
        raise AssertionError(f"write({_bad!r}) should raise")
    except TypeError:
        pass

# Non-contiguous writable memoryview is rejected with BufferError.
_m = memoryview(bytearray(b"noncontig"))[::-2]
try:
    _t.write(memoryview(_m))
    raise AssertionError("non-contiguous buffer should raise")
except BufferError:
    pass

print("memory_bio_write_rejects_non_buffer OK")
