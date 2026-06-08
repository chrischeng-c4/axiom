# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "mkstemp_returns_fd_and_path"
# subject = "tempfile.mkstemp"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.mkstemp: mkstemp() returns an (int fd, str path) pair and the path exists on disk"""
import os
import tempfile

_fd, _path = tempfile.mkstemp()
try:
    assert isinstance(_fd, int), f"fd type = {type(_fd)!r}"
    assert isinstance(_path, str), f"path type = {type(_path)!r}"
    assert os.path.exists(_path), "mkstemp file exists"
finally:
    os.close(_fd)
    os.unlink(_path)
print("mkstemp_returns_fd_and_path OK")
