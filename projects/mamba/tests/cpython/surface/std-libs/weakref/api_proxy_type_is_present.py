# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "surface"
# case = "api_proxy_type_is_present"
# subject = "weakref.ProxyType"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""weakref.ProxyType: api_proxy_type_is_present (surface)."""
import weakref

assert hasattr(weakref, "ProxyType")
print("api_proxy_type_is_present OK")
