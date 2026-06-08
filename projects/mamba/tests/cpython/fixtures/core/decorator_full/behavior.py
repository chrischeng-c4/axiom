# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/decorator_full: decorator runtime behavior (CPython 3.12 oracle)."""

# Generic language sanity.
assert 1 + 1 == 2
assert isinstance(True, int)


# A class decorator receives and may mutate-and-return the class object.
def tag(value):
    def deco(cls):
        cls.extra = value
        return cls
    return deco


@tag("hello")
class Single:
    pass


assert Single.extra == "hello"


# Stacked class decorators apply bottom-up; later (outer) wins on overwrite,
# and accumulating decorators see each other's effect in order.
def set_ten(cls):
    cls.extra = 10
    return cls


def add_five(cls):
    cls.extra += 5
    return cls


@add_five   # runs second, sees extra == 10
@set_ten    # runs first
class Accumulate:
    pass


assert Accumulate.extra == 15


# A decorator factory may REPLACE the wrapped function; stacking applies the
# innermost factory first, so the outermost replacement is what survives.
def replace_with(num):
    def deco(func):
        return lambda: num
    return deco


@replace_with(2)   # applied last -> wins
@replace_with(1)   # applied first
def picked():
    return 42


assert picked() == 2


# A decorator may attach attributes to the function it returns; stacking lets
# several attribute-setters compose on one function object.
def funcattrs(**kwds):
    def deco(func):
        func.__dict__.update(kwds)
        return func
    return deco


class Holder:
    @funcattrs(abc=1, xyz="haha")
    @funcattrs(booh=42)
    def foo(self):
        return 42


assert Holder().foo() == 42
assert Holder.foo.abc == 1
assert Holder.foo.xyz == "haha"
assert Holder.foo.booh == 42


# Real-world stack: memoize over a call-counter. The counter sees one call per
# distinct (hashable) argument; unhashable args bypass the cache every time.
def countcalls(counts):
    def deco(func):
        counts[func.__name__] = 0

        def call(*args, **kwds):
            counts[func.__name__] += 1
            return func(*args, **kwds)
        call.__name__ = func.__name__
        return call
    return deco


def memoize(func):
    saved = {}

    def call(*args):
        try:
            return saved[args]
        except KeyError:
            res = func(*args)
            saved[args] = res
            return res
        except TypeError:  # unhashable args -> recompute
            return func(*args)
    call.__name__ = func.__name__
    return call


counts = {}


@memoize
@countcalls(counts)
def double(x):
    return x * 2


assert double.__name__ == "double"
assert counts == {"double": 0}
assert double(2) == 4 and counts["double"] == 1
assert double(2) == 4 and counts["double"] == 1   # cache hit, no recount
assert double(3) == 6 and counts["double"] == 2
assert double([10]) == [10, 10] and counts["double"] == 3   # unhashable
assert double([10]) == [10, 10] and counts["double"] == 4   # still uncached


# classmethod() applied to an already-bound method keeps that binding and
# ignores the implicit cls passed by the descriptor.
class Source:
    def foo(self, cls):
        return "spam"


class Bound:
    bar = classmethod(Source().foo)


assert Bound.bar() == "spam"

print("behavior OK")
