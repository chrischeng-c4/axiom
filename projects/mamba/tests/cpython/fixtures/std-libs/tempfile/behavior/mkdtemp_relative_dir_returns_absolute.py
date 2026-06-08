# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "mkdtemp_relative_dir_returns_absolute"
# subject = "tempfile.mkdtemp"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.mkdtemp: mkdtemp(dir='.') still returns an absolute path"""
import os
import tempfile

_rel = tempfile.mkdtemp(dir=".")
try:
    assert os.path.isabs(_rel), f"mkdtemp(dir='.') absolute = {_rel!r}"
finally:
    os.rmdir(_rel)
print("mkdtemp_relative_dir_returns_absolute OK")
