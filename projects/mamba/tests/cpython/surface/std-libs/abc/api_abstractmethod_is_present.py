# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "abc"
# dimension = "surface"
# case = "api_abstractmethod_is_present"
# subject = "abc.abstractmethod"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""abc.abstractmethod: api_abstractmethod_is_present (surface)."""
import abc

assert hasattr(abc, "abstractmethod")
print("api_abstractmethod_is_present OK")
