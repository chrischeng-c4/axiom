# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "update_wrapper_from_builtin_resets_annotations"
# subject = "functools.update_wrapper"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.update_wrapper: update_wrapper from a builtin (max, type) copies name/doc and resets __annotations__ to empty"""
import functools


# Updating from the `max` builtin copies its name/doc and resets the
# wrapper's annotations to an empty dict.
def from_builtin():
    pass


functools.update_wrapper(from_builtin, max)
assert from_builtin.__name__ == "max", f"name = {from_builtin.__name__!r}"
assert from_builtin.__doc__.startswith("max("), f"doc = {from_builtin.__doc__!r}"
assert from_builtin.__annotations__ == {}, "annotations reset"


# Updating from the `type` builtin also yields empty annotations and an
# empty __type_params__ tuple.
def from_type(*args):
    pass


functools.update_wrapper(from_type, type)
assert from_type.__name__ == "type", f"name = {from_type.__name__!r}"
assert from_type.__annotations__ == {}, "type annotations"
assert from_type.__type_params__ == (), "type __type_params__ empty"

print("update_wrapper_from_builtin_resets_annotations OK")
