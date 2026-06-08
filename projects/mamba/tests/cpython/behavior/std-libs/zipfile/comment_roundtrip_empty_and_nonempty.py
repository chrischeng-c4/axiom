# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "behavior"
# case = "comment_roundtrip_empty_and_nonempty"
# subject = "zipfile.ZipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile.ZipFile: a bytes comment set on an empty (append) archive and on a non-empty archive both survive a close/reopen"""
import zipfile
import os
import tempfile

# A comment set on an empty (append-mode) archive survives a reopen.
with tempfile.TemporaryDirectory() as _td:
    _zp = os.path.join(_td, "empty.zip")
    with zipfile.ZipFile(_zp, "a", zipfile.ZIP_STORED) as _zf:
        assert not _zf.filelist, "fresh append archive has no entries"
        _zf.comment = b"this is a comment"
    with zipfile.ZipFile(_zp, "r") as _zf:
        assert _zf.comment == b"this is a comment", f"empty-archive comment = {_zf.comment!r}"

# A comment set on a non-empty archive survives a reopen.
with tempfile.TemporaryDirectory() as _td:
    _zp = os.path.join(_td, "data.zip")
    with zipfile.ZipFile(_zp, "w", zipfile.ZIP_STORED) as _zf:
        _zf.writestr("foo.txt", "O, for a Muse of Fire!")
    with zipfile.ZipFile(_zp, "a", zipfile.ZIP_STORED) as _zf:
        assert _zf.filelist, "non-empty archive has entries"
        _zf.comment = b"trailing comment"
    with zipfile.ZipFile(_zp, "r") as _zf:
        assert _zf.comment == b"trailing comment", f"nonempty-archive comment = {_zf.comment!r}"

print("comment_roundtrip_empty_and_nonempty OK")
