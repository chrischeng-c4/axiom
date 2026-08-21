# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "binary_readline_size_caps_bytes"
# subject = "io.BufferedReader"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io.BufferedReader: readline(size) on a binary stream caps the returned bytes per call, handles NUL bytes, None reads a whole line, and a float size raises TypeError"""
import io

import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    path = os.path.join(d, "data.txt")
    with open(path, "wb") as f:
        f.write(b"abc\ndef\nxyzzy\nfoo\x00bar\ntail")
    with open(path, "rb") as f:
        assert f.readline() == b"abc\n", "full line"
        assert f.readline(10) == b"def\n", "size beyond newline"
        assert f.readline(2) == b"xy", "size mid-line"
        assert f.readline(4) == b"zzy\n", "size to newline"
        assert f.readline() == b"foo\x00bar\n", "line with NUL byte"
        assert f.readline(None) == b"tail", "None size reads whole line"
        float_size = False
        try:
            f.readline(5.3)
        except TypeError:
            float_size = True
        assert float_size, "float readline size did not raise TypeError"

print("binary_readline_size_caps_bytes OK")
