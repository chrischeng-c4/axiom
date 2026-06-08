# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "surface"
# case = "api_check_id_max_is_present"
# subject = "lzma.CHECK_ID_MAX"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""lzma.CHECK_ID_MAX: api_check_id_max_is_present (surface)."""
import lzma

assert hasattr(lzma, "CHECK_ID_MAX")
print("api_check_id_max_is_present OK")
