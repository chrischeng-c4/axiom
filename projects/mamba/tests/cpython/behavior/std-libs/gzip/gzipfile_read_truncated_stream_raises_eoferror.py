# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "behavior"
# case = "gzipfile_read_truncated_stream_raises_eoferror"
# subject = "gzip.GzipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_gzip.py"
# status = "filled"
# ///
"""gzip.GzipFile: reading a stream missing its 8-byte trailer through GzipFile raises EOFError once the reader walks past the payload, while a sized read of exactly the payload length still recovers the bytes; header-only prefixes raise EOFError on first read"""
import gzip
import io

_data = b"line of repeated content\n" * 50
# Drop the 8-byte trailer (CRC32 + ISIZE) to truncate the stream.
_truncated = gzip.compress(_data)[:-8]

# Reading the whole stream walks past the missing trailer -> EOFError.
with gzip.GzipFile(fileobj=io.BytesIO(_truncated)) as _f:
    try:
        _f.read()
        raise AssertionError("expected EOFError on full read")
    except EOFError:
        pass

# Reading exactly the payload length succeeds; the next read hits the
# missing trailer and raises EOFError.
with gzip.GzipFile(fileobj=io.BytesIO(_truncated)) as _f:
    assert _f.read(len(_data)) == _data, "payload bytes recovered"
    try:
        _f.read(1)
        raise AssertionError("expected EOFError past payload")
    except EOFError:
        pass

# Streams truncated inside the header (only a few bytes) also raise
# EOFError on the first read.
for _n in range(2, 10):
    with gzip.GzipFile(fileobj=io.BytesIO(_truncated[:_n])) as _f:
        try:
            _f.read(1)
            raise AssertionError(f"expected EOFError at prefix {_n}")
        except EOFError:
            pass

print("gzipfile_read_truncated_stream_raises_eoferror OK")
