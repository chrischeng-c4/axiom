# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections_abc"
# dimension = "surface"
# case = "api_items_view_is_present"
# subject = "collections.abc.ItemsView"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""collections.abc.ItemsView: api_items_view_is_present (surface)."""
import collections.abc

assert hasattr(collections.abc, "ItemsView")
print("api_items_view_is_present OK")
