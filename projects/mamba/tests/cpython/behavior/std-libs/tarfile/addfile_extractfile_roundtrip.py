# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "addfile_extractfile_roundtrip"
# subject = "tarfile.TarFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.TarFile: content added via addfile is retrieved byte-for-byte via extractfile after a write -> seek(0) -> read round-trip"""
import tarfile
import io

_buf = io.BytesIO()
_data = b"Round-trip test data for tarfile."
with tarfile.open(fileobj=_buf, mode="w") as _tf:
    _ti = tarfile.TarInfo("roundtrip.txt")
    _ti.size = len(_data)
    _tf.addfile(_ti, io.BytesIO(_data))
_buf.seek(0)
with tarfile.open(fileobj=_buf, mode="r") as _tf:
    _fh = _tf.extractfile("roundtrip.txt")
    assert _fh is not None, "extractfile not None"
    assert _fh.read() == _data, "round-trip data"

print("addfile_extractfile_roundtrip OK")
