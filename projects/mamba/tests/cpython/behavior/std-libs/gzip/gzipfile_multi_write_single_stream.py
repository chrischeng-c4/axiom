# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "behavior"
# case = "gzipfile_multi_write_single_stream"
# subject = "gzip.GzipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""gzip.GzipFile: two successive writes to one GzipFile concatenate into a single decompressed stream"""
import gzip
import io

_buf = io.BytesIO()
with gzip.GzipFile(fileobj=_buf, mode="wb") as _w:
    _w.write(b"part1 ")
    _w.write(b"part2")
_buf.seek(0)
with gzip.GzipFile(fileobj=_buf, mode="rb") as _r:
    _out = _r.read()
assert _out == b"part1 part2", f"multi-write = {_out!r}"

print("gzipfile_multi_write_single_stream OK")
