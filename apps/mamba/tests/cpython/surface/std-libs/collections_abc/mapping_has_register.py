# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections_abc"
# dimension = "surface"
# case = "mapping_has_register"
# subject = "collections.abc.Mapping"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.abc.Mapping: mapping_has_register (surface)."""
import collections.abc

assert hasattr(collections.abc.Mapping, "register")
print("mapping_has_register OK")
