# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/class_system: language-area behavior asserts (CPython 3.12 oracle)."""

# Generic language behavior checks.
assert 1 + 1 == 2
assert "a" + "b" == "ab"
assert isinstance(True, int)
assert isinstance(1, int)
assert type(()) is tuple
assert type([]) is list
assert type({}) is dict
assert len("abc") == 3
assert list(range(3)) == [0, 1, 2]

# isinstance/issubclass over a user hierarchy and MRO basics.
class Base:
    pass
class Mid(Base):
    pass
class Leaf(Mid):
    pass

leaf = Leaf()
assert isinstance(leaf, Leaf)
assert isinstance(leaf, Mid)
assert isinstance(leaf, Base)
assert isinstance(leaf, object)
assert not isinstance(leaf, str)
assert issubclass(Leaf, Base)
assert issubclass(Leaf, object)
assert not issubclass(Base, Leaf)
assert Leaf.__mro__ == (Leaf, Mid, Base, object)

# isinstance accepts a tuple of types.
assert isinstance(leaf, (str, Base))
assert not isinstance(leaf, (str, bytes))

# type(x) and x.__class__ agree for a user instance.
assert type(leaf) is Leaf
assert leaf.__class__ is Leaf

print("behavior OK")
