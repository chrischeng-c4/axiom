# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_param_spec_args_is_present"
# subject = "typing.ParamSpecArgs"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.ParamSpecArgs: api_param_spec_args_is_present (surface)."""
import typing

assert hasattr(typing, "ParamSpecArgs")
print("api_param_spec_args_is_present OK")
