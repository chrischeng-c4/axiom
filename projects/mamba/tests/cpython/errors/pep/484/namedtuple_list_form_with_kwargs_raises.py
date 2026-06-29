# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "errors"
# case = "namedtuple_list_form_with_kwargs_raises"
# subject = "typing.NamedTuple"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.NamedTuple: namedtuple_list_form_with_kwargs_raises (errors)."""
import typing

_raised = False
try:
    typing.NamedTuple('Bad', [('x', int)], y=str)
except TypeError:
    _raised = True
assert _raised, "namedtuple_list_form_with_kwargs_raises: expected TypeError"
print("namedtuple_list_form_with_kwargs_raises OK")
