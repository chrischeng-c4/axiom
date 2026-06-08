# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "behavior"
# case = "samefile_same_path_and_abspath"
# subject = "os.path.samefile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
"""os.path.samefile: samefile is True when both arguments resolve to the same file: a temp file vs itself, and the temp file vs its abspath()"""
import os
import os.path
import tempfile

with tempfile.NamedTemporaryFile(delete=False) as _sf:
    _sfname = _sf.name
try:
    assert os.path.samefile(_sfname, _sfname), "samefile same path"
    _abs = os.path.abspath(_sfname)
    assert os.path.samefile(_sfname, _abs), "samefile abspath"
finally:
    os.unlink(_sfname)

print("samefile_same_path_and_abspath OK")
