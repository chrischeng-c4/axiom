# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "api_length_hint_is_present"
# subject = "operator.length_hint"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""operator.length_hint: api_length_hint_is_present (surface)."""
import operator

assert hasattr(operator, "length_hint")
print("api_length_hint_is_present OK")
