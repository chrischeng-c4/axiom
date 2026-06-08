# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "surface"
# case = "mapping_attr"
# subject = "selectors"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""selectors: mapping_attr (surface)."""
import selectors

assert hasattr(selectors, "Mapping")
print("mapping_attr OK")
