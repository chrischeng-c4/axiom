# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "update_wrapper_custom_assigned_updated_tuples"
# subject = "functools.update_wrapper"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.update_wrapper: custom assigned tuples skip a missing source attribute and custom updated tuples merge mapping attributes in place"""
import functools


def src():
    pass


def wrapper():
    pass


# A missing assigned attribute on the source is simply skipped; an updated
# mapping attribute is merged in place (src has none, so it stays empty).
wrapper.dict_attr = {}
functools.update_wrapper(wrapper, src, assigned=("attr",), updated=("dict_attr",))
assert "attr" not in wrapper.__dict__, "missing assigned attr skipped"
assert wrapper.dict_attr == {}, "updated dict stays empty (src has none)"


# An updated attribute that is missing on the wrapper raises AttributeError.
del wrapper.dict_attr
try:
    functools.update_wrapper(
        wrapper, src, assigned=("attr",), updated=("dict_attr",)
    )
    raise AssertionError("expected AttributeError for missing updated attr")
except AttributeError:
    pass


# An updated attribute that is not a mapping also raises AttributeError
# (an int has no .update()).
wrapper.dict_attr = 1
try:
    functools.update_wrapper(
        wrapper, src, assigned=("attr",), updated=("dict_attr",)
    )
    raise AssertionError("expected AttributeError for non-mapping updated attr")
except AttributeError:
    pass

print("update_wrapper_custom_assigned_updated_tuples OK")
