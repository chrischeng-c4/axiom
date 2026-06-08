# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "behavior"
# case = "append_mode_adds_members"
# subject = "zipfile.ZipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile.ZipFile: reopening an on-disk archive in 'a' mode adds a new member while preserving the existing one"""
import zipfile
import os
import tempfile

with tempfile.TemporaryDirectory() as _td:
    _zippath = os.path.join(_td, "archive.zip")
    with zipfile.ZipFile(_zippath, "w") as _zf:
        _zf.writestr("first.txt", b"first")
    with zipfile.ZipFile(_zippath, "a") as _zf:
        _zf.writestr("second.txt", b"second")
    with zipfile.ZipFile(_zippath, "r") as _zf:
        _names = _zf.namelist()
        assert "first.txt" in _names, f"first in names = {_names!r}"
        assert "second.txt" in _names, "second in names"
        assert _zf.read("first.txt") == b"first", "preserved first content"
        assert _zf.read("second.txt") == b"second", "appended second content"

print("append_mode_adds_members OK")
