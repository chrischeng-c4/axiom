# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_tpflags_is_abstract_is_present"
# subject = "inspect.TPFLAGS_IS_ABSTRACT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.TPFLAGS_IS_ABSTRACT: api_tpflags_is_abstract_is_present (surface)."""
import inspect

assert hasattr(inspect, "TPFLAGS_IS_ABSTRACT")
print("api_tpflags_is_abstract_is_present OK")
