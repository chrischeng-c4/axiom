# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "surface"
# case = "api_preset_extreme_is_present"
# subject = "lzma.PRESET_EXTREME"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""lzma.PRESET_EXTREME: api_preset_extreme_is_present (surface)."""
import lzma

assert hasattr(lzma, "PRESET_EXTREME")
print("api_preset_extreme_is_present OK")
