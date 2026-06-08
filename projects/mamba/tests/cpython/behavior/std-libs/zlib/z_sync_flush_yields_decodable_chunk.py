# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "behavior"
# case = "z_sync_flush_yields_decodable_chunk"
# subject = "zlib.compressobj"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""zlib.compressobj: flushing a compressor with Z_SYNC_FLUSH emits a chunk that a decompressor can decode mid-stream without the final flush"""
import zlib

_co = zlib.compressobj(zlib.Z_BEST_COMPRESSION)
_dco = zlib.decompressobj()
_chunk = b"sync-flush payload " * 32
_first = _co.compress(_chunk)
_second = _co.flush(zlib.Z_SYNC_FLUSH)
assert _dco.decompress(_first + _second) == _chunk, "Z_SYNC_FLUSH chunk decodable"

print("z_sync_flush_yields_decodable_chunk OK")
