# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "behavior"
# case = "open_text_exclusive_roundtrip"
# subject = "bz2.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.open: bz2.open xt mode creates a text file exclusively and rt reads back the written string"""
import bz2
import os
import tempfile

with tempfile.TemporaryDirectory() as td:
    xt = os.path.join(td, "g.bz2")
    with bz2.open(xt, "xt", encoding="utf-8") as f:
        f.write("hi")
    with bz2.open(xt, "rt", encoding="utf-8") as f:
        assert f.read() == "hi", "xt round-trip"
print("open_text_exclusive_roundtrip OK")
