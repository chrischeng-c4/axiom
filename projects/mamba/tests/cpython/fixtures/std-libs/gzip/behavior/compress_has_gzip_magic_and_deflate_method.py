# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gzip"
# dimension = "behavior"
# case = "compress_has_gzip_magic_and_deflate_method"
# subject = "gzip.compress"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""gzip.compress: a compressed stream starts with the gzip magic bytes 0x1f 0x8b and a CM byte of 0x08 (DEFLATE), and is itself a bytes object"""
import gzip

_data = b"hello world this is a test"
_compressed = gzip.compress(_data)
assert isinstance(_compressed, bytes), f"compress type = {type(_compressed)!r}"
# gzip magic bytes: 1f 8b
assert _compressed[:2] == b"\x1f\x8b", f"gzip magic = {_compressed[:2]!r}"
# CM byte (compression method) = 8 means DEFLATE.
assert _compressed[2] == 0x08, f"CM byte should be DEFLATE=8, got {_compressed[2]:#x}"

print("compress_has_gzip_magic_and_deflate_method OK")
