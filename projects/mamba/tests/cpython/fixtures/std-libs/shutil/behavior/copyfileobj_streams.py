# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "behavior"
# case = "copyfileobj_streams"
# subject = "shutil.copyfileobj"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
"""shutil.copyfileobj: copyfileobj copies bytes from one file-like object to another; an io.BytesIO source content lands verbatim in the destination BytesIO"""
import shutil
import io

src_io = io.BytesIO(b"stream data")
dst_io = io.BytesIO()
shutil.copyfileobj(src_io, dst_io)
assert dst_io.getvalue() == b"stream data", f"copyfileobj = {dst_io.getvalue()!r}"

print("copyfileobj_streams OK")
