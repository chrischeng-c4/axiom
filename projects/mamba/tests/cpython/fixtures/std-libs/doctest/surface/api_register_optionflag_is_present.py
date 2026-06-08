# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "surface"
# case = "api_register_optionflag_is_present"
# subject = "doctest.register_optionflag"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""doctest.register_optionflag: api_register_optionflag_is_present (surface)."""
import doctest

assert hasattr(doctest, "register_optionflag")
print("api_register_optionflag_is_present OK")
