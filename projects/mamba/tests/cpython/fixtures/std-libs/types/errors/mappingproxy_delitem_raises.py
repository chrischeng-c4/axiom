# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "errors"
# case = "mappingproxy_delitem_raises"
# subject = "types.MappingProxyType"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.MappingProxyType: mappingproxy_delitem_raises (errors)."""
import operator, types

_raised = False
try:
    operator.delitem(types.MappingProxyType({'a': 1}), 'a')
except TypeError:
    _raised = True
assert _raised, "mappingproxy_delitem_raises: expected TypeError"
print("mappingproxy_delitem_raises OK")
