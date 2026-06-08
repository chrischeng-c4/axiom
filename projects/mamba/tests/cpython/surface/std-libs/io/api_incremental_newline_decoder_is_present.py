# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "surface"
# case = "api_incremental_newline_decoder_is_present"
# subject = "io.IncrementalNewlineDecoder"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""io.IncrementalNewlineDecoder: api_incremental_newline_decoder_is_present (surface)."""
import io

assert hasattr(io, "IncrementalNewlineDecoder")
print("api_incremental_newline_decoder_is_present OK")
