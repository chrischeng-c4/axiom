# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "surface"
# case = "api_proxy_is_present"
# subject = "weakref.proxy"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""weakref.proxy: api_proxy_is_present (surface)."""
import weakref

assert hasattr(weakref, "proxy")
print("api_proxy_is_present OK")
