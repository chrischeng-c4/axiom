# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "behavior"
# case = "independent_member_positions"
# subject = "zipfile.ZipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zipfile"
# status = "filled"
# ///
"""zipfile.ZipFile: two members opened from one archive at the same time keep independent read positions"""
import zipfile
import io

_buf = io.BytesIO()
with zipfile.ZipFile(_buf, "w") as _zf:
    _zf.writestr("a.txt", "123")
    _zf.writestr("b.txt", "456")
_buf.seek(0)
with zipfile.ZipFile(_buf, "r") as _zf:
    with _zf.open("a.txt", "r") as _a, _zf.open("b.txt", "r") as _b:
        assert _a.read(1) == b"1", "a read 1"
        assert _b.seek(1) == 1, "b seek to 1"
        assert _b.read(1) == b"5", "b read after seek"
        assert _a.read(1) == b"2", "a keeps its own position"

print("independent_member_positions OK")
