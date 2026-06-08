# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "reprlib"
# dimension = "surface"
# case = "api_repr_is_present_2"
# subject = "reprlib.repr"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""reprlib.repr: api_repr_is_present_2 (surface)."""
import reprlib

assert hasattr(reprlib, "repr")
print("api_repr_is_present_2 OK")
