# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "behavior"
# case = "newtype_identity_with_metadata"
# subject = "typing.NewType"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.NewType: NewType produces a callable identity function with introspectable metadata: UserId=NewType('UserId',int) gives UserId(5)==5, UserId.__name__=='UserId', UserId.__supertype__ is int"""
from typing import NewType

# NewType produces a callable identity function with introspectable metadata.
UserId = NewType("UserId", int)
assert UserId(5) == 5
assert UserId.__name__ == "UserId"
assert UserId.__supertype__ is int

print("newtype_identity_with_metadata OK")
