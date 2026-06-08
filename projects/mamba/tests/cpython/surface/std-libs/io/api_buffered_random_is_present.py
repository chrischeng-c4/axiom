# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "surface"
# case = "api_buffered_random_is_present"
# subject = "io.BufferedRandom"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""io.BufferedRandom: api_buffered_random_is_present (surface)."""
import io

assert hasattr(io, "BufferedRandom")
print("api_buffered_random_is_present OK")
