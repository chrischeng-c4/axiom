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


# A rejected send() must not consume the generator: a following next()
# still yields the first value.
g3 = gen()
try:
    g3.send("not_none")
except TypeError:
    pass
print("send_then_next:", next(g3))


# PEP 479: a StopIteration that escapes a generator body is converted
# to RuntimeError (the original is attached as __cause__) instead of
# silently ending iteration.
def raises_stopiteration():
    raise StopIteration
    yield  # pragma: no cover


try:
    next(raises_stopiteration())
    print("pep479: no_raise")
except RuntimeError as e:
    print(
        "pep479:",
        "raised StopIteration" in str(e),
        type(e.__cause__).__name__,
    )
