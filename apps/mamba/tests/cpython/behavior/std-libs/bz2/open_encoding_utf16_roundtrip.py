# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "behavior"
# case = "open_encoding_utf16_roundtrip"
# subject = "bz2.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.open: a non-default encoding (utf-16-le) survives a bz2.open text write/read round-trip"""
import bz2
import os
import tempfile

with tempfile.TemporaryDirectory() as td:
    fn = os.path.join(td, "f.bz2")
    text = "héllo wörld\nsecond line\n"
    with bz2.open(fn, "wt", encoding="utf-16-le") as f:
        f.write(text)
    with bz2.open(fn, "rt", encoding="utf-16-le") as f:
        assert f.read() == text, "utf-16-le round-trip"
print("open_encoding_utf16_roundtrip OK")
