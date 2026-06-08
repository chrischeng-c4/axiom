# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "behavior"
# case = "member_seek_tell_contract"
# subject = "zipfile.ZipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zipfile"
# status = "filled"
# ///
"""zipfile.ZipFile: an opened member supports SEEK_SET/SEEK_CUR/SEEK_END seeks with tell() tracking the offset, and read after seeking returns the bytes at that position"""
import zipfile
import io
import os

_txt = b"Where's Bruce?"
_bloc = _txt.find(b"Bruce")

_buf = io.BytesIO()
with zipfile.ZipFile(_buf, "w") as _zf:
    _zf.writestr("foo.txt", _txt)
_buf.seek(0)
with zipfile.ZipFile(_buf, "r") as _zf:
    with _zf.open("foo.txt", "r") as _fp:
        _fp.seek(_bloc, os.SEEK_SET)
        assert _fp.tell() == _bloc, f"after SEEK_SET tell = {_fp.tell()!r}"
        _fp.seek(-_bloc, os.SEEK_CUR)
        assert _fp.tell() == 0, f"after relative back tell = {_fp.tell()!r}"
        _fp.seek(_bloc, os.SEEK_CUR)
        assert _fp.tell() == _bloc, f"after relative fwd tell = {_fp.tell()!r}"
        assert _fp.read(5) == _txt[_bloc:_bloc + 5], "read 5 from seeked pos"
        assert _fp.tell() == _bloc + 5, f"tell after read = {_fp.tell()!r}"
        _fp.seek(0, os.SEEK_END)
        assert _fp.tell() == len(_txt), f"SEEK_END tell = {_fp.tell()!r}"
        _fp.seek(0, os.SEEK_SET)
        assert _fp.tell() == 0, "rewind to 0"

print("member_seek_tell_contract OK")
