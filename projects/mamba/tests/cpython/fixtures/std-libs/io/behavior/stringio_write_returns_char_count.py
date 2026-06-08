# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "stringio_write_returns_char_count"
# subject = "io.StringIO"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""io.StringIO: StringIO.write returns the number of characters written (write('hello') -> 5)"""
import io

_buf = io.StringIO()
_n = _buf.write("hello")
assert _n == 5, f"write returns char count = {_n!r}"

print("stringio_write_returns_char_count OK")
