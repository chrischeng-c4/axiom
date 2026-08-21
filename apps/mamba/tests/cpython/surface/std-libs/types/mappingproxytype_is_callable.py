# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "surface"
# case = "mappingproxytype_is_callable"
# subject = "types.MappingProxyType"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""types.MappingProxyType: mappingproxytype_is_callable (surface)."""
import types

assert callable(types.MappingProxyType)
print("mappingproxytype_is_callable OK")
