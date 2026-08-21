# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "stringio_read_n_caps_chars"
# subject = "io.StringIO"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""io.StringIO: read(n) returns at most n characters from the current position"""
import io

_buf = io.StringIO("hello world")
assert _buf.read(5) == "hello", "read(5) caps at 5 chars"

print("stringio_read_n_caps_chars OK")
