# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/context_manager: with-protocol error paths (CPython 3.12 oracle).

The `with` statement enforces the protocol and propagates failures from
__exit__: a missing dunder is a TypeError, and an exception raised inside
__exit__ replaces (and chains from) the in-flight exception.
"""


# Missing __exit__: TypeError raised at the `with` statement.
class NoExit:
    def __enter__(self):
        return self


try:
    with NoExit():
        print("no_exit: no_raise")
except TypeError as e:
    assert "does not support the context manager protocol" in str(e), str(e)
    print("no_exit:", type(e).__name__, "missed __exit__" in str(e))


# Missing __enter__: TypeError raised at the `with` statement.
class NoEnter:
    def __exit__(self, *a):
        return False


try:
    with NoEnter():
        print("no_enter: no_raise")
except TypeError as e:
    assert "does not support the context manager protocol" in str(e), str(e)
    print("no_enter:", type(e).__name__, "missed __enter__" in str(e))


# __exit__ raising while an exception is in flight: the new exception
# propagates and chains from the original via __context__.
class ExitRaises:
    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc, tb):
        raise TypeError("exit-fail")


try:
    with ExitRaises():
        raise ValueError("body-fail")
except TypeError as e:
    ctx = e.__context__
    assert isinstance(ctx, ValueError), repr(ctx)
    assert str(ctx) == "body-fail", str(ctx)
    print("exit_raises:", type(e).__name__, "ctx=" + type(ctx).__name__, str(ctx))


# __exit__ returning falsy lets the original exception propagate unchanged.
class NoSuppress:
    def __enter__(self):
        return self

    def __exit__(self, *a):
        return False


try:
    with NoSuppress():
        raise IndexError("idx")
except IndexError as e:
    print("no_suppress:", type(e).__name__, str(e))
