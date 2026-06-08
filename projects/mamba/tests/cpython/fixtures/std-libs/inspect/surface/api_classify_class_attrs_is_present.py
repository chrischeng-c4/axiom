# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_classify_class_attrs_is_present"
# subject = "inspect.classify_class_attrs"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.classify_class_attrs: api_classify_class_attrs_is_present (surface)."""
import inspect

assert hasattr(inspect, "classify_class_attrs")
print("api_classify_class_attrs_is_present OK")
