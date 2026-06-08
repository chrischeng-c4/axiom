# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "api_attrgetter_is_present"
# subject = "operator.attrgetter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""operator.attrgetter: api_attrgetter_is_present (surface)."""
import operator

assert hasattr(operator, "attrgetter")
print("api_attrgetter_is_present OK")
