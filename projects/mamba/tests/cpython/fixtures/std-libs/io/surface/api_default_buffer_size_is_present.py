# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "surface"
# case = "api_default_buffer_size_is_present"
# subject = "io.DEFAULT_BUFFER_SIZE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""io.DEFAULT_BUFFER_SIZE: api_default_buffer_size_is_present (surface)."""
import io

assert hasattr(io, "DEFAULT_BUFFER_SIZE")
print("api_default_buffer_size_is_present OK")
