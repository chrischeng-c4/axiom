# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "errors"
# case = "empty_file_raises_badzipfile"
# subject = "zipfile.ZipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile.ZipFile: opening a zero-byte file in read mode raises BadZipFile"""
import zipfile
import os
import tempfile

with tempfile.TemporaryDirectory() as _td:
    _empty = os.path.join(_td, "empty.zip")
    open(_empty, "wb").close()
    _raised = False
    try:
        zipfile.ZipFile(_empty)
    except zipfile.BadZipFile:
        _raised = True
    assert _raised, "empty file -> BadZipFile"

print("empty_file_raises_badzipfile OK")
