# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "named_file_text_mode_roundtrip"
# subject = "tempfile.NamedTemporaryFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.NamedTemporaryFile: mode='w' makes NamedTemporaryFile a text file; the written string reads back identically from the named path"""
import os
import tempfile

with tempfile.NamedTemporaryFile(mode="w", suffix=".txt", delete=False) as _ntf:
    _name = _ntf.name
    _ntf.write("text line\n")
try:
    with open(_name) as _f:
        assert _f.read() == "text line\n", "text mode ntf"
finally:
    os.unlink(_name)
print("named_file_text_mode_roundtrip OK")
