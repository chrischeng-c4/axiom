# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "mkdir_rmdir_lifecycle"
# subject = "pathlib.Path"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.Path: in a TemporaryDirectory, mkdir() creates a dir that is_dir(), and rmdir() then removes it so it no longer exists()"""
import pathlib

import tempfile
Path = pathlib.Path

with tempfile.TemporaryDirectory() as _tmpdir:
    _subdir = Path(_tmpdir) / "newdir"
    _subdir.mkdir()
    assert _subdir.is_dir(), "mkdir creates dir"
    _subdir.rmdir()
    assert not _subdir.exists(), "rmdir removes dir"
print("mkdir_rmdir_lifecycle OK")
