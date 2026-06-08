# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "decompressobj_eof_flag_flips_at_stream_end"
# subject = "zlib.decompressobj"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.decompressobj: the decompressobj eof flag is False before input and mid-stream, and flips to True only once the full stream has been fed"""
import zlib

_stream = zlib.compress(b"foo")
_dco = zlib.decompressobj()
assert _dco.eof is False, "eof False before any input"
_dco.decompress(_stream[:-2])
assert _dco.eof is False, "eof False mid-stream"
_dco.decompress(_stream[-2:])
assert _dco.eof is True, "eof True after full stream"

print("decompressobj_eof_flag_flips_at_stream_end OK")
