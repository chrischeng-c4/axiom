# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "695"
# dimension = "behavior"
# case = "generic_function_polymorphic"
# subject = "typing.TypeVar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.TypeVar: a generic function def first[T](xs)->T is fully polymorphic at runtime: first([1,2,3]) is 1 and first(['a','b']) is 'a' (the type param is erased)"""


# A generic function is fully polymorphic at runtime; the type param is erased.
def first[T](xs: list[T]) -> T:
    return xs[0]


assert first([1, 2, 3]) == 1
assert first(["a", "b"]) == "a"

print("generic_function_polymorphic OK")
