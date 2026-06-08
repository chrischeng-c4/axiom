# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "behavior"
# case = "gzipfile_capability_flags_and_name"
# subject = "gzip.GzipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""gzip.GzipFile: a GzipFile over a nameless fileobj reports name=='' , mode==WRITE/READ, the complementary readable/writable/seekable flags, closed transitions across the with-block, and fileno() raises UnsupportedOperation with no underlying fd"""
import gzip
import io

# A GzipFile wrapping a nameless fileobj reports name == "".
_buf = io.BytesIO()
with gzip.GzipFile(fileobj=_buf, mode="wb") as _w:
    _w.write(b"payload" * 20)
    assert _w.name == "", f"write name = {_w.name!r}"
    assert _w.mode == gzip.WRITE, f"write mode = {_w.mode!r}"
    assert _w.readable() is False, "writer not readable"
    assert _w.writable() is True, "writer writable"
    assert _w.seekable() is True, "writer seekable"
    assert _w.closed is False, "open writer not closed"
    # No underlying fd: fileno() raises while open.
    try:
        _w.fileno()
        raise AssertionError("expected UnsupportedOperation")
    except io.UnsupportedOperation:
        pass

# After the context exits the object reports closed.
assert _w.closed is True, "writer closed after with-block"

# Reading reports the complementary capability flags.
_buf.seek(0)
with gzip.GzipFile(fileobj=_buf, mode="rb") as _r:
    assert _r.read() == b"payload" * 20, "read round-trip"
    assert _r.mode == gzip.READ, f"read mode = {_r.mode!r}"
    assert _r.readable() is True, "reader readable"
    assert _r.writable() is False, "reader not writable"
    assert _r.seekable() is True, "reader seekable"

print("gzipfile_capability_flags_and_name OK")
