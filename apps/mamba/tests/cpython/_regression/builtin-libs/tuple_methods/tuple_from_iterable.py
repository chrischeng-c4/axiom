# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Regression: tuple(x) must consume custom iterators, generators, and dicts.
# Prior to the fix, tuple() on a user-class iterator returned (); tuple() on
# a generator leaked a trailing None from the StopIteration sentinel; tuple()
# on a dict returned () instead of the keys.

class CountDown:
    def __init__(self, n):
        self.n = n
    def __iter__(self):
        return self
    def __next__(self):
        if self.n <= 0:
            raise StopIteration
        self.n = self.n - 1
        return self.n + 1

# Custom iterator
print(tuple(CountDown(3)))

# Generator
def g():
    yield 1
    yield 2
    yield 3
print(tuple(g()))

# Dict (iterates keys)
print(tuple({1: "a", 2: "b", 3: "c"}))

# Set / frozenset / list / str / range sanity
print(tuple([10, 20, 30]))
print(tuple("abc"))
print(tuple(range(4)))
print(tuple(frozenset([7])))

# User iterable that returns a fresh iterator each __iter__ call
class Seq:
    def __init__(self, items):
        self.items = items
    def __iter__(self):
        return iter(self.items)

s = Seq([100, 200, 300])
print(tuple(s))
print(tuple(s))
