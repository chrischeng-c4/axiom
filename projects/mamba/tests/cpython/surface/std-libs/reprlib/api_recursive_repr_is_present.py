# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "reprlib"
# dimension = "surface"
# case = "api_recursive_repr_is_present"
# subject = "reprlib.recursive_repr"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""reprlib.recursive_repr: api_recursive_repr_is_present (surface)."""
import reprlib

assert hasattr(reprlib, "recursive_repr")
print("api_recursive_repr_is_present OK")
