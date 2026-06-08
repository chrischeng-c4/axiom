# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "surface"
# case = "api_mode_normal_is_present"
# subject = "lzma.MODE_NORMAL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""lzma.MODE_NORMAL: api_mode_normal_is_present (surface)."""
import lzma

assert hasattr(lzma, "MODE_NORMAL")
print("api_mode_normal_is_present OK")
