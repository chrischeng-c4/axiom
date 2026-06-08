# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "surface"
# case = "api_filter_arm_is_present"
# subject = "lzma.FILTER_ARM"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""lzma.FILTER_ARM: api_filter_arm_is_present (surface)."""
import lzma

assert hasattr(lzma, "FILTER_ARM")
print("api_filter_arm_is_present OK")
