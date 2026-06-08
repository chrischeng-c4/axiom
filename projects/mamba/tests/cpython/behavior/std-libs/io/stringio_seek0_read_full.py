# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "stringio_seek0_read_full"
# subject = "io.StringIO"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""io.StringIO: after writing then seek(0), read() returns the full buffer contents"""
import io

_buf = io.StringIO()
_buf.write("abcdef")
_buf.seek(0)
assert _buf.read() == "abcdef", "read after seek(0)"

print("stringio_seek0_read_full OK")
