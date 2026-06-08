# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_device_encoding_is_present"
# subject = "os.device_encoding"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.device_encoding: api_device_encoding_is_present (surface)."""
import os

assert hasattr(os, "device_encoding")
print("api_device_encoding_is_present OK")
