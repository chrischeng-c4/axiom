# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "errors"
# case = "read_missing_member_raises_keyerror"
# subject = "zipfile.ZipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile.ZipFile: reading a member that is not in the archive raises KeyError; open() of the same missing member also raises KeyError"""
import zipfile
import io

_buf = io.BytesIO()
with zipfile.ZipFile(_buf, "w") as _z:
    _z.writestr("a.txt", "hello")
_buf.seek(0)
with zipfile.ZipFile(_buf, "r") as _z:
    _read_raised = False
    try:
        _z.read("missing.txt")
    except KeyError:
        _read_raised = True
    assert _read_raised, "missing member read -> KeyError"

    _open_raised = False
    try:
        _z.open("missing.txt", "r")
    except KeyError:
        _open_raised = True
    assert _open_raised, "missing member open -> KeyError"

print("read_missing_member_raises_keyerror OK")
