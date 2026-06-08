# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dis"
# dimension = "surface"
# case = "api_show_code_is_present"
# subject = "dis.show_code"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""dis.show_code: api_show_code_is_present (surface)."""
import dis

assert hasattr(dis, "show_code")
print("api_show_code_is_present OK")
