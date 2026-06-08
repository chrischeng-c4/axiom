# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "behavior"
# case = "exists_isfile_isdir_on_real_paths"
# subject = "os.path.exists"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
"""os.path.exists: inside a TemporaryDirectory, a written file is exists()/isfile() but not isdir(), the directory is isdir() but not isfile(), and a missing name is not exists()"""
import os.path
import tempfile

with tempfile.TemporaryDirectory() as _tmpdir:
    _file = os.path.join(_tmpdir, "test.txt")
    with open(_file, "w") as _f:
        _f.write("hello")
    assert os.path.exists(_file), "file exists"
    assert os.path.isfile(_file), "isfile"
    assert not os.path.isdir(_file), "not isdir for file"
    assert os.path.isdir(_tmpdir), "isdir for dir"
    assert not os.path.isfile(_tmpdir), "not isfile for dir"
    assert not os.path.exists(os.path.join(_tmpdir, "nofile")), "nonexistent"

print("exists_isfile_isdir_on_real_paths OK")
