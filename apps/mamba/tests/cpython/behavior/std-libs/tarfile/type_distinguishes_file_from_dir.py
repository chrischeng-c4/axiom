# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "type_distinguishes_file_from_dir"
# subject = "tarfile.TarFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.TarFile: a member tagged REGTYPE reads back as isreg() while a member tagged DIRTYPE reads back as isdir(), and TarInfo.type round-trips each constant"""
import tarfile
import io

_buf = io.BytesIO()
with tarfile.open(fileobj=_buf, mode="w") as _tf:
    _ti_file = tarfile.TarInfo("regular.txt")
    _ti_file.type = tarfile.REGTYPE
    _ti_file.size = 0
    _tf.addfile(_ti_file)
    _ti_dir = tarfile.TarInfo("mydir/")
    _ti_dir.type = tarfile.DIRTYPE
    _tf.addfile(_ti_dir)
_buf.seek(0)
with tarfile.open(fileobj=_buf, mode="r") as _tf:
    _reg = _tf.getmember("regular.txt")
    _dir = _tf.getmember("mydir/")
    assert _reg.type == tarfile.REGTYPE, f"regular type = {_reg.type!r}"
    assert _dir.type == tarfile.DIRTYPE, f"dir type = {_dir.type!r}"
    assert _reg.isreg(), "isreg for regular"
    assert _dir.isdir(), "isdir for directory"

print("type_distinguishes_file_from_dir OK")
