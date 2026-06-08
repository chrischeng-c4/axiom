# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "surface"
# case = "api_sys_is_present"
# subject = "selectors.sys"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""selectors.sys: api_sys_is_present (surface)."""
import selectors

assert hasattr(selectors, "sys")
print("api_sys_is_present OK")
