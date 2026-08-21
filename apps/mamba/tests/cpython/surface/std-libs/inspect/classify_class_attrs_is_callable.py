# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "classify_class_attrs_is_callable"
# subject = "inspect.classify_class_attrs"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.classify_class_attrs: classify_class_attrs_is_callable (surface)."""
import inspect

assert callable(inspect.classify_class_attrs)
print("classify_class_attrs_is_callable OK")
