# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "errors"
# case = "bad_open_modes_raise_valueerror"
# subject = "zipfile.ZipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile.ZipFile: ZipFile.open() rejects the bogus per-member modes 'q', 'U', 'rU' with ValueError"""
import zipfile
import io

_buf = io.BytesIO()
with zipfile.ZipFile(_buf, "w") as _z:
    _z.writestr("foo.txt", "data")
_buf.seek(0)
with zipfile.ZipFile(_buf, "r") as _z:
    for _mode in ("q", "U", "rU"):
        _raised = False
        try:
            _z.open("foo.txt", _mode)
        except ValueError:
            _raised = True
        assert _raised, f"open mode {_mode!r} -> ValueError"

print("bad_open_modes_raise_valueerror OK")
