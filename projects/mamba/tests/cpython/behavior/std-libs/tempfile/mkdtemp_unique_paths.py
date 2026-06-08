# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "mkdtemp_unique_paths"
# subject = "tempfile.mkdtemp"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.mkdtemp: two mkdtemp() calls return two distinct existing directories"""
import os
import tempfile

_d1 = tempfile.mkdtemp()
_d2 = tempfile.mkdtemp()
try:
    assert _d1 != _d2, f"mkdtemp unique: {_d1} vs {_d2}"
    assert os.path.isdir(_d1), "d1 is dir"
    assert os.path.isdir(_d2), "d2 is dir"
finally:
    os.rmdir(_d1); os.rmdir(_d2)
print("mkdtemp_unique_paths OK")
