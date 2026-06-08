# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "surface"
# case = "api_extract_tb_is_present"
# subject = "traceback.extract_tb"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""traceback.extract_tb: api_extract_tb_is_present (surface)."""
import traceback

assert hasattr(traceback, "extract_tb")
print("api_extract_tb_is_present OK")
