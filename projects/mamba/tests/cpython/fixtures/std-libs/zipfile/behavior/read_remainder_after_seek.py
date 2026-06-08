# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "behavior"
# case = "read_remainder_after_seek"
# subject = "zipfile.ZipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zipfile"
# status = "filled"
# ///
"""zipfile.ZipFile: read(-1) after a relative SEEK_CUR returns the remainder of the member from the seeked position"""
import zipfile
import io
import os

_charge = b"Charge men!"
_cloc = _charge.find(b"men")

_buf = io.BytesIO()
with zipfile.ZipFile(_buf, "w") as _zf:
    _zf.writestr("c.txt", _charge)
_buf.seek(0)
with zipfile.ZipFile(_buf, "r") as _zf:
    with _zf.open("c.txt", "r") as _fp:
        _fp.seek(_cloc, os.SEEK_CUR)
        assert _fp.read(-1) == b"men!", "read(-1) after seek"

print("read_remainder_after_seek OK")
