# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "surface"
# case = "api_proxy_types_is_present"
# subject = "weakref.ProxyTypes"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""weakref.ProxyTypes: api_proxy_types_is_present (surface)."""
import weakref

assert hasattr(weakref, "ProxyTypes")
print("api_proxy_types_is_present OK")
