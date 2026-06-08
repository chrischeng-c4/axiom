# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "abc"
# dimension = "errors"
# case = "instantiate_abstract_raises"
# subject = "abc.ABC"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_abc.py"
# status = "filled"
# ///
"""abc.ABC: instantiate_abstract_raises (errors)."""
import abc

_raised = False
try:
    type('A', (abc.ABC,), {'required': abc.abstractmethod(lambda self: 0)})()
except TypeError:
    _raised = True
assert _raised, "instantiate_abstract_raises: expected TypeError"
print("instantiate_abstract_raises OK")
