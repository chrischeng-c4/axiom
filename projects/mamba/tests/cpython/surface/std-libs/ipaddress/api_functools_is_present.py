# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "surface"
# case = "api_functools_is_present"
# subject = "ipaddress.functools"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ipaddress.functools: api_functools_is_present (surface)."""
import ipaddress

assert hasattr(ipaddress, "functools")
print("api_functools_is_present OK")
