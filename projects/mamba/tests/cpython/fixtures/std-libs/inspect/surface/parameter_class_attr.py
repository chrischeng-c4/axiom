# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "parameter_class_attr"
# subject = "inspect"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect: parameter_class_attr (surface)."""
import inspect

assert hasattr(inspect, "Parameter")
print("parameter_class_attr OK")
