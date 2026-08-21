# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "tar_and_data_filters_preserve_name_type"
# subject = "tarfile.data_filter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.data_filter: tar_filter and data_filter return sanitized copies that preserve a safe member's name and type"""
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
        _t = tarfile.tar_filter(_m, "")
        assert _t.name == _m.name, f"tar_filter name {_t.name!r}"
        assert _t.type == _m.type, f"tar_filter type {_t.type!r}"
        _data = tarfile.data_filter(_m, "")
        assert _data.name == _m.name, f"data_filter name {_data.name!r}"
        assert _data.type == _m.type, f"data_filter type {_data.type!r}"

print("tar_and_data_filters_preserve_name_type OK")
