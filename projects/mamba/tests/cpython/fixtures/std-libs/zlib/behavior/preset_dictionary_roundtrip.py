# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "preset_dictionary_roundtrip"
# subject = "zlib.compressobj"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.compressobj: a preset zdict supplied to both compressobj and decompressobj round-trips the payload, while a decompressor without the dict raises zlib.error"""
import zlib

_TEXT = b"the quick brown fox jumps over the lazy dog " * 64
_zdict = b"the quick brown fox lazy dog"
_co = zlib.compressobj(zdict=_zdict)
_cd = _co.compress(_TEXT) + _co.flush()
_dco = zlib.decompressobj(zdict=_zdict)
assert _dco.decompress(_cd) + _dco.flush() == _TEXT, "zdict round-trip"

_no_dict = zlib.decompressobj()
_raised = False
try:
    _no_dict.decompress(_cd)
except zlib.error:
    _raised = True
assert _raised, "missing zdict raises zlib.error"

print("preset_dictionary_roundtrip OK")
