# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "behavior"
# case = "non_ascii_filename_roundtrip"
# subject = "zipfile.ZipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile.ZipFile: filenames are stored as str; a non-ASCII name round-trips and keeps its insertion order on reopen"""
import zipfile
import io

_buf = io.BytesIO()
with zipfile.ZipFile(_buf, "w") as _zf:
    _zf.writestr("foo.txt", "ascii name")
    _zf.writestr("ö.txt", "unicode name")
    assert isinstance(_zf.infolist()[0].filename, str), "filename is str"

_buf.seek(0)
with zipfile.ZipFile(_buf, "r") as _zf:
    assert _zf.filelist[0].filename == "foo.txt", f"name 0 = {_zf.filelist[0].filename!r}"
    assert _zf.filelist[1].filename == "ö.txt", f"name 1 = {_zf.filelist[1].filename!r}"

print("non_ascii_filename_roundtrip OK")
