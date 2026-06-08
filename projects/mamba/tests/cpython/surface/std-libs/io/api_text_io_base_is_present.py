# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "surface"
# case = "api_text_io_base_is_present"
# subject = "io.TextIOBase"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""io.TextIOBase: api_text_io_base_is_present (surface)."""
import io

assert hasattr(io, "TextIOBase")
print("api_text_io_base_is_present OK")
