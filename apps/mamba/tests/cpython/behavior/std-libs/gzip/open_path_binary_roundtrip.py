# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "behavior"
# case = "open_path_binary_roundtrip"
# subject = "gzip.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""gzip.open: gzip.open on a tempfile path writes bytes in 'wb' and reads them back identically in 'rb'"""
import gzip
import os
import tempfile

with tempfile.TemporaryDirectory() as _d:
    _p = os.path.join(_d, "data.gz")
    with gzip.open(_p, "wb") as _gf:
        _gf.write(b"gzipped content")
    with gzip.open(_p, "rb") as _gf2:
        _content = _gf2.read()
    assert _content == b"gzipped content", f"gzip.open round-trip = {_content!r}"

print("open_path_binary_roundtrip OK")
