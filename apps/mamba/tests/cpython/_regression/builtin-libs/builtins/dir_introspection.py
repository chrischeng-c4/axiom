# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""builtins: dir() — default listing and __dir__ override contract."""

# Distilled from CPython Lib/test/test_builtin.py BuiltinTest.test_dir
# (re-curated: dropped ModuleType/__slots__/traceback internals).

import sys

# dir() with no argument includes names bound in the current scope.
local_var = 1
assert "local_var" in dir()

# dir(module) includes its public attributes.
assert "version" in dir(sys)

# dir(type) lists methods but not dunder class internals like __mro__.
assert "strip" in dir(str)
assert "__mro__" not in dir(str)

# dir(instance) reflects instance and class attributes.
class Plain:
    def __init__(self):
        self.x = 7
        self.y = 8


p = Plain()
assert "x" in dir(p)
assert "y" in dir(p)

# A __dir__ returning a list is sorted by dir().
class CustomList:
    def __dir__(self):
        return ["kan", "ga", "roo"]


assert dir(CustomList()) == ["ga", "kan", "roo"]

# A __dir__ returning any iterable is coerced to a sorted list.
class CustomTuple:
    def __dir__(self):
        return ("b", "c", "a")


res = dir(CustomTuple())
assert isinstance(res, list)
assert res == ["a", "b", "c"]

# A __dir__ returning a set is also sorted into a list.
class CustomSet:
    def __dir__(self):
        return {"b", "c", "a"}


res = dir(CustomSet())
assert isinstance(res, list)
assert sorted(res) == ["a", "b", "c"]

# __dir__ returning a non-iterable raises TypeError.
class BadDir:
    def __dir__(self):
        return 7


try:
    dir(BadDir())
    raise AssertionError("expected TypeError")
except TypeError:
    pass

# dir() accepts at most one argument.
try:
    dir(42, 42)
    raise AssertionError("expected TypeError")
except TypeError:
    pass

# list.__dir__() sorts to the same set as dir(list).
assert sorted([].__dir__()) == dir([])

print("dir_introspection OK")
