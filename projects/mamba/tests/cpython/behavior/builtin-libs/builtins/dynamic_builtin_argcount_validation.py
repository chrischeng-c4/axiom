# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "behavior"
# case = "dynamic_builtin_argcount_validation"
# subject = "builtins.dynamic-call-arity"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""Dynamic builtin calls validate argument counts instead of dropping extras."""

import builtins


try:
    builtins.len([1], 2)
    assert False
except TypeError:
    pass

try:
    builtins.int("10", 2, 99)
    assert False
except TypeError:
    pass

try:
    builtins.range(1, 2, 3, 4)
    assert False
except TypeError:
    pass

try:
    builtins.slice(1, 2, 3, 4)
    assert False
except TypeError:
    pass

ctor = int
assert ctor() == 0
try:
    ctor("10", 2, 99)
    assert False
except TypeError:
    pass

ctor = list
assert ctor() == []
try:
    ctor([1], [2])
    assert False
except TypeError:
    pass

ctor = dict
assert ctor() == {}
try:
    ctor({}, {})
    assert False
except TypeError:
    pass

ctor = range
try:
    ctor(1, 2, 3, 4)
    assert False
except TypeError:
    pass

ctor = bytes
assert ctor() == b""
try:
    ctor(b"a", "utf-8", "strict", 1)
    assert False
except TypeError:
    pass

print("dynamic_builtin_argcount_validation OK")
