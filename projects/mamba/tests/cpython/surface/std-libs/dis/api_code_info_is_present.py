# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dis"
# dimension = "surface"
# case = "api_code_info_is_present"
# subject = "dis.code_info"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""dis.code_info: api_code_info_is_present (surface)."""
import dis

assert hasattr(dis, "code_info")
print("api_code_info_is_present OK")
