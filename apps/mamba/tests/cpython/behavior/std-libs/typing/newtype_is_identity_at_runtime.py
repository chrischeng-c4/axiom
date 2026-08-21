# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "behavior"
# case = "newtype_is_identity_at_runtime"
# subject = "typing.NewType"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_typing.py"
# status = "filled"
# ///
"""typing.NewType: NewType('UserId', int) is the identity function at runtime: UserId(5) == 5 and is a plain int; the name is exposed via __name__"""
import typing

UserId = typing.NewType("UserId", int)
value = UserId(5)
assert value == 5, "NewType call is the identity function at runtime"
assert type(value) is int, "NewType('UserId', int)(5) is a plain int"
assert UserId.__name__ == "UserId", "NewType exposes its name via __name__"
print("newtype_is_identity_at_runtime OK")
