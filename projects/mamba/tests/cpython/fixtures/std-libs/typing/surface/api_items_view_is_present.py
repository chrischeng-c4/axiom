# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_items_view_is_present"
# subject = "typing.ItemsView"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.ItemsView: api_items_view_is_present (surface)."""
import typing

assert hasattr(typing, "ItemsView")
print("api_items_view_is_present OK")
