# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "behavior"
# case = "gzipfile_write_seek_write_doubles"
# subject = "gzip.GzipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""gzip.GzipFile: write, seek forward by the data length, then write again: the closed stream decompresses to the message repeated twice"""
import gzip
import io

# write, seek forward by the data length, write again: the stream
# decompresses to the message repeated twice.
_msg = b"important message here."
_sink = io.BytesIO()
with gzip.GzipFile(fileobj=_sink, mode="w") as _sw:
    _sw.write(_msg)
    _sw.seek(len(_msg))
    _sw.write(_msg)
assert gzip.decompress(_sink.getvalue()) == _msg * 2, "write/seek/write"

print("gzipfile_write_seek_write_doubles OK")
