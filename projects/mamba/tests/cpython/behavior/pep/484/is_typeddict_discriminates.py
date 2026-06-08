# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "behavior"
# case = "is_typeddict_discriminates"
# subject = "typing.is_typeddict"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.is_typeddict: is_typeddict distinguishes TypedDict classes from everything else: True for a TypedDict subclass (Movie), False for a NamedTuple class (Point), for dict, and for int"""
from typing import NamedTuple, TypedDict, is_typeddict


class Movie(TypedDict):
    title: str
    year: int


class Point(NamedTuple):
    x: int
    y: int = 0


# is_typeddict distinguishes TypedDict classes from everything else.
assert is_typeddict(Movie) is True
assert is_typeddict(Point) is False
assert is_typeddict(dict) is False
assert is_typeddict(int) is False

print("is_typeddict_discriminates OK")
