# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "behavior"
# case = "extractall_to_disk"
# subject = "zipfile.ZipFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zipfile.ZipFile: extractall writes every member (including a nested subdir) to a temp directory on disk with the correct content"""
import zipfile
import io
import os
import tempfile

with tempfile.TemporaryDirectory() as _tmpdir:
    _buf = io.BytesIO()
    with zipfile.ZipFile(_buf, "w") as _zf:
        _zf.writestr("file1.txt", b"content1")
        _zf.writestr("subdir/file2.txt", b"content2")
    _buf.seek(0)
    with zipfile.ZipFile(_buf, "r") as _zf:
        _zf.extractall(_tmpdir)
    assert os.path.exists(os.path.join(_tmpdir, "file1.txt")), "file1.txt extracted"
    assert os.path.exists(os.path.join(_tmpdir, "subdir", "file2.txt")), \
        "subdir/file2.txt extracted"
    with open(os.path.join(_tmpdir, "file1.txt"), "rb") as _f:
        assert _f.read() == b"content1", "extracted content"

print("extractall_to_disk OK")
