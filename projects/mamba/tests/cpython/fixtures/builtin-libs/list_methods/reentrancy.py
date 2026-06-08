# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""list_methods: re-entrant mutation during element comparison.

CPython compares elements with an identity-first rich-compare, so a list is
never "equal" to itself by value, and an element whose __eq__/__lt__ mutates
a list mid-scan must not crash. These probe that hardening.
"""

# An element whose __eq__ clears the enclosing list and declines comparison.
class Clearer:
    def __eq__(self, other):
        host.clear()
        return NotImplemented

    __hash__ = None


# index() never matches the list against its own element -> ValueError.
host = [Clearer()]
try:
    host.index(host)
    raise AssertionError("expected ValueError")
except ValueError:
    pass

# A list subclass whose own __eq__ touches the argument but declines.
class DeclineEq(list):
    def __eq__(self, other):
        str(other)
        return NotImplemented

    __hash__ = None


# count() of the list within itself finds zero matches, no crash.
host = DeclineEq([Clearer()])
assert host.count(host) == 0

# remove() of the list within itself raises ValueError, no crash.
host = DeclineEq([Clearer()])
try:
    host.remove(host)
    raise AssertionError("expected ValueError")
except ValueError:
    pass

# `in` over elements that clear the list during comparison stays safe.
host = [Clearer(), Clearer()]
assert (3 in host) is False

# An element whose __lt__ clears the other operand mid-compare -> TypeError.
class EvilLt:
    def __lt__(self, other):
        other.clear()
        return NotImplemented


nested = [[EvilLt()]]
try:
    nested[0] < nested
    raise AssertionError("expected TypeError")
except TypeError:
    pass

# Slice-assigning a generator that clears the target mid-iteration must
# report the size mismatch on the extended slice, not crash.
class ClearingIter:
    def __init__(self, lst):
        self.lst = lst

    def __iter__(self):
        yield from self.lst
        self.lst.clear()


target = list(range(5))
try:
    target[::-1] = ClearingIter(target)
    raise AssertionError("expected ValueError")
except ValueError:
    pass

print("reentrancy OK")
