# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "695"
# dimension = "behavior"
# case = "typeparam_default_args"
# subject = "typing.TypeVar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.TypeVar: default arguments annotated with a type param work normally: def defaulted[T](a=..., *, b=...) yields ('a','b'), (1,'b'), ('a',2)"""


# Default arguments annotated with a type param work normally.
def defaulted[T](a: T = "a", *, b: T = "b"):
    return (a, b)


assert defaulted() == ("a", "b")
assert defaulted(1) == (1, "b")
assert defaulted(b=2) == ("a", 2)

print("typeparam_default_args OK")
