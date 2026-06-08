# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "surface"
# case = "api_incremental_decoder_is_present"
# subject = "codecs.IncrementalDecoder"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""codecs.IncrementalDecoder: api_incremental_decoder_is_present (surface)."""
import codecs

assert hasattr(codecs, "IncrementalDecoder")
print("api_incremental_decoder_is_present OK")
