# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "behavior"
# case = "gzipfile_write_returns_count_and_tell_tracks"
# subject = "gzip.GzipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""gzip.GzipFile: GzipFile.write() accepts any buffer (array.array) and returns the byte count while tell() tracks the uncompressed position"""
import array
import gzip
import io

# write() accepts any buffer (e.g. array.array) and returns the byte
# count; tell() tracks the uncompressed position.
_q = array.array("Q", [1, 2, 3, 4, 5])
_nbytes = len(_q) * _q.itemsize
with gzip.GzipFile(fileobj=io.BytesIO(), mode="w") as _wb:
    assert _wb.write(_q) == _nbytes, f"write returned {_wb.write!r}"
    assert _wb.tell() == _nbytes, f"tell = {_wb.tell()!r}"

print("gzipfile_write_returns_count_and_tell_tracks OK")
