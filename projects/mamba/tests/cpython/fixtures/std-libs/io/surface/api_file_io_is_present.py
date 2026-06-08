# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "surface"
# case = "api_file_io_is_present"
# subject = "io.FileIO"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""io.FileIO: api_file_io_is_present (surface)."""
import io

assert hasattr(io, "FileIO")
print("api_file_io_is_present OK")
