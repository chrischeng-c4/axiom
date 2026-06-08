# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_io_is_present"
# subject = "typing.IO"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.IO: api_io_is_present (surface)."""
import typing

assert hasattr(typing, "IO")
print("api_io_is_present OK")
