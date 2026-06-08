# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "stringio_seek_end_positions_at_end"
# subject = "io.StringIO"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""io.StringIO: seek(0, SEEK_END) returns and positions at the byte/char length of the buffer"""
import io

_end = io.StringIO("abc")
_pos = _end.seek(0, io.SEEK_END)
assert _pos == 3, f"seek SEEK_END = {_pos!r}"

print("stringio_seek_end_positions_at_end OK")
