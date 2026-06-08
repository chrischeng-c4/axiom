# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""Exceptions interacting with generator resumption (CPython 3.12 oracle)."""


# A bare `raise` inside a generator re-raises the exception active at the
# point where the generator is resumed.
def reraiser():
    yield 1
    raise
    yield 2  # noqa: unreachable


it = reraiser()
try:
    1 / 0
except ZeroDivisionError:
    next(it)  # advances past the first yield while ZeroDivisionError is active
    try:
        next(it)  # hits the bare `raise`
        raise AssertionError("expected ZeroDivisionError")
    except ZeroDivisionError:
        print("bare_raise: generator re-raised the active ZeroDivisionError")


# throw() injects into a yield nested inside a try/except; the inner handler
# swallows it, then a bare `raise` re-raises the outer handler's exception.
class MainError(Exception):
    pass


class SubError(Exception):
    pass


def nested():
    try:
        raise MainError()
    except MainError:
        try:
            yield
        except SubError:
            pass  # swallow the thrown SubError
        raise  # re-raise the still-active MainError


coro = nested()
coro.send(None)  # run to the yield, MainError active
try:
    coro.throw(SubError())
    raise AssertionError("expected MainError")
except MainError:
    print("throw_then_reraise: SubError swallowed, MainError re-raised")


# Returning from a generator surfaces the value on StopIteration.
def with_return():
    yield 1
    return "done"


g = with_return()
assert next(g) == 1
try:
    next(g)
    raise AssertionError("expected StopIteration")
except StopIteration as e:
    assert e.value == "done"
    print("return_value: StopIteration.value =", e.value)

print("generator_exceptions OK")
