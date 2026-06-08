# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "surface"
# case = "api_namedtuple_is_present"
# subject = "selectors.namedtuple"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""selectors.namedtuple: api_namedtuple_is_present (surface)."""
import selectors

assert hasattr(selectors, "namedtuple")
print("api_namedtuple_is_present OK")
