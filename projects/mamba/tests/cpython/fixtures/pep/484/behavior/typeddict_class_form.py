# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "behavior"
# case = "typeddict_class_form"
# subject = "typing.TypedDict"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.TypedDict: a class-form TypedDict carries field metadata while instances are plain dicts: Movie(TypedDict) with title:str, year:int gives a dict instance, __annotations__=={'title':str,'year':int}, __total__ is True, __required_keys__==frozenset({'title','year'})"""
from typing import TypedDict


# Class-form TypedDict carries field metadata; instances are plain dicts.
class Movie(TypedDict):
    title: str
    year: int


m: Movie = {"title": "Dune", "year": 2021}
assert isinstance(m, dict)
assert Movie.__annotations__ == {"title": str, "year": int}
assert Movie.__total__ is True
assert Movie.__required_keys__ == frozenset({"title", "year"})

print("typeddict_class_form OK")
