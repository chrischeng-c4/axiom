# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "surface"
# case = "api_abstractmethod_is_present"
# subject = "selectors.abstractmethod"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""selectors.abstractmethod: api_abstractmethod_is_present (surface)."""
import selectors

assert hasattr(selectors, "abstractmethod")
print("api_abstractmethod_is_present OK")
