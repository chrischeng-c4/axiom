# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keyword"
# dimension = "surface"
# case = "kwlist_is_list"
# subject = "keyword.kwlist"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""keyword.kwlist: kwlist_is_list (surface)."""
import keyword

assert type(keyword.kwlist).__name__ == "list"
print("kwlist_is_list OK")
