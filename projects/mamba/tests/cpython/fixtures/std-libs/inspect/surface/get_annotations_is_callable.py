# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "get_annotations_is_callable"
# subject = "inspect.get_annotations"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.get_annotations: get_annotations_is_callable (surface)."""
import inspect

assert callable(inspect.get_annotations)
print("get_annotations_is_callable OK")
