# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "stringio_getvalue_ignores_position"
# subject = "io.StringIO"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""io.StringIO: getvalue() returns the entire buffer regardless of the current read/write position"""
import io

_buf = io.StringIO()
_buf.write("full")
_buf.seek(0)
_buf.read(2)
assert _buf.getvalue() == "full", f"getvalue mid-read = {_buf.getvalue()!r}"

print("stringio_getvalue_ignores_position OK")
