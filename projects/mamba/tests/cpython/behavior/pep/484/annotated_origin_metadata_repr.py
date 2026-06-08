# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "behavior"
# case = "annotated_origin_metadata_repr"
# subject = "typing.Annotated"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.Annotated: Annotated keeps its origin and folds nested metadata in order: Annotated[Annotated[int,4],5]==Annotated[int,4,5], .__origin__ is int, .__metadata__==(4,5), repr=='typing.Annotated[int, 4, 5]'; equality is order/identity sensitive and hashable metadata makes the form deduplicable"""
from typing import Annotated, List

# Annotated keeps its origin type and folds nested metadata in order.
A = Annotated[Annotated[int, 4], 5]
assert A == Annotated[int, 4, 5]
assert A.__origin__ is int
assert A.__metadata__ == (4, 5)
assert repr(A) == "typing.Annotated[int, 4, 5]"
assert repr(Annotated[List[int], 9]) == "typing.Annotated[typing.List[int], 9]"
# Equality is sensitive to metadata order and identity.
assert Annotated[int, 4, 5] != Annotated[int, 5, 4]
assert Annotated[int, 4] != Annotated[str, 4]
# Hashable metadata makes the form hashable and deduplicable.
assert len({Annotated[int, 4, 5], Annotated[int, 4, 5]}) == 1

print("annotated_origin_metadata_repr OK")
