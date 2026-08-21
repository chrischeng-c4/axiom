# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "errors"
# case = "defaultdict_noncallable_factory_raises"
# subject = "collections.defaultdict"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.defaultdict: defaultdict_noncallable_factory_raises (errors)."""
import collections

_raised = False
try:
    collections.defaultdict('not callable')
except TypeError:
    _raised = True
assert _raised, "defaultdict_noncallable_factory_raises: expected TypeError"
print("defaultdict_noncallable_factory_raises OK")
