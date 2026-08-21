# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "surface"
# case = "zipinfo_attribute_surface"
# subject = "zipfile.ZipInfo"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile.ZipInfo: a ZipInfo from getinfo exposes the documented attributes filename, file_size, compress_size, compress_type"""
import zipfile
import io

_buf = io.BytesIO()
with zipfile.ZipFile(_buf, "w") as _zf:
    _zf.writestr("hello.txt", "Hello, World!")
_buf.seek(0)
with zipfile.ZipFile(_buf, "r") as _zf:
    _info = _zf.getinfo("hello.txt")
    assert isinstance(_info, zipfile.ZipInfo), f"getinfo type = {type(_info)!r}"
    for _attr in ("filename", "file_size", "compress_size", "compress_type"):
        assert hasattr(_info, _attr), f"ZipInfo missing {_attr}"
    assert _info.filename == "hello.txt", f"filename = {_info.filename!r}"
    assert _info.file_size == 13, f"file_size = {_info.file_size!r}"

print("zipinfo_attribute_surface OK")
