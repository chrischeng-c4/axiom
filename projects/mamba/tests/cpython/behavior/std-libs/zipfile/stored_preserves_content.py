# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "behavior"
# case = "stored_preserves_content"
# subject = "zipfile.ZipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile.ZipFile: ZIP_STORED writes no compression: file_size equals compress_size and the content round-trips byte-for-byte"""
import zipfile
import io

_data = b"Hello, World! This is test data."
_buf = io.BytesIO()
with zipfile.ZipFile(_buf, "w", compression=zipfile.ZIP_STORED) as _zf:
    _zf.writestr("stored.txt", _data)

_buf.seek(0)
with zipfile.ZipFile(_buf, "r") as _zf:
    _info = _zf.getinfo("stored.txt")
    assert _info.file_size == _info.compress_size, "STORED: file_size == compress_size"
    assert _zf.read("stored.txt") == _data, "STORED: content matches"

print("stored_preserves_content OK")
