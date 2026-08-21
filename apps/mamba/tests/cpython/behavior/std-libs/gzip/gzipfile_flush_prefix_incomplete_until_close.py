# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "behavior"
# case = "gzipfile_flush_prefix_incomplete_until_close"
# subject = "gzip.GzipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""gzip.GzipFile: flush() commits a syncable prefix but the trailer is only written on close: the closed stream decompresses while the flushed-only prefix raises EOFError"""
import gzip
import io

# flush() on a writer commits a syncable prefix, but the trailer is only
# written on close. The fully closed stream decompresses; the flushed-
# but-not-closed prefix is incomplete and raises EOFError.
_sink = io.BytesIO()
_msg = b"flushed content"
with gzip.GzipFile(fileobj=_sink, mode="w") as _w:
    _w.write(_msg)
    _w.flush()
    _partial = _sink.getvalue()
_full = _sink.getvalue()

assert gzip.decompress(_full) == _msg, "closed stream decompresses"
try:
    gzip.decompress(_partial)
    raise AssertionError("expected EOFError on flushed-only prefix")
except EOFError:
    pass

print("gzipfile_flush_prefix_incomplete_until_close OK")
