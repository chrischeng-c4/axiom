# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_keys_view_is_present"
# subject = "typing.KeysView"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.KeysView: api_keys_view_is_present (surface)."""
import typing

assert hasattr(typing, "KeysView")
print("api_keys_view_is_present OK")
