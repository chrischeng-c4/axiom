# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "errors"
# case = "mappingproxy_setitem_raises"
# subject = "types.MappingProxyType"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.MappingProxyType: mappingproxy_setitem_raises (errors)."""
import operator, types

_raised = False
try:
    operator.setitem(types.MappingProxyType({'a': 1}), 'b', 2)
except TypeError:
    _raised = True
assert _raised, "mappingproxy_setitem_raises: expected TypeError"
print("mappingproxy_setitem_raises OK")
