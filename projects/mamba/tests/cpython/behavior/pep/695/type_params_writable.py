# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "695"
# dimension = "behavior"
# case = "type_params_writable"
# subject = "typing.TypeVar"
# kind = "semantic"
# xfail = "cls/fn.__type_params__ returns None on mamba and the attribute is not writable (probed 2026-05-29)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.TypeVar: __type_params__ is writable on both a generic function and a generic class: assigning () is observed back"""


# __type_params__ is a writable attribute on both functions and classes.
def gen_fn[A]():
    pass


class GenCls[A]:
    pass


gen_fn.__type_params__ = ()
GenCls.__type_params__ = ()
assert gen_fn.__type_params__ == ()
assert GenCls.__type_params__ == ()

print("type_params_writable OK")
