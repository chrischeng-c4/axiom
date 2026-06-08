# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "extractall_writes_tree_to_disk"
# subject = "tarfile.TarFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.TarFile: extractall(filter='data') materializes both a top-level file and a nested subdir/file onto a TemporaryDirectory with the original content"""
import tarfile
import io
import os
import tempfile

with tempfile.TemporaryDirectory() as _tmpdir:
    _buf = io.BytesIO()
    with tarfile.open(fileobj=_buf, mode="w") as _tf:
        for _fname, _fc in [("file1.txt", b"one"), ("subdir/file2.txt", b"two")]:
            _ti = tarfile.TarInfo(_fname)
            _ti.size = len(_fc)
            _tf.addfile(_ti, io.BytesIO(_fc))
    _buf.seek(0)
    with tarfile.open(fileobj=_buf, mode="r") as _tf:
        _tf.extractall(_tmpdir, filter="data")
    assert os.path.exists(os.path.join(_tmpdir, "file1.txt")), "file1.txt extracted"
    assert os.path.exists(os.path.join(_tmpdir, "subdir", "file2.txt")), "subdir/file2.txt"
    with open(os.path.join(_tmpdir, "file1.txt"), "rb") as _f:
        assert _f.read() == b"one", "extracted file1 content"

print("extractall_writes_tree_to_disk OK")
