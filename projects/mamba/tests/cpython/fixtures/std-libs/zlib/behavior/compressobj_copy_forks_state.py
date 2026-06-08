# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "compressobj_copy_forks_state"
# subject = "zlib.compressobj"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.compressobj: copy() forks a compressor at the copy point so both share the pre-copy prefix then diverge with independent follow-up input, each decompressing to its own concatenation"""
import zlib

_TEXT = b"the quick brown fox jumps over the lazy dog " * 64
_c0 = zlib.compressobj(zlib.Z_BEST_COMPRESSION)
_prefix = _c0.compress(_TEXT)
_c1 = _c0.copy()
_alt = b"ZZZ" * 50
_s0 = _prefix + _c0.compress(_TEXT) + _c0.flush()
_s1 = _prefix + _c1.compress(_alt) + _c1.flush()
assert zlib.decompress(_s0) == _TEXT + _TEXT, "copy: original keeps compressing"
assert zlib.decompress(_s1) == _TEXT + _alt, "copy: fork diverges from prefix"

print("compressobj_copy_forks_state OK")
