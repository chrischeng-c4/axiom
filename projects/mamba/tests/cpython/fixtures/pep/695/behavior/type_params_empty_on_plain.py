# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "695"
# dimension = "behavior"
# case = "type_params_empty_on_plain"
# subject = "typing.TypeVar"
# kind = "semantic"
# xfail = "plain.__type_params__ returns None on mamba, not () (probed 2026-05-29)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.TypeVar: non-generic callables, classes, and builtin types carry an empty-but-present __type_params__ == () (plain fn, plain class, type, object)"""


# Non-generic callables and classes have an empty (but present) __type_params__.
def plain():
    pass


class Plain:
    pass


assert plain.__type_params__ == ()
assert Plain.__type_params__ == ()
# Even builtin types carry the attribute.
assert type.__type_params__ == ()
assert object.__type_params__ == ()

print("type_params_empty_on_plain OK")
