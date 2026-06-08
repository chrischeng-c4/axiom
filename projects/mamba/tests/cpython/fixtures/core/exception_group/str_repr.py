# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/exception_group: str() and repr() of (nested) groups (3.12)."""


# str() summarises the message plus a sub-exception count.
beg = BaseExceptionGroup("flat", [ValueError(1), KeyboardInterrupt(2)])
assert str(beg) == "flat (2 sub-exceptions)", str(beg)

# repr() of a base group that holds a BaseException keeps the BEG name.
assert repr(beg) == (
    "BaseExceptionGroup('flat', [ValueError(1), KeyboardInterrupt(2)])"
), repr(beg)

# Nesting the group renders the inner repr recursively and recounts leaves.
nested = BaseExceptionGroup("nested", [beg, ValueError(1), beg])
assert str(nested) == "nested (3 sub-exceptions)", str(nested)
assert repr(nested) == (
    "BaseExceptionGroup('nested', ["
    "BaseExceptionGroup('flat', [ValueError(1), KeyboardInterrupt(2)]), "
    "ValueError(1), "
    "BaseExceptionGroup('flat', [ValueError(1), KeyboardInterrupt(2)])])"
), repr(nested)

# An all-Exception group reprs under the plain ExceptionGroup name.
eg = BaseExceptionGroup("flat", [ValueError(1), TypeError(2)])
assert repr(eg) == "ExceptionGroup('flat', [ValueError(1), TypeError(2)])", repr(eg)


# A subclass uses its own class name in the repr.
class MyEG(ExceptionGroup):
    pass


m = MyEG("flat", [ValueError(1), TypeError(2)])
assert str(m) == "flat (2 sub-exceptions)", str(m)
assert repr(m) == "MyEG('flat', [ValueError(1), TypeError(2)])", repr(m)

print("str_repr OK")
