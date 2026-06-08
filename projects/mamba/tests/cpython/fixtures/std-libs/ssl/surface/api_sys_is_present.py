# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_sys_is_present"
# subject = "ssl.sys"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.sys: api_sys_is_present (surface)."""
import ssl

assert hasattr(ssl, "sys")
print("api_sys_is_present OK")
