# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "mkdtemp_returns_existing_dir"
# subject = "tempfile.mkdtemp"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.mkdtemp: mkdtemp() returns a str path to a directory that exists on disk"""
import os
import tempfile

_dpath = tempfile.mkdtemp()
try:
    assert isinstance(_dpath, str), f"mkdtemp type = {type(_dpath)!r}"
    assert os.path.isdir(_dpath), f"mkdtemp is dir: {_dpath!r}"
finally:
    os.rmdir(_dpath)
print("mkdtemp_returns_existing_dir OK")
