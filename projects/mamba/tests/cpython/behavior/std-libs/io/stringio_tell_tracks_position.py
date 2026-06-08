# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "stringio_tell_tracks_position"
# subject = "io.StringIO"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""io.StringIO: tell() starts at 0, advances by characters read, and returns to 0 after seek(0)"""
import io

_t = io.StringIO("hello")
assert _t.tell() == 0, f"initial tell = {_t.tell()!r}"
_t.read(3)
assert _t.tell() == 3, f"after read(3) tell = {_t.tell()!r}"
_t.seek(0)
assert _t.tell() == 0, f"after seek(0) tell = {_t.tell()!r}"

print("stringio_tell_tracks_position OK")
