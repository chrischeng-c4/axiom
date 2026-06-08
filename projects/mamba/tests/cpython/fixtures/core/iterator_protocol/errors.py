# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/<area>: iterator/generator/async error paths (CPython 3.12 oracle)."""


# Calling next() on an exhausted iterator raises StopIteration.
it = iter([1])
next(it)
try:
    next(it)
    print("exhausted: no_raise")
except StopIteration as e:
    print("exhausted:", type(e).__name__, str(e)[:40] or "(no msg)")


# Iter on non-iterable raises TypeError.
try:
    iter(42)  # type: ignore[call-overload]
    print("non_iter: no_raise")
except TypeError as e:
    print("non_iter:", type(e).__name__, str(e)[:60])


# Calling send() on a not-yet-started generator without None raises
# TypeError.
def gen():
    yield 1


g = gen()
try:
    g.send("not_none")
    print("send_unstarted: no_raise")
except TypeError as e:
    print("send_unstarted:", type(e).__name__, str(e)[:60])


# Calling throw() with a non-exception raises TypeError.
g2 = gen()
try:
    g2.throw(42)  # type: ignore[arg-type]
    print("throw_int: no_raise")
except TypeError as e:
    print("throw_int:", type(e).__name__, str(e)[:60])


# A class exposing __iter__ that returns self but NO __next__ is not a
# valid iterator: iter() rejects it because the returned object cannot
# be advanced.
class NoNext:
    def __iter__(self):
        return self


try:
    iter(NoNext())
    print("no_next: no_raise")
except TypeError as e:
    print("no_next:", type(e).__name__, str(e)[:60])


# An iterator whose __next__ vanishes mid-iteration (deleted from the
# class) stops being iterable: the for-loop's next lookup raises
# TypeError instead of silently halting.
class Vanishing:
    def __iter__(self):
        return self

    def __next__(self):
        del Vanishing.__next__
        return 1


try:
    for _ in Vanishing():
        pass
    print("vanishing_next: no_raise")
except TypeError as e:
    print("vanishing_next:", type(e).__name__, str(e)[:60])
