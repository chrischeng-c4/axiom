# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""yield-from a non-generator iterable (e.g. range): next() delegates,
send() of a value hits the missing .send and raises AttributeError,
throw() propagates, and close() is silent (PEP 380)."""


# Plain-iterable delegation: values flow straight through.
def deleg_range():
    yield from range(3)


assert list(deleg_range()) == [0, 1, 2]
print("nongen next:", "ok")


# Delegating empty iterables yields nothing; first next() stops.
def deleg_empty():
    yield from ()
    yield from []


g = deleg_empty()
try:
    next(g)
    raise AssertionError("expected StopIteration")
except StopIteration:
    pass
print("nongen empty:", "ok")


# A range iterator has no .send, so sending a non-None value while
# delegating raises AttributeError; the generator's finally still runs.
trace = []


def deleg_send():
    try:
        yield from range(3)
    finally:
        trace.append("finish")


g = deleg_send()
assert next(g) == 0
try:
    g.send(42)
    raise AssertionError("expected AttributeError")
except AttributeError as e:
    assert "send" in str(e)
assert trace == ["finish"], trace
print("nongen send:", "ok")


# throw() while delegating to a non-generator subiterator propagates
# straight out (the subiterator cannot intercept it).
trace = []


def deleg_throw():
    try:
        yield from range(10)
    finally:
        trace.append("finish")


g = deleg_throw()
next(g)
try:
    g.throw(ValueError("boom"))
    raise AssertionError("expected ValueError")
except ValueError as e:
    assert e.args[0] == "boom"
assert trace == ["finish"], trace
print("nongen throw:", "ok")


# close() while delegating to a non-generator is silent (no error
# surfaces to the caller) but the generator's own finally runs.
trace = []


def deleg_close():
    try:
        yield from range(3)
    finally:
        trace.append("finish")


g = deleg_close()
next(g)
g.close()
assert trace == ["finish"], trace
print("nongen close:", "ok")
