# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "behavior"
# case = "namelist_read_roundtrip"
# subject = "zipfile.ZipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile.ZipFile: writestr two members to an in-memory archive, reopen, and confirm namelist contains both names, read returns the exact bytes, and open() yields a file-like object with the same content"""
import zipfile
import io

_buf = io.BytesIO()
with zipfile.ZipFile(_buf, "w", compression=zipfile.ZIP_STORED) as _zf:
    _zf.writestr("hello.txt", "Hello, World!")
    _zf.writestr("nested/data.txt", "nested content")

_buf.seek(0)
with zipfile.ZipFile(_buf, "r") as _zf:
    _names = _zf.namelist()
    assert isinstance(_names, list), f"namelist type = {type(_names)!r}"
    assert "hello.txt" in _names, f"hello.txt in namelist = {_names!r}"
    assert "nested/data.txt" in _names, "nested in namelist"

    _data = _zf.read("hello.txt")
    assert isinstance(_data, bytes), f"read type = {type(_data)!r}"
    assert _data == b"Hello, World!", f"read data = {_data!r}"

    _infos = _zf.infolist()
    assert isinstance(_infos, list), f"infolist type = {type(_infos)!r}"
    assert len(_infos) == 2, f"infolist len = {len(_infos)!r}"

    with _zf.open("hello.txt") as _fh:
        assert _fh.read() == b"Hello, World!", "open+read content"

print("namelist_read_roundtrip OK")
