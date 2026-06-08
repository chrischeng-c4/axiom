# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections_abc"
# dimension = "surface"
# case = "api_keys_view_is_present"
# subject = "collections.abc.KeysView"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""collections.abc.KeysView: api_keys_view_is_present (surface)."""
import collections.abc

assert hasattr(collections.abc, "KeysView")
print("api_keys_view_is_present OK")
