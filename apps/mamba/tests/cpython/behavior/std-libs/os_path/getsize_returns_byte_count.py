# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "behavior"
# case = "getsize_returns_byte_count"
# subject = "os.path.getsize"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
"""os.path.getsize: getsize returns the file's byte length; an 11-byte payload written to a temp file reports getsize == 11"""
import os
import os.path
import tempfile

with tempfile.NamedTemporaryFile(delete=False) as _ntf:
    _ntf.write(b"hello world")
    _ntfname = _ntf.name
try:
    _sz = os.path.getsize(_ntfname)
    assert _sz == 11, f"getsize = {_sz!r}"
finally:
    os.unlink(_ntfname)

print("getsize_returns_byte_count OK")
