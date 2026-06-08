# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_runtime_checkable_is_present"
# subject = "typing.runtime_checkable"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.runtime_checkable: api_runtime_checkable_is_present (surface)."""
import typing

assert hasattr(typing, "runtime_checkable")
print("api_runtime_checkable_is_present OK")
