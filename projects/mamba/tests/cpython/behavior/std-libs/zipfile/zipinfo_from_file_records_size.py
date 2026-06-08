# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "behavior"
# case = "zipinfo_from_file_records_size"
# subject = "zipfile.ZipInfo"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zipfile"
# status = "filled"
# ///
"""zipfile.ZipInfo: ZipInfo.from_file on a real file records its byte size, applies the supplied arcname, and is_dir() is False"""
import zipfile
import os
import posixpath
import tempfile

with tempfile.TemporaryDirectory() as _td:
    _fpath = os.path.join(_td, "payload.bin")
    with open(_fpath, "wb") as _f:
        _f.write(b"0123456789")
    _zi = zipfile.ZipInfo.from_file(_fpath, "renamed")
    assert posixpath.basename(_zi.filename) == "renamed", \
        f"from_file arcname = {_zi.filename!r}"
    assert not _zi.is_dir(), "file is_dir() == False"
    assert _zi.file_size == 10, f"from_file size = {_zi.file_size!r}"

print("zipinfo_from_file_records_size OK")
