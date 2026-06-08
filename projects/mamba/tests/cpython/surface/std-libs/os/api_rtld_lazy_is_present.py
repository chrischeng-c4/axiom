# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_rtld_lazy_is_present"
# subject = "os.RTLD_LAZY"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.RTLD_LAZY: api_rtld_lazy_is_present (surface)."""
import os

assert hasattr(os, "RTLD_LAZY")
print("api_rtld_lazy_is_present OK")
