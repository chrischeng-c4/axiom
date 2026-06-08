# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_p_nowaito_is_present"
# subject = "os.P_NOWAITO"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.P_NOWAITO: api_p_nowaito_is_present (surface)."""
import os

assert hasattr(os, "P_NOWAITO")
print("api_p_nowaito_is_present OK")
