# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "partial_repr_names_type_and_args"
# subject = "functools.partial"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.partial: repr(partial(...)) is module-qualified and reflects the captured positional args"""
import functools


def capture(*args, **kw):
    return (args, kw)


assert repr(functools.partial(capture)) == f"functools.partial({capture!r})", "repr bare"
assert repr(functools.partial(capture, 7)) == f"functools.partial({capture!r}, 7)", (
    "repr with positional"
)

print("partial_repr_names_type_and_args OK")
