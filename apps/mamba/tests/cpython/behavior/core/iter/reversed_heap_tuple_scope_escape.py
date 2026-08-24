# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "iter"
# dimension = "behavior"
# case = "reversed_heap_tuple_scope_escape"
# subject = "builtins.reversed"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_iter.py"
# status = "filled"
# ///
"""A reversed iterator retains a function-local heap tuple after return."""


def make_escaped_reverse():
    heap_101 = "heap-" + str(101) + "-alpha"
    heap_202 = "heap-" + str(202) + "-beta"
    heap_303 = "heap-" + str(303) + "-gamma"
    source = (heap_101, heap_202, heap_303)
    return reversed(source)


iterator = make_escaped_reverse()
first = next(iterator)
remaining = tuple(iterator)
assert first == "heap-303-gamma"
assert remaining == ("heap-202-beta", "heap-101-alpha")
try:
    next(iterator)
except StopIteration:
    pass
else:
    raise AssertionError("reversed iterator did not remain exhausted")

print("reversed-heap-tuple-scope-escape", first, *remaining, sep="|")
