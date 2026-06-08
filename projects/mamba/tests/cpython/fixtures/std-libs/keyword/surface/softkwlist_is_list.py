# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keyword"
# dimension = "surface"
# case = "softkwlist_is_list"
# subject = "keyword.softkwlist"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""keyword.softkwlist: softkwlist_is_list (surface)."""
import keyword

assert type(keyword.softkwlist).__name__ == "list"
print("softkwlist_is_list OK")
