# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "surface"
# case = "api_check_sha256_is_present"
# subject = "lzma.CHECK_SHA256"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""lzma.CHECK_SHA256: api_check_sha256_is_present (surface)."""
import lzma

assert hasattr(lzma, "CHECK_SHA256")
print("api_check_sha256_is_present OK")
