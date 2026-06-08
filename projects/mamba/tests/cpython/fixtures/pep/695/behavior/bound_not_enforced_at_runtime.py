# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "695"
# dimension = "behavior"
# case = "bound_not_enforced_at_runtime"
# subject = "typing.TypeVar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.TypeVar: a type-param bound is metadata only: def shout[T: str](x) still runs for a non-str arg (shout('hi')=='hi!' and shout(42)=='42!')"""


# A bound is metadata only: passing a value that violates it still runs.
def shout[T: str](x: T) -> str:
    return f"{x}!"


assert shout("hi") == "hi!"
assert shout(42) == "42!"  # int violates `T: str` but runs anyway

print("bound_not_enforced_at_runtime OK")
