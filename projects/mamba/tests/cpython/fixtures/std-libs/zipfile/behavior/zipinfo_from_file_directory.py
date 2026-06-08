# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "behavior"
# case = "zipinfo_from_file_directory"
# subject = "zipfile.ZipInfo"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_zipfile"
# status = "filled"
# ///
"""zipfile.ZipInfo: ZipInfo.from_file on a directory appends a trailing slash to the arcname, marks is_dir() True, uses ZIP_STORED, and has zero file_size"""
import zipfile
import os
import tempfile

with tempfile.TemporaryDirectory() as _td:
    _sub = os.path.join(_td, "adir")
    os.mkdir(_sub)
    _zi = zipfile.ZipInfo.from_file(_sub, "stuff")
    assert _zi.filename == "stuff/", f"dir arcname = {_zi.filename!r}"
    assert _zi.is_dir(), "dir is_dir() == True"
    assert _zi.compress_type == zipfile.ZIP_STORED, "dir compress_type"
    assert _zi.file_size == 0, f"dir file_size = {_zi.file_size!r}"

print("zipinfo_from_file_directory OK")
