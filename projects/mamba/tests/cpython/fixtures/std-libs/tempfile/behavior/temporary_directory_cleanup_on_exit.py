# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "temporary_directory_cleanup_on_exit"
# subject = "tempfile.TemporaryDirectory"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.TemporaryDirectory: TemporaryDirectory removes the directory (and files created inside it) when the with-block exits"""
import os
import tempfile

_path = None
with tempfile.TemporaryDirectory() as _d:
    _path = _d
    _fpath = os.path.join(_d, "test.txt")
    with open(_fpath, "w") as _f:
        _f.write("inside tmpdir")
    assert os.path.exists(_fpath), "file inside tmpdir"
assert not os.path.exists(_path), "tmpdir removed after with"
print("temporary_directory_cleanup_on_exit OK")
