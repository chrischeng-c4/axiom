# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "behavior"
# case = "open_exclusive_create_mode"
# subject = "bz2.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.open: bz2.open xb mode creates a file exclusively and a second xb open raises FileExistsError"""
import bz2
import os
import tempfile

with tempfile.TemporaryDirectory() as td:
    fn = os.path.join(td, "f.bz2")
    with bz2.open(fn, "xb") as f:
        f.write(b"data")
    try:
        with bz2.open(fn, "xb"):
            pass
        raise AssertionError("expected FileExistsError on second xb open")
    except FileExistsError:
        pass
print("open_exclusive_create_mode OK")
