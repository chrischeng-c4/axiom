# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""Instance protocol dunders: __call__/__bool__/__len__/__contains__ etc."""


class Proxy:
    def __init__(self, x):
        self.x = x

    def __call__(self, *args):
        return ("called", self.x, args)

    def __bool__(self):
        return bool(self.x)

    def __len__(self):
        return len(self.x)

    def __contains__(self, value):
        return value in self.x

    def __str__(self):
        return "Proxy:%s" % (self.x,)

    def __repr__(self):
        return "Proxy(%r)" % (self.x,)


p = Proxy([10, 20, 30])

# __call__ makes the instance callable.
assert callable(p)
assert p(1, 2) == ("called", [10, 20, 30], (1, 2))

# __bool__ drives truthiness.
assert p
assert not Proxy([])
assert not Proxy(0)

# __len__ powers len() and falls back for truthiness when __bool__ absent.
assert len(p) == 3

# __contains__ powers `in`.
assert 20 in p
assert 99 not in p

# __str__ vs __repr__ are distinct hooks.
assert str(p) == "Proxy:[10, 20, 30]"
assert repr(p) == "Proxy([10, 20, 30])"


# A class with __len__ but no __bool__ is falsy iff empty.
class Bag:
    def __init__(self, n):
        self.n = n

    def __len__(self):
        return self.n


assert Bag(2)
assert not Bag(0)


# Default object identity: distinct instances are unequal; same is equal;
# default str equals default repr.
class Plain:
    pass


a, b = Plain(), Plain()
assert a == a
assert a != b
assert not (a == b)
assert str(a) == repr(a)

print("dunder_protocols OK")
