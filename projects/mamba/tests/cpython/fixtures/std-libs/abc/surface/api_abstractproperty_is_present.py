# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "abc"
# dimension = "surface"
# case = "api_abstractproperty_is_present"
# subject = "abc.abstractproperty"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""abc.abstractproperty: api_abstractproperty_is_present (surface)."""
import abc

assert hasattr(abc, "abstractproperty")
print("api_abstractproperty_is_present OK")
