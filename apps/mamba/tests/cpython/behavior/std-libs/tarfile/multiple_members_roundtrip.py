# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "multiple_members_roundtrip"
# subject = "tarfile.TarFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.TarFile: three distinct members written into one archive are each retrieved with their own content intact via extractfile"""
import tarfile
import io

_files = {"a.txt": b"alpha", "b.txt": b"beta", "c.txt": b"gamma"}

_buf = io.BytesIO()
with tarfile.open(fileobj=_buf, mode="w") as _tf:
    for _name, _content in _files.items():
        _ti = tarfile.TarInfo(_name)
        _ti.size = len(_content)
        _tf.addfile(_ti, io.BytesIO(_content))
_buf.seek(0)
with tarfile.open(fileobj=_buf, mode="r") as _tf:
    for _name, _content in _files.items():
        _fh = _tf.extractfile(_name)
        assert _fh is not None, f"extractfile {_name}"
        assert _fh.read() == _content, f"content mismatch: {_name}"

print("multiple_members_roundtrip OK")
