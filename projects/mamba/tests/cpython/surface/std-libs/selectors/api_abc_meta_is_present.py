# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "surface"
# case = "api_abc_meta_is_present"
# subject = "selectors.ABCMeta"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""selectors.ABCMeta: api_abc_meta_is_present (surface)."""
import selectors

assert hasattr(selectors, "ABCMeta")
print("api_abc_meta_is_present OK")
