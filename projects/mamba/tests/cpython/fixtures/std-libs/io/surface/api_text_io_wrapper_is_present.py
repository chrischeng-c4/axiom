# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "surface"
# case = "api_text_io_wrapper_is_present"
# subject = "io.TextIOWrapper"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""io.TextIOWrapper: api_text_io_wrapper_is_present (surface)."""
import io

assert hasattr(io, "TextIOWrapper")
print("api_text_io_wrapper_is_present OK")
