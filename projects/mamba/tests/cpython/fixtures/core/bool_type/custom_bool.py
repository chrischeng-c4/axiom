# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/bool_type: the truth-value protocol on user types — __bool__,
the __len__ fallback, blocking with __bool__ = None, and error propagation."""


# __bool__ is consulted at least once when a value drives a condition.
class Counter:
    def __init__(self):
        self.count = 0

    def __bool__(self):
        self.count += 1
        return True


c = Counter()
if c or True:
    pass
assert c.count >= 1


# With no __bool__, truthiness falls back to __len__: zero is falsy, nonzero truthy.
class Sized:
    def __init__(self, n):
        self.n = n

    def __len__(self):
        return self.n


assert bool(Sized(0)) is False
assert bool(Sized(3)) is True
if Sized(0):
    raise AssertionError("empty Sized should be falsy")


# Setting __bool__ = None disables truth-testing entirely (even if __len__ exists).
class Blocked:
    __bool__ = None


class BlockedWithLen:
    __bool__ = None

    def __len__(self):
        return 10


for cls in (Blocked, BlockedWithLen):
    try:
        bool(cls())
        raise AssertionError("expected TypeError for __bool__ = None")
    except TypeError:
        pass


# A __bool__ that raises propagates the error out of any boolean context,
# including conditions reached through a comparison's result.
class RaisingBool:
    def __bool__(self):
        raise TypeError("not a real boolean")


class Comparable:
    def __gt__(self, other):
        return RaisingBool()


try:
    if Comparable() > 0:
        pass
    raise AssertionError("expected TypeError from raising __bool__")
except TypeError:
    pass


# An illegal __len__ surfaces the same error whether reached via bool() or len().
for badval in ("illegal", -1):
    class BadLen:
        def __len__(self):
            return badval

    try:
        bool(BadLen())
        raise AssertionError("expected error from illegal __len__")
    except Exception as e_bool:
        try:
            len(BadLen())
            raise AssertionError("len should have raised too")
        except Exception as e_len:
            assert str(e_bool) == str(e_len)

print("custom_bool OK")
