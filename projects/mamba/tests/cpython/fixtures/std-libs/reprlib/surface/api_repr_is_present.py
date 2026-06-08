# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "reprlib"
# dimension = "surface"
# case = "api_repr_is_present"
# subject = "reprlib.Repr"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""reprlib.Repr: api_repr_is_present (surface)."""
import reprlib

assert hasattr(reprlib, "Repr")
print("api_repr_is_present OK")
