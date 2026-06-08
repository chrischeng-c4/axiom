# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "abc"
# dimension = "surface"
# case = "api_abstractstaticmethod_is_present"
# subject = "abc.abstractstaticmethod"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""abc.abstractstaticmethod: api_abstractstaticmethod_is_present (surface)."""
import abc

assert hasattr(abc, "abstractstaticmethod")
print("api_abstractstaticmethod_is_present OK")
