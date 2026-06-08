# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "behavior"
# case = "open_errors_ignore_handler"
# subject = "bz2.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.open: bz2.open rt with errors='ignore' drops undecodable bytes when reading binary content as ascii"""
import bz2
import os
import tempfile

with tempfile.TemporaryDirectory() as td:
    fn = os.path.join(td, "f.bz2")
    with bz2.open(fn, "wb") as f:
        f.write(b"foo\xffbar")
    with bz2.open(fn, "rt", encoding="ascii", errors="ignore") as f:
        assert f.read() == "foobar", "errors=ignore"
print("open_errors_ignore_handler OK")
