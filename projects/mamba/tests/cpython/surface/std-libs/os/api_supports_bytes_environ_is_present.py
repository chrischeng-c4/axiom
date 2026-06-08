# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_supports_bytes_environ_is_present"
# subject = "os.supports_bytes_environ"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.supports_bytes_environ: api_supports_bytes_environ_is_present (surface)."""
import os

assert hasattr(os, "supports_bytes_environ")
print("api_supports_bytes_environ_is_present OK")
