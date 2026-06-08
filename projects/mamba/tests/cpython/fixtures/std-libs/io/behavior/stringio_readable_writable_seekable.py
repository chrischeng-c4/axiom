# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "stringio_readable_writable_seekable"
# subject = "io.StringIO"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""io.StringIO: a StringIO reports readable(), writable(), and seekable() all True"""
import io

_r = io.StringIO("x")
assert _r.readable(), "StringIO readable"
assert _r.writable(), "StringIO writable"
assert _r.seekable(), "StringIO seekable"

print("stringio_readable_writable_seekable OK")
