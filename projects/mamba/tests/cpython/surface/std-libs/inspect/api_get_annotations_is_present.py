# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_get_annotations_is_present"
# subject = "inspect.get_annotations"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.get_annotations: api_get_annotations_is_present (surface)."""
import inspect

assert hasattr(inspect, "get_annotations")
print("api_get_annotations_is_present OK")
