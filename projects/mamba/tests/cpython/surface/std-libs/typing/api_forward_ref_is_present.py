# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_forward_ref_is_present"
# subject = "typing.ForwardRef"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.ForwardRef: api_forward_ref_is_present (surface)."""
import typing

assert hasattr(typing, "ForwardRef")
print("api_forward_ref_is_present OK")
