# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "surface"
# case = "api_string_io_is_present"
# subject = "io.StringIO"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""io.StringIO: api_string_io_is_present (surface)."""
import io

assert hasattr(io, "StringIO")
print("api_string_io_is_present OK")
