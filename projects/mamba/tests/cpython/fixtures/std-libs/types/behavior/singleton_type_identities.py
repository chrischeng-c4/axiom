# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "singleton_type_identities"
# subject = "types.NoneType"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.NoneType: NoneType/NotImplementedType/EllipsisType are exactly the runtime types of None/NotImplemented/Ellipsis (isinstance holds and type(None) is types.NoneType)"""
import types

assert isinstance(None, types.NoneType)
assert isinstance(NotImplemented, types.NotImplementedType)
assert isinstance(Ellipsis, types.EllipsisType)
assert type(None) is types.NoneType
assert type(NotImplemented) is types.NotImplementedType
assert type(Ellipsis) is types.EllipsisType

print("singleton_type_identities OK")
