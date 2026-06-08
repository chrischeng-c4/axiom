# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "surface"
# case = "api_buffered_reader_is_present"
# subject = "io.BufferedReader"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""io.BufferedReader: api_buffered_reader_is_present (surface)."""
import io

assert hasattr(io, "BufferedReader")
print("api_buffered_reader_is_present OK")
