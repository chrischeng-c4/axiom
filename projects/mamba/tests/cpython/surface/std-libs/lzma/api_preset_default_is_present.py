# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "surface"
# case = "api_preset_default_is_present"
# subject = "lzma.PRESET_DEFAULT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""lzma.PRESET_DEFAULT: api_preset_default_is_present (surface)."""
import lzma

assert hasattr(lzma, "PRESET_DEFAULT")
print("api_preset_default_is_present OK")
