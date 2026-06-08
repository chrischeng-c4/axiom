# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "surface"
# case = "api_text_encoding_is_present"
# subject = "io.text_encoding"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""io.text_encoding: api_text_encoding_is_present (surface)."""
import io

assert hasattr(io, "text_encoding")
print("api_text_encoding_is_present OK")
