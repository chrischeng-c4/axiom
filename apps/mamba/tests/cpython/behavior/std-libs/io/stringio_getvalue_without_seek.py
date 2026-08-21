# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "stringio_getvalue_without_seek"
# subject = "io.StringIO"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""io.StringIO: getvalue() returns written content without needing a prior seek(0)"""
import io

_buf = io.StringIO()
_buf.write("abc")
assert _buf.getvalue() == "abc", f"getvalue = {_buf.getvalue()!r}"

print("stringio_getvalue_without_seek OK")
