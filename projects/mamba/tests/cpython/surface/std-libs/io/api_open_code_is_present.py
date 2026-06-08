# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "surface"
# case = "api_open_code_is_present"
# subject = "io.open_code"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""io.open_code: api_open_code_is_present (surface)."""
import io

assert hasattr(io, "open_code")
print("api_open_code_is_present OK")
