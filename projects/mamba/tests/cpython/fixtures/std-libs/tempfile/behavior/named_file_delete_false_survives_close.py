# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "named_file_delete_false_survives_close"
# subject = "tempfile.NamedTemporaryFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.NamedTemporaryFile: delete=False keeps the file after close; the caller must unlink it manually"""
import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    with tempfile.NamedTemporaryFile(dir=d, delete=False) as keep:
        keep.write(b"blat")
        keep_name = keep.name
    assert os.path.exists(keep_name), "delete=False keeps the file"
    os.unlink(keep_name)
    assert os.listdir(d) == [], "dir empty after manual unlink"
print("named_file_delete_false_survives_close OK")
