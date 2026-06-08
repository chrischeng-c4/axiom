# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "abc"
# dimension = "surface"
# case = "api_abstractclassmethod_is_present"
# subject = "abc.abstractclassmethod"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""abc.abstractclassmethod: api_abstractclassmethod_is_present (surface)."""
import abc

assert hasattr(abc, "abstractclassmethod")
print("api_abstractclassmethod_is_present OK")
