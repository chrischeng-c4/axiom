# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "parameter_var_positional_attr"
# subject = "inspect.Parameter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.Parameter: parameter_var_positional_attr (surface)."""
import inspect

assert hasattr(inspect.Parameter, "VAR_POSITIONAL")
print("parameter_var_positional_attr OK")
