# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "behavior"
# case = "open_fileobj_read_modes"
# subject = "gzip.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""gzip.open: gzip.open accepts an in-memory BytesIO fileobj and honours 'r'/'rb' (bytes) and 'rt' (decoded str) read modes over the same compressed payload"""
import gzip
import io

_raw = b"the quick brown fox\n" * 25
_text = _raw.decode("ascii")
_compressed = gzip.compress(_raw)

with gzip.open(io.BytesIO(_compressed), "r") as _f:
    assert _f.read() == _raw, "open fileobj mode r"
with gzip.open(io.BytesIO(_compressed), "rb") as _f:
    assert _f.read() == _raw, "open fileobj mode rb"
with gzip.open(io.BytesIO(_compressed), "rt", encoding="ascii") as _f:
    assert _f.read() == _text, "open fileobj mode rt"

print("open_fileobj_read_modes OK")
