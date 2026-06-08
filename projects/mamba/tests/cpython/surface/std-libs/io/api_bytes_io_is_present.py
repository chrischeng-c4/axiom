# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "surface"
# case = "api_bytes_io_is_present"
# subject = "io.BytesIO"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""io.BytesIO: api_bytes_io_is_present (surface)."""
import io

assert hasattr(io, "BytesIO")
print("api_bytes_io_is_present OK")
