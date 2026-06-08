# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "fully_trusted_filter_is_identity"
# subject = "tarfile.fully_trusted_filter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.fully_trusted_filter: fully_trusted_filter returns the same TarInfo object it was given (identity), applying no sanitization"""
import tarfile
import io

# Build a small in-memory archive: a file and a directory.
_buf = io.BytesIO()
with tarfile.open(fileobj=_buf, mode="w") as _tf:
    _f = tarfile.TarInfo("safe.txt")
    _f.size = 3
    _tf.addfile(_f, io.BytesIO(b"abc"))
    _d = tarfile.TarInfo("dir/")
    _d.type = tarfile.DIRTYPE
    _tf.addfile(_d)

_buf.seek(0)
with tarfile.open(fileobj=_buf, mode="r") as _tf:
    for _m in _tf.getmembers():
        _out = tarfile.fully_trusted_filter(_m, "")
        assert _out is _m, f"fully_trusted not identity for {_m.name!r}"

print("fully_trusted_filter_is_identity OK")
