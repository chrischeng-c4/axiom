# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "textwrapper_exposes_buffer_layers"
# subject = "io.TextIOWrapper"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io.TextIOWrapper: a text-mode handle exposes .mode, .buffer.mode and .buffer.raw.mode reflecting the layered stack"""
import io

import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    path = os.path.join(d, "data.txt")
    with open(path, "w+", encoding="utf-8") as f:
        assert f.mode == "w+", f"text mode = {f.mode!r}"
        assert f.buffer.mode == "rb+", f"buffer mode = {f.buffer.mode!r}"
        assert f.buffer.raw.mode == "rb+", f"raw mode = {f.buffer.raw.mode!r}"

print("textwrapper_exposes_buffer_layers OK")
