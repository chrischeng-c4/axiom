# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "parameter_empty_sentinel_attr"
# subject = "inspect.Parameter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.Parameter: parameter_empty_sentinel_attr (surface)."""
import inspect

assert hasattr(inspect.Parameter, "empty")
print("parameter_empty_sentinel_attr OK")
