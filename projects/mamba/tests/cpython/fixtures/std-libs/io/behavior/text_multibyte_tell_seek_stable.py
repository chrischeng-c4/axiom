# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "text_multibyte_tell_seek_stable"
# subject = "io.TextIOWrapper"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io.TextIOWrapper: for a multibyte encoding (euc_jp), tell() positions are stable across seek/readline round-trips"""
import io

import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    path = os.path.join(d, "data.txt")
    with open(path, "w", encoding="euc_jp") as f:
        f.write("AB\nうえ\n")
    with open(path, "r", encoding="euc_jp") as f:
        assert f.readline() == "AB\n", "euc_jp line 1"
        p0 = f.tell()
        assert f.readline() == "うえ\n", "euc_jp line 2"
        p1 = f.tell()
        f.seek(p0)
        assert f.readline() == "うえ\n", "euc_jp re-read after seek"
        assert f.tell() == p1, "euc_jp tell after re-read"

print("text_multibyte_tell_seek_stable OK")
