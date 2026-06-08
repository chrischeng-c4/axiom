# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "temporary_directory_path_is_absolute"
# subject = "tempfile.TemporaryDirectory"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.TemporaryDirectory: the path yielded by TemporaryDirectory() is an existing, absolute directory while the with-block is open"""
import os
import tempfile

with tempfile.TemporaryDirectory() as _d:
    assert os.path.isdir(_d), f"tmpdir exists = {_d!r}"
    assert os.path.isabs(_d), f"tmpdir is absolute = {_d!r}"
assert not os.path.exists(_d), "tmpdir removed after with"
print("temporary_directory_path_is_absolute OK")
