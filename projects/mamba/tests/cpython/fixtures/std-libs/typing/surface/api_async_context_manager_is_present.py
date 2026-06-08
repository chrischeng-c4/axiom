# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_async_context_manager_is_present"
# subject = "typing.AsyncContextManager"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.AsyncContextManager: api_async_context_manager_is_present (surface)."""
import typing

assert hasattr(typing, "AsyncContextManager")
print("api_async_context_manager_is_present OK")
