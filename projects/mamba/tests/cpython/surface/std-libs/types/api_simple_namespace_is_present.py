# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "surface"
# case = "api_simple_namespace_is_present"
# subject = "types.SimpleNamespace"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""types.SimpleNamespace: api_simple_namespace_is_present (surface)."""
import types

assert hasattr(types, "SimpleNamespace")
print("api_simple_namespace_is_present OK")
