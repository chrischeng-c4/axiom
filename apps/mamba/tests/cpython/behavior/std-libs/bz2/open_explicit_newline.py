# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "behavior"
# case = "open_explicit_newline"
# subject = "bz2.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
"""bz2.open: an explicit newline argument disables universal-newline translation on a text read"""
import bz2
import os
import tempfile

with tempfile.TemporaryDirectory() as td:
    fn = os.path.join(td, "f.bz2")
    plain = "a\nb\nc\n"
    with bz2.open(fn, "wt", encoding="utf-8", newline="\n") as f:
        f.write(plain)
    with bz2.open(fn, "rt", encoding="utf-8", newline="\r") as f:
        assert f.readlines() == [plain], "explicit newline"
print("open_explicit_newline OK")
