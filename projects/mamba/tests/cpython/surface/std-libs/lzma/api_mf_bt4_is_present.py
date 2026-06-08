# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "surface"
# case = "api_mf_bt4_is_present"
# subject = "lzma.MF_BT4"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""lzma.MF_BT4: api_mf_bt4_is_present (surface)."""
import lzma

assert hasattr(lzma, "MF_BT4")
print("api_mf_bt4_is_present OK")
