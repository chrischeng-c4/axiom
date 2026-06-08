# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "695"
# dimension = "behavior"
# case = "nested_generator_yields_typevars"
# subject = "typing.TypeVar"
# kind = "semantic"
# xfail = "the yielded objects are not TypeVars with __name__ on mamba (type params erased; probed 2026-05-29)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.TypeVar: a nested generic generator (def make_gen[A] -> def gen[B]) yields the TypeVars of each enclosing scope, named 'A' and 'B'"""


# A nested generator can still yield the TypeVars from each enclosing scope.
def make_gen[A]():
    def gen[B]():
        yield A
        yield B
    return gen


first, second = list(make_gen()())
assert first.__name__ == "A"
assert second.__name__ == "B"

print("nested_generator_yields_typevars OK")
