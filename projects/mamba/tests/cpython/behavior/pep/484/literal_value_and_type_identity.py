# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "behavior"
# case = "literal_value_and_type_identity"
# subject = "typing.Literal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.Literal: Literal compares by value-and-type, dedups, ignores order, flattens nesting, and keeps bool/int distinct: Literal[1,2]==Literal[2,1], Literal[1,2,3]==Literal[1,2,3,3], Literal[True]!=Literal[1], Literal[0]!=Literal[False], and Literal[Literal[1,2],3].__args__==(1,2,3)"""
from typing import Literal

# Literal compares by value-and-type, dedups, and ignores order.
assert Literal[1] == Literal[1]
assert Literal[1, 2] == Literal[2, 1]
assert Literal[1, 2, 3] == Literal[1, 2, 3, 3]
assert Literal[1] != Literal[2]
# bool and int literals are kept distinct by type.
assert Literal[True] != Literal[1]
assert Literal[0] != Literal[False]
# Nested Literals flatten.
flat = Literal[Literal[1, 2], 3]
assert flat == Literal[1, 2, 3]
assert flat.__args__ == (1, 2, 3)

print("literal_value_and_type_identity OK")
