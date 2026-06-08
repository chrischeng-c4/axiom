# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "behavior"
# case = "writestr_with_zipinfo_metadata"
# subject = "zipfile.ZipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile.ZipFile: writestr accepts a ZipInfo argument carrying metadata (compress_type) and the named member reads back its content"""
import zipfile
import io

_buf = io.BytesIO()
_zi = zipfile.ZipInfo("meta.txt")
_zi.compress_type = zipfile.ZIP_STORED
with zipfile.ZipFile(_buf, "w") as _zf:
    _zf.writestr(_zi, b"metadata content")

_buf.seek(0)
with zipfile.ZipFile(_buf, "r") as _zf:
    assert _zf.read("meta.txt") == b"metadata content", "writestr with ZipInfo"

print("writestr_with_zipinfo_metadata OK")
