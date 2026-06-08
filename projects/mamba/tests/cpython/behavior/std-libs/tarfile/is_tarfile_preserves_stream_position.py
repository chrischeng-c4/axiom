# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "is_tarfile_preserves_stream_position"
# subject = "tarfile.is_tarfile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.is_tarfile: is_tarfile returns True for a valid in-memory tar stream and leaves the file-like object's seek position at 0 (untouched)"""
import tarfile
import io

_buf = io.BytesIO()
with tarfile.open(fileobj=_buf, mode="w") as _tf:
    _ti = tarfile.TarInfo("b.txt")
    _ti.size = 1
    _tf.addfile(_ti, io.BytesIO(b"y"))

_flo = io.BytesIO(_buf.getvalue())
_flo.seek(0)
assert tarfile.is_tarfile(_flo), "is_tarfile valid stream"
assert _flo.tell() == 0, f"position preserved = {_flo.tell()!r}"

print("is_tarfile_preserves_stream_position OK")
