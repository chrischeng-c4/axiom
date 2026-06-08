# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "surface"
# case = "api_raw_io_base_is_present"
# subject = "io.RawIOBase"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""io.RawIOBase: api_raw_io_base_is_present (surface)."""
import io

assert hasattr(io, "RawIOBase")
print("api_raw_io_base_is_present OK")
