# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "behavior"
# case = "open_path_text_roundtrip"
# subject = "gzip.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""gzip.open: gzip.open on a tempfile path in 'wt' writes str and 'rt' reads the same str back"""
import gzip
import os
import tempfile

with tempfile.TemporaryDirectory() as _d:
    _p = os.path.join(_d, "data.gz")
    with gzip.open(_p, "wt") as _gf:
        _gf.write("text content\n")
    with gzip.open(_p, "rt") as _gf2:
        _content = _gf2.read()
    assert _content == "text content\n", f"text mode = {_content!r}"

print("open_path_text_roundtrip OK")
